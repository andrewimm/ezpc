//! CPU state and register management
//!
//! The 8088 CPU state includes:
//! - General purpose registers (AX, CX, DX, BX, SP, BP, SI, DI)
//! - Segment registers (ES, CS, SS, DS)
//! - Instruction pointer (IP) and flags
//! - Lazy flag evaluation state
//! - Cycle counters
//! - Prefetch queue

use crate::memory::MemoryBus;

/// 8088 CPU state
pub struct Cpu {
    /// General purpose registers (16-bit)
    /// - Index 0: AX (accumulator)
    /// - Index 1: CX (counter)
    /// - Index 2: DX (data)
    /// - Index 3: BX (base)
    /// - Index 4: SP (stack pointer)
    /// - Index 5: BP (base pointer)
    /// - Index 6: SI (source index)
    /// - Index 7: DI (destination index)
    ///
    /// Stored as u16 for word access. Byte access uses low/high byte extraction.
    pub regs: [u16; 8],

    /// Segment registers (16-bit)
    /// - Index 0: ES (extra segment)
    /// - Index 1: CS (code segment)
    /// - Index 2: SS (stack segment)
    /// - Index 3: DS (data segment)
    pub segments: [u16; 4],

    /// Instruction pointer
    pub ip: u16,

    /// Flags register (computed lazily when needed)
    flags: u16,

    /// Lazy flag evaluation: stores last operation result
    /// Using u32 to handle carry flag computation for 16-bit operations
    last_result: u32,

    /// Lazy flag evaluation: operation type that produced last_result
    last_op: FlagOp,

    /// Total CPU cycles executed
    pub total_cycles: u64,

    /// Cycles spent on current instruction (reset at instruction start)
    pub current_instruction_cycles: u16,

    /// Prefetch queue (8088 has 4-byte queue)
    prefetch_queue: [u8; 4],

    /// Current number of bytes in prefetch queue
    prefetch_len: u8,

    /// Cycles spent filling prefetch queue
    prefetch_cycles: u16,

    /// Segment override prefix (None or segment index 0-3 for ES/CS/SS/DS)
    pub segment_override: Option<u8>,

    /// Repeat prefix for string operations
    pub repeat_prefix: RepeatPrefix,

    /// IP of the repeat prefix byte (used to loop back for REP)
    pub repeat_ip: u16,

    /// Delay interrupt recognition by one instruction (set by STI)
    /// The 8088 delays interrupt recognition after STI to allow STI;IRET sequences
    delay_interrupt: bool,

    /// CPU is halted (set by HLT instruction, cleared by interrupt)
    /// When halted, the CPU doesn't execute instructions but still checks for interrupts
    pub halted: bool,
}

/// Repeat prefix type for string operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RepeatPrefix {
    /// No repeat prefix
    None,
    /// REP/REPE prefix (0xF3) - repeat while ZF=1 (for CMPS/SCAS)
    Rep,
    /// REPNE prefix (0xF2) - repeat while ZF=0 (for CMPS/SCAS)
    RepNe,
}

/// Operation type for lazy flag evaluation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlagOp {
    None,
    Add8,
    Add16,
    Adc8,
    Adc16,
    Sub8,
    Sub16,
    Sbb8,
    Sbb16,
    And8,
    And16,
    Or8,
    Or16,
    Xor8,
    Xor16,
    Inc8,
    Inc16,
    Dec8,
    Dec16,
}

impl Cpu {
    /// Create a new CPU with reset state
    pub fn new() -> Self {
        Self {
            regs: [0; 8],
            segments: [0; 4],
            ip: 0,
            flags: 0,
            last_result: 0,
            last_op: FlagOp::None,
            total_cycles: 0,
            current_instruction_cycles: 0,
            prefetch_queue: [0; 4],
            prefetch_len: 0,
            prefetch_cycles: 0,
            segment_override: None,
            repeat_prefix: RepeatPrefix::None,
            repeat_ip: 0,
            delay_interrupt: false,
            halted: false,
        }
    }

    /// Reset CPU to power-on state
    /// On 8088, CS starts at 0xF000 and IP at 0xFFF0 (reset vector at F000:FFF0)
    pub fn reset(&mut self) {
        self.regs = [0; 8];
        self.segments = [0; 4];
        self.segments[1] = 0xF000; // CS = 0xF000
        self.ip = 0xFFF0; // IP = 0xFFF0
        self.flags = 0x0002; // Bit 1 is always set on 8088
        self.last_result = 0;
        self.last_op = FlagOp::None;
        self.total_cycles = 0;
        self.current_instruction_cycles = 0;
        self.prefetch_queue = [0; 4];
        self.prefetch_len = 0;
        self.prefetch_cycles = 0;
        self.segment_override = None;
        self.repeat_prefix = RepeatPrefix::None;
        self.repeat_ip = 0;
        self.halted = false;
    }

    // === Register Access Methods ===

    /// Read an 8-bit register
    /// Register encoding: 0=AL, 1=CL, 2=DL, 3=BL, 4=AH, 5=CH, 6=DH, 7=BH
    #[inline(always)]
    pub fn read_reg8(&self, reg: u8) -> u8 {
        let idx = (reg & 0b11) as usize;
        if reg < 4 {
            // Low byte (AL, CL, DL, BL)
            self.regs[idx] as u8
        } else {
            // High byte (AH, CH, DH, BH)
            (self.regs[idx] >> 8) as u8
        }
    }

    /// Write an 8-bit register
    /// Register encoding: 0=AL, 1=CL, 2=DL, 3=BL, 4=AH, 5=CH, 6=DH, 7=BH
    #[inline(always)]
    pub fn write_reg8(&mut self, reg: u8, value: u8) {
        let idx = (reg & 0b11) as usize;
        if reg < 4 {
            // Low byte (AL, CL, DL, BL)
            self.regs[idx] = (self.regs[idx] & 0xFF00) | (value as u16);
        } else {
            // High byte (AH, CH, DH, BH)
            self.regs[idx] = (self.regs[idx] & 0x00FF) | ((value as u16) << 8);
        }
    }

    /// Read a 16-bit register
    /// Register encoding: 0=AX, 1=CX, 2=DX, 3=BX, 4=SP, 5=BP, 6=SI, 7=DI
    #[inline(always)]
    pub fn read_reg16(&self, reg: u8) -> u16 {
        self.regs[(reg & 0b111) as usize]
    }

    /// Write a 16-bit register
    /// Register encoding: 0=AX, 1=CX, 2=DX, 3=BX, 4=SP, 5=BP, 6=SI, 7=DI
    #[inline(always)]
    pub fn write_reg16(&mut self, reg: u8, value: u16) {
        self.regs[(reg & 0b111) as usize] = value;
    }

    /// Read a segment register
    /// Segment encoding: 0=ES, 1=CS, 2=SS, 3=DS
    #[inline(always)]
    pub fn read_seg(&self, seg: u8) -> u16 {
        self.segments[(seg & 0b11) as usize]
    }

    /// Write a segment register
    /// Segment encoding: 0=ES, 1=CS, 2=SS, 3=DS
    #[inline(always)]
    pub fn write_seg(&mut self, seg: u8, value: u16) {
        self.segments[(seg & 0b11) as usize] = value;
    }

    // === Memory Access Methods ===

    /// Compute physical address from segment:offset
    /// Physical address = (segment << 4) + offset
    #[inline(always)]
    pub fn compute_address(segment: u16, offset: u16) -> u32 {
        ((segment as u32) << 4) + (offset as u32)
    }

    /// Read a byte from memory using segment:offset addressing
    #[inline(always)]
    pub fn read_mem8(&self, mem: &MemoryBus, segment: u16, offset: u16) -> u8 {
        let addr = Self::compute_address(segment, offset);
        mem.read_u8(addr)
    }

    /// Write a byte to memory using segment:offset addressing
    #[inline(always)]
    pub fn write_mem8(&mut self, mem: &mut MemoryBus, segment: u16, offset: u16, value: u8) {
        let addr = Self::compute_address(segment, offset);
        mem.write_u8(addr, value);
    }

    /// Read a word from memory using segment:offset addressing
    #[inline(always)]
    pub fn read_mem16(&self, mem: &MemoryBus, segment: u16, offset: u16) -> u16 {
        let addr = Self::compute_address(segment, offset);
        mem.read_u16(addr)
    }

    /// Write a word to memory using segment:offset addressing
    #[inline(always)]
    pub fn write_mem16(&mut self, mem: &mut MemoryBus, segment: u16, offset: u16, value: u16) {
        let addr = Self::compute_address(segment, offset);
        mem.write_u16(addr, value);
    }

    // === Lazy Flag Evaluation ===

    /// Flag bit positions
    pub const CF: u16 = 1 << 0; // Carry
    pub const PF: u16 = 1 << 2; // Parity
    pub const AF: u16 = 1 << 4; // Auxiliary carry
    pub const ZF: u16 = 1 << 6; // Zero
    pub const SF: u16 = 1 << 7; // Sign
    pub const TF: u16 = 1 << 8; // Trap
    pub const IF: u16 = 1 << 9; // Interrupt enable
    pub const DF: u16 = 1 << 10; // Direction
    pub const OF: u16 = 1 << 11; // Overflow

    /// Set lazy flag state after an operation
    #[inline(always)]
    pub fn set_lazy_flags(&mut self, result: u32, op: FlagOp) {
        self.last_result = result;
        self.last_op = op;
    }

    /// Compute flags from lazy state
    /// OF, AF, and control flags (DF, IF, TF) are set eagerly and preserved from self.flags
    /// Other flags (CF, ZF, SF, PF) are computed lazily from last_result and last_op
    fn compute_flags(&self) -> u16 {
        let mut flags = 0b0010; // Bit 1 always set on 8088

        // Preserve OF, AF, and control flags (DF, IF, TF) which are set eagerly
        flags |= self.flags & (Self::OF | Self::AF | Self::DF | Self::IF | Self::TF);

        match self.last_op {
            FlagOp::None => return self.flags | 0b0010, // Ensure bit 1 is set

            FlagOp::Add8
            | FlagOp::Adc8
            | FlagOp::Sub8
            | FlagOp::Sbb8
            | FlagOp::Inc8
            | FlagOp::Dec8
            | FlagOp::And8
            | FlagOp::Or8
            | FlagOp::Xor8 => {
                let result = self.last_result as u8;

                // Zero flag
                if result == 0 {
                    flags |= Self::ZF;
                }

                // Sign flag (bit 7 for 8-bit)
                if result & 0x80 != 0 {
                    flags |= Self::SF;
                }

                // Parity flag (even parity of low 8 bits)
                if result.count_ones() % 2 == 0 {
                    flags |= Self::PF;
                }

                // Carry flag (bit 8 for 8-bit operations)
                if matches!(
                    self.last_op,
                    FlagOp::Add8 | FlagOp::Adc8 | FlagOp::Sub8 | FlagOp::Sbb8
                ) {
                    if self.last_result & 0x100 != 0 {
                        flags |= Self::CF;
                    }
                }

                // Overflow flag for 8-bit operations
                if matches!(self.last_op, FlagOp::Add8 | FlagOp::Sub8) {
                    // OF is set if sign bit changed incorrectly
                    // This is a simplified check; full implementation needs operands
                }
            }

            FlagOp::Add16
            | FlagOp::Adc16
            | FlagOp::Sub16
            | FlagOp::Sbb16
            | FlagOp::Inc16
            | FlagOp::Dec16
            | FlagOp::And16
            | FlagOp::Or16
            | FlagOp::Xor16 => {
                let result = self.last_result as u16;

                // Zero flag
                if result == 0 {
                    flags |= Self::ZF;
                }

                // Sign flag (bit 15 for 16-bit)
                if result & 0x8000 != 0 {
                    flags |= Self::SF;
                }

                // Parity flag (even parity of low 8 bits)
                if (result as u8).count_ones() % 2 == 0 {
                    flags |= Self::PF;
                }

                // Carry flag (bit 16 for 16-bit operations)
                if matches!(
                    self.last_op,
                    FlagOp::Add16 | FlagOp::Adc16 | FlagOp::Sub16 | FlagOp::Sbb16
                ) {
                    if self.last_result & 0x10000 != 0 {
                        flags |= Self::CF;
                    }
                }
            }
        }

        flags
    }

    /// Get the flags register (triggers flag computation)
    #[inline(always)]
    pub fn get_flags(&mut self) -> u16 {
        self.flags = self.compute_flags();
        self.flags
    }

    /// Set the flags register directly
    #[inline(always)]
    pub fn set_flags(&mut self, flags: u16) {
        self.flags = flags | 0b0010; // Bit 1 always set
        self.last_op = FlagOp::None;
    }

    /// Check if a flag is set (triggers computation if needed)
    #[inline(always)]
    pub fn get_flag(&mut self, flag: u16) -> bool {
        self.get_flags() & flag != 0
    }

    /// Set or clear a specific flag
    #[inline(always)]
    pub fn set_flag(&mut self, flag: u16, value: bool) {
        self.flags = self.compute_flags();
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
        self.last_op = FlagOp::None;
    }

    /// Compute and set OF and AF flags for 8-bit ADD operation
    /// OF: overflow when two same-sign operands produce opposite-sign result
    /// AF: auxiliary carry from bit 3 to bit 4
    #[inline(always)]
    pub fn set_add8_of_af(&mut self, op1: u8, op2: u8, result: u32) {
        // Overflow: ((op1 ^ result) & (op2 ^ result)) & 0x80
        let overflow = ((op1 as u32 ^ result) & (op2 as u32 ^ result)) & 0x80;
        if overflow != 0 {
            self.flags |= Self::OF;
        } else {
            self.flags &= !Self::OF;
        }

        // Auxiliary carry: carry from bit 3 to bit 4
        let aux_carry = ((op1 as u32) ^ (op2 as u32) ^ result) & 0x10;
        if aux_carry != 0 {
            self.flags |= Self::AF;
        } else {
            self.flags &= !Self::AF;
        }
    }

    /// Compute and set OF and AF flags for 16-bit ADD operation
    #[inline(always)]
    pub fn set_add16_of_af(&mut self, op1: u16, op2: u16, result: u32) {
        // Overflow: ((op1 ^ result) & (op2 ^ result)) & 0x8000
        let overflow = ((op1 as u32 ^ result) & (op2 as u32 ^ result)) & 0x8000;
        if overflow != 0 {
            self.flags |= Self::OF;
        } else {
            self.flags &= !Self::OF;
        }

        // Auxiliary carry: carry from bit 3 to bit 4
        let aux_carry = ((op1 as u32) ^ (op2 as u32) ^ result) & 0x10;
        if aux_carry != 0 {
            self.flags |= Self::AF;
        } else {
            self.flags &= !Self::AF;
        }
    }

    /// Compute and set OF and AF flags for 8-bit SUB operation
    /// OF: overflow when subtracting different-sign operands produces wrong sign
    /// AF: auxiliary borrow from bit 4 to bit 3
    #[inline(always)]
    pub fn set_sub8_of_af(&mut self, op1: u8, op2: u8, result: u32) {
        // Overflow: ((op1 ^ op2) & (op1 ^ result)) & 0x80
        let overflow = ((op1 as u32 ^ op2 as u32) & (op1 as u32 ^ result)) & 0x80;
        if overflow != 0 {
            self.flags |= Self::OF;
        } else {
            self.flags &= !Self::OF;
        }

        // Auxiliary borrow: borrow from bit 4 to bit 3
        let aux_borrow = ((op1 as u32) ^ (op2 as u32) ^ result) & 0x10;
        if aux_borrow != 0 {
            self.flags |= Self::AF;
        } else {
            self.flags &= !Self::AF;
        }
    }

    /// Compute and set OF and AF flags for 16-bit SUB operation
    #[inline(always)]
    pub fn set_sub16_of_af(&mut self, op1: u16, op2: u16, result: u32) {
        // Overflow: ((op1 ^ op2) & (op1 ^ result)) & 0x8000
        let overflow = ((op1 as u32 ^ op2 as u32) & (op1 as u32 ^ result)) & 0x8000;
        if overflow != 0 {
            self.flags |= Self::OF;
        } else {
            self.flags &= !Self::OF;
        }

        // Auxiliary borrow: borrow from bit 4 to bit 3
        let aux_borrow = ((op1 as u32) ^ (op2 as u32) ^ result) & 0x10;
        if aux_borrow != 0 {
            self.flags |= Self::AF;
        } else {
            self.flags &= !Self::AF;
        }
    }

    /// Compute and set OF and AF flags for 8-bit INC operation
    /// INC doesn't affect CF, and OF is set when incrementing 0x7F
    #[inline(always)]
    pub fn set_inc8_of_af(&mut self, op1: u8, result: u8) {
        // Overflow only when 0x7F + 1 = 0x80 (max positive to min negative)
        if op1 == 0x7F && result == 0x80 {
            self.flags |= Self::OF;
        } else {
            self.flags &= !Self::OF;
        }

        // Auxiliary carry from bit 3 to bit 4
        let aux_carry = ((op1 as u32) ^ 1 ^ (result as u32)) & 0x10;
        if aux_carry != 0 {
            self.flags |= Self::AF;
        } else {
            self.flags &= !Self::AF;
        }
    }

    /// Compute and set OF and AF flags for 16-bit INC operation
    #[inline(always)]
    pub fn set_inc16_of_af(&mut self, op1: u16, result: u16) {
        // Overflow only when 0x7FFF + 1 = 0x8000
        if op1 == 0x7FFF && result == 0x8000 {
            self.flags |= Self::OF;
        } else {
            self.flags &= !Self::OF;
        }

        // Auxiliary carry from bit 3 to bit 4
        let aux_carry = ((op1 as u32) ^ 1 ^ (result as u32)) & 0x10;
        if aux_carry != 0 {
            self.flags |= Self::AF;
        } else {
            self.flags &= !Self::AF;
        }
    }

    /// Compute and set OF and AF flags for 8-bit DEC operation
    /// DEC doesn't affect CF, and OF is set when decrementing 0x80
    #[inline(always)]
    pub fn set_dec8_of_af(&mut self, op1: u8, result: u8) {
        // Overflow only when 0x80 - 1 = 0x7F (min negative to max positive)
        if op1 == 0x80 && result == 0x7F {
            self.flags |= Self::OF;
        } else {
            self.flags &= !Self::OF;
        }

        // Auxiliary borrow from bit 4 to bit 3
        let aux_borrow = ((op1 as u32) ^ 1 ^ (result as u32)) & 0x10;
        if aux_borrow != 0 {
            self.flags |= Self::AF;
        } else {
            self.flags &= !Self::AF;
        }
    }

    /// Compute and set OF and AF flags for 16-bit DEC operation
    #[inline(always)]
    pub fn set_dec16_of_af(&mut self, op1: u16, result: u16) {
        // Overflow only when 0x8000 - 1 = 0x7FFF
        if op1 == 0x8000 && result == 0x7FFF {
            self.flags |= Self::OF;
        } else {
            self.flags &= !Self::OF;
        }

        // Auxiliary borrow from bit 4 to bit 3
        let aux_borrow = ((op1 as u32) ^ 1 ^ (result as u32)) & 0x10;
        if aux_borrow != 0 {
            self.flags |= Self::AF;
        } else {
            self.flags &= !Self::AF;
        }
    }

    /// Compute and set OF and AF flags for 8-bit ADC operation
    /// ADC uses same overflow/auxiliary carry computation as ADD
    /// OF: overflow when two same-sign operands produce opposite-sign result
    /// AF: auxiliary carry from bit 3 to bit 4
    #[inline(always)]
    pub fn set_adc8_of_af(&mut self, op1: u8, op2: u8, carry: u8, result: u32) {
        // For overflow, we compute: ((op1 ^ result) & (op2 ^ result)) & 0x80
        // This detects when adding two same-sign numbers produces opposite sign
        let overflow = ((op1 as u32 ^ result) & (op2 as u32 ^ result)) & 0x80;
        if overflow != 0 {
            self.flags |= Self::OF;
        } else {
            self.flags &= !Self::OF;
        }

        // Auxiliary carry: carry from bit 3 to bit 4
        // XOR all three operands (op1, op2, carry) with result to detect bit 4 change
        let aux_carry = ((op1 as u32) ^ (op2 as u32) ^ (carry as u32) ^ result) & 0x10;
        if aux_carry != 0 {
            self.flags |= Self::AF;
        } else {
            self.flags &= !Self::AF;
        }
    }

    /// Compute and set OF and AF flags for 16-bit ADC operation
    #[inline(always)]
    pub fn set_adc16_of_af(&mut self, op1: u16, op2: u16, carry: u16, result: u32) {
        // Overflow: ((op1 ^ result) & (op2 ^ result)) & 0x8000
        let overflow = ((op1 as u32 ^ result) & (op2 as u32 ^ result)) & 0x8000;
        if overflow != 0 {
            self.flags |= Self::OF;
        } else {
            self.flags &= !Self::OF;
        }

        // Auxiliary carry: carry from bit 3 to bit 4
        let aux_carry = ((op1 as u32) ^ (op2 as u32) ^ (carry as u32) ^ result) & 0x10;
        if aux_carry != 0 {
            self.flags |= Self::AF;
        } else {
            self.flags &= !Self::AF;
        }
    }

    /// Compute and set OF and AF flags for 8-bit SBB operation
    /// SBB: op1 - op2 - borrow
    /// OF: overflow flag
    /// AF: auxiliary borrow from bit 4 to bit 3
    #[inline(always)]
    pub fn set_sbb8_of_af(&mut self, op1: u8, op2: u8, borrow: u8, result: u32) {
        // Overflow: ((op1 ^ op2) & (op1 ^ result)) & 0x80
        let overflow = ((op1 as u32 ^ op2 as u32) & (op1 as u32 ^ result)) & 0x80;
        if overflow != 0 {
            self.flags |= Self::OF;
        } else {
            self.flags &= !Self::OF;
        }

        // Auxiliary borrow: borrow from bit 4 to bit 3
        // XOR all three operands (op1, op2, borrow) with result to detect bit 4 change
        let aux_borrow = ((op1 as u32) ^ (op2 as u32) ^ (borrow as u32) ^ result) & 0x10;
        if aux_borrow != 0 {
            self.flags |= Self::AF;
        } else {
            self.flags &= !Self::AF;
        }
    }

    /// Compute and set OF and AF flags for 16-bit SBB operation
    /// SBB: op1 - op2 - borrow
    /// OF: overflow flag
    /// AF: auxiliary borrow from bit 4 to bit 3
    #[inline(always)]
    pub fn set_sbb16_of_af(&mut self, op1: u16, op2: u16, borrow: u16, result: u32) {
        // Overflow: ((op1 ^ op2) & (op1 ^ result)) & 0x8000
        let overflow = ((op1 as u32 ^ op2 as u32) & (op1 as u32 ^ result)) & 0x8000;
        if overflow != 0 {
            self.flags |= Self::OF;
        } else {
            self.flags &= !Self::OF;
        }

        // Auxiliary borrow: borrow from bit 4 to bit 3
        let aux_borrow = ((op1 as u32) ^ (op2 as u32) ^ (borrow as u32) ^ result) & 0x10;
        if aux_borrow != 0 {
            self.flags |= Self::AF;
        } else {
            self.flags &= !Self::AF;
        }
    }

    /// Clear OF, CF, and AF flags (used by logical operations)
    /// Logical operations (AND, OR, XOR) always clear CF and OF, and leave AF undefined
    #[inline(always)]
    pub fn clear_of_cf_af(&mut self) {
        self.flags &= !(Self::OF | Self::CF | Self::AF);
    }

    // === Instruction Decoding Methods ===

    /// Fetch a byte from CS:IP and advance IP
    #[inline(always)]
    pub fn fetch_u8(&mut self, mem: &MemoryBus) -> u8 {
        let byte = self.read_mem8(mem, self.segments[1], self.ip);
        self.ip = self.ip.wrapping_add(1);
        byte
    }

    /// Fetch a word from CS:IP and advance IP (little-endian)
    #[inline(always)]
    pub fn fetch_u16(&mut self, mem: &MemoryBus) -> u16 {
        let low = self.fetch_u8(mem) as u16;
        let high = self.fetch_u8(mem) as u16;
        (high << 8) | low
    }

    /// Fetch a signed byte from CS:IP and advance IP
    #[inline(always)]
    pub fn fetch_i8(&mut self, mem: &MemoryBus) -> i8 {
        self.fetch_u8(mem) as i8
    }

    /// Fetch a signed word from CS:IP and advance IP (little-endian)
    #[inline(always)]
    pub fn fetch_i16(&mut self, mem: &MemoryBus) -> i16 {
        self.fetch_u16(mem) as i16
    }

    /// Decode a ModR/M byte from CS:IP
    /// Returns the decoded ModR/M with any displacement/address loaded
    pub fn decode_modrm(&mut self, mem: &MemoryBus) -> crate::cpu::decode::ModRM {
        use crate::cpu::decode::{AddressingMode, ModRM};

        let modrm_byte = self.fetch_u8(mem);
        let mut modrm = ModRM::decode(modrm_byte);

        // Read displacement or address if needed
        modrm = match modrm.mode {
            AddressingMode::MemoryDisp8 { .. } => {
                let disp = self.fetch_i8(mem);
                modrm.with_disp8(disp)
            }
            AddressingMode::MemoryDisp16 { .. } => {
                let disp = self.fetch_i16(mem);
                modrm.with_disp16(disp)
            }
            AddressingMode::DirectAddress { .. } => {
                let addr = self.fetch_u16(mem);
                modrm.with_direct_addr(addr)
            }
            _ => modrm,
        };

        modrm
    }

    /// Decode a register operand from a ModR/M reg field
    #[inline(always)]
    pub fn decode_reg_operand(reg: u8, is_byte: bool) -> crate::cpu::decode::Operand {
        use crate::cpu::decode::Operand;
        if is_byte {
            Operand::reg8(reg)
        } else {
            Operand::reg16(reg)
        }
    }

    /// Decode a register/memory operand from a ModR/M r/m field
    pub fn decode_rm_operand(
        modrm: &crate::cpu::decode::ModRM,
        is_byte: bool,
    ) -> crate::cpu::decode::Operand {
        use crate::cpu::decode::{AddressingMode, Operand};

        match modrm.mode {
            AddressingMode::RegisterDirect { rm_reg } => {
                if is_byte {
                    Operand::reg8(rm_reg)
                } else {
                    Operand::reg16(rm_reg)
                }
            }
            AddressingMode::MemoryIndirect { base_index } => {
                if is_byte {
                    Operand::mem8(base_index)
                } else {
                    Operand::mem16(base_index)
                }
            }
            AddressingMode::MemoryDisp8 { base_index, disp } => {
                if is_byte {
                    Operand::mem8_disp(base_index, disp as i16)
                } else {
                    Operand::mem16_disp(base_index, disp as i16)
                }
            }
            AddressingMode::MemoryDisp16 { base_index, disp } => {
                if is_byte {
                    Operand::mem8_disp(base_index, disp)
                } else {
                    Operand::mem16_disp(base_index, disp)
                }
            }
            AddressingMode::DirectAddress { addr } => {
                // Direct addressing: use 0xFF sentinel in value field, address in disp field
                if is_byte {
                    Operand::mem8_disp(0xFF, addr as i16)
                } else {
                    Operand::mem16_disp(0xFF, addr as i16)
                }
            }
        }
    }

    // === Operand Read/Write Methods ===

    /// Read the value of an operand
    /// This is used by instruction handlers to get operand values
    #[inline(always)]
    pub fn read_operand(&self, mem: &MemoryBus, operand: &crate::cpu::decode::Operand) -> u16 {
        use crate::cpu::decode::OperandType;

        match operand.op_type {
            OperandType::None => 0,
            OperandType::Reg8 => self.read_reg8(operand.value as u8) as u16,
            OperandType::Reg16 => self.read_reg16(operand.value as u8),
            OperandType::SegReg => self.read_seg(operand.value as u8),
            OperandType::Imm8 | OperandType::Imm16 => operand.value,
            OperandType::Mem8 | OperandType::Mem16 => {
                // For memory operands, check if this is direct addressing or indirect
                // Direct addressing: value == 0xFF (sentinel), address in disp field
                // Indirect addressing: value 0-7 (base_index encoding)
                // Note: For group instructions, high byte contains reg field, so mask it off
                let base_index = (operand.value & 0xFF) as u8;

                if base_index == 0xFF {
                    // Direct addressing [disp16]: address is in disp field
                    let segment = if operand.segment != 0xFF {
                        self.read_seg(operand.segment)
                    } else {
                        self.segments[3] // DS default for direct addressing
                    };
                    let addr = operand.disp as u16;
                    if operand.op_type == OperandType::Mem8 {
                        self.read_mem8(mem, segment, addr) as u16
                    } else {
                        self.read_mem16(mem, segment, addr)
                    }
                } else {
                    // Indirect addressing: calculate EA from base_index
                    let (seg_idx, offset) = self.calculate_ea_from_operand(operand, base_index);

                    // Use segment override if present, otherwise use default from EA calculation
                    let segment = if operand.segment != 0xFF {
                        self.read_seg(operand.segment)
                    } else {
                        self.segments[seg_idx as usize]
                    };

                    if operand.op_type == OperandType::Mem8 {
                        self.read_mem8(mem, segment, offset) as u16
                    } else {
                        self.read_mem16(mem, segment, offset)
                    }
                }
            }
            OperandType::Direct => {
                // Direct addressing: operand.value is the offset
                let segment = if operand.segment != 0xFF {
                    self.read_seg(operand.segment)
                } else {
                    self.segments[3] // DS default
                };
                self.read_mem16(mem, segment, operand.value)
            }
            OperandType::Rel8 | OperandType::Rel16 => operand.value,
        }
    }

    /// Write a value to an operand
    /// This is used by instruction handlers to write results
    #[inline(always)]
    pub fn write_operand(
        &mut self,
        mem: &mut MemoryBus,
        operand: &crate::cpu::decode::Operand,
        value: u16,
    ) {
        use crate::cpu::decode::OperandType;

        match operand.op_type {
            OperandType::None => { /* No destination */ }
            OperandType::Reg8 => self.write_reg8(operand.value as u8, value as u8),
            OperandType::Reg16 => self.write_reg16(operand.value as u8, value),
            OperandType::SegReg => self.write_seg(operand.value as u8, value),
            OperandType::Mem8 | OperandType::Mem16 => {
                // For memory operands, check if this is direct addressing or indirect
                // Direct addressing: value == 0xFF (sentinel), address in disp field
                // Indirect addressing: value 0-7 (base_index encoding)
                // Note: For group instructions, high byte contains reg field, so mask it off
                let base_index = (operand.value & 0xFF) as u8;

                if base_index == 0xFF {
                    // Direct addressing [disp16]: address is in disp field
                    let segment = if operand.segment != 0xFF {
                        self.read_seg(operand.segment)
                    } else {
                        self.segments[3] // DS default for direct addressing
                    };
                    let addr = operand.disp as u16;
                    if operand.op_type == OperandType::Mem8 {
                        self.write_mem8(mem, segment, addr, value as u8);
                    } else {
                        self.write_mem16(mem, segment, addr, value);
                    }
                } else {
                    // Indirect addressing: calculate EA from base_index
                    let (seg_idx, offset) = self.calculate_ea_from_operand(operand, base_index);

                    // Use segment override if present, otherwise use default from EA calculation
                    let segment = if operand.segment != 0xFF {
                        self.read_seg(operand.segment)
                    } else {
                        self.segments[seg_idx as usize]
                    };

                    if operand.op_type == OperandType::Mem8 {
                        self.write_mem8(mem, segment, offset, value as u8);
                    } else {
                        self.write_mem16(mem, segment, offset, value);
                    }
                }
            }
            OperandType::Direct => {
                // Direct addressing: operand.value is the offset
                let segment = if operand.segment != 0xFF {
                    self.read_seg(operand.segment)
                } else {
                    self.segments[3] // DS default
                };
                self.write_mem16(mem, segment, operand.value, value);
            }
            OperandType::Imm8 | OperandType::Imm16 | OperandType::Rel8 | OperandType::Rel16 => {
                panic!("Cannot write to immediate or relative operand")
            }
        }
    }

    /// Helper to calculate effective address from operand encoding
    /// Calculate effective address from operand
    /// Returns (segment_index, effective_address)
    pub fn calculate_ea_from_operand(
        &self,
        operand: &crate::cpu::decode::Operand,
        base_index: u8,
    ) -> (u8, u16) {
        let disp = operand.disp as u16;

        // Effective address calculation based on r/m field
        // See Intel 8086 manual Table 2-2
        match base_index {
            0b000 => {
                // [BX + SI + disp]
                let ea = self
                    .read_reg16(3)
                    .wrapping_add(self.read_reg16(6))
                    .wrapping_add(disp);
                (3, ea) // DS default
            }
            0b001 => {
                // [BX + DI + disp]
                let ea = self
                    .read_reg16(3)
                    .wrapping_add(self.read_reg16(7))
                    .wrapping_add(disp);
                (3, ea) // DS default
            }
            0b010 => {
                // [BP + SI + disp]
                let ea = self
                    .read_reg16(5)
                    .wrapping_add(self.read_reg16(6))
                    .wrapping_add(disp);
                (2, ea) // SS default
            }
            0b011 => {
                // [BP + DI + disp]
                let ea = self
                    .read_reg16(5)
                    .wrapping_add(self.read_reg16(7))
                    .wrapping_add(disp);
                (2, ea) // SS default
            }
            0b100 => {
                // [SI + disp]
                let ea = self.read_reg16(6).wrapping_add(disp);
                (3, ea) // DS default
            }
            0b101 => {
                // [DI + disp]
                let ea = self.read_reg16(7).wrapping_add(disp);
                (3, ea) // DS default
            }
            0b110 => {
                // [BP + disp]
                let ea = self.read_reg16(5).wrapping_add(disp);
                (2, ea) // SS default
            }
            0b111 => {
                // [BX + disp]
                let ea = self.read_reg16(3).wrapping_add(disp);
                (3, ea) // DS default
            }
            _ => {
                panic!(
                    "Invalid base_index encoding in calculate_ea_from_operand: 0x{:02X}\n\
                     CS:IP = {:04X}:{:04X} (IP has advanced past instruction start)\n\
                     Operand: type={:?}, value=0x{:04X}, disp=0x{:04X}, segment=0x{:02X}",
                    base_index,
                    self.segments[1],
                    self.ip,
                    operand.op_type,
                    operand.value,
                    operand.disp,
                    operand.segment
                )
            }
        }
    }

    // === Execution Methods ===

    /// Execute one instruction (tier 1 execution)
    ///
    /// Fetches the opcode at CS:IP, decodes the instruction using tier 1
    /// decoding, and executes it. This is the cold path - no caching.
    ///
    /// Prefix bytes (segment overrides, REP) are handled by setting state
    /// in their handlers. This function loops to consume all prefixes before
    /// executing the actual instruction.
    ///
    /// When halted, the CPU skips instruction execution but still checks for interrupts.
    ///
    /// Returns the number of CPU cycles consumed (placeholder: always 4 for now).
    pub fn step(&mut self, mem: &mut MemoryBus) -> u16 {
        use crate::cpu::tier1::DISPATCH_TABLE;

        // If CPU is halted, skip instruction execution but check for interrupts
        if self.halted {
            self.check_interrupts(mem);
            return 4; // HLT consumes cycles while waiting
        }

        // Reset instruction cycle counter at start of new instruction
        self.current_instruction_cycles = 0;

        // Clear prefix state at start of instruction
        self.segment_override = None;
        self.repeat_prefix = RepeatPrefix::None;

        let cs = self.read_seg(1);

        // Execute instruction, looping while prefix handlers set state
        loop {
            // Remember if we had prefix state set before this instruction
            let had_seg_override = self.segment_override;
            let had_repeat_prefix = self.repeat_prefix;

            // Fetch and execute one opcode
            let opcode = self.read_mem8(mem, cs, self.ip);
            self.ip = self.ip.wrapping_add(1);

            let handler = DISPATCH_TABLE[opcode as usize];
            let instr = self.decode_instruction_t1(mem, opcode, handler);
            instr.execute(self, mem);

            // If a prefix was set by this instruction, continue to fetch next byte
            // Otherwise, we executed the actual instruction and we're done
            if self.segment_override == had_seg_override && self.repeat_prefix == had_repeat_prefix
            {
                break;
            }
        }

        // After instruction execution, check for hardware interrupts
        self.check_interrupts(mem);

        // Accumulate instruction cycles into total cycles
        self.total_cycles += self.current_instruction_cycles as u64;

        // TODO: Return actual cycle count from instruction
        // For now, return placeholder value
        4
    }

    /// Check and handle hardware interrupts from the PIC
    ///
    /// Called at the end of each instruction. If interrupts are enabled (IF=1)
    /// and the PIC has a pending interrupt, this will acknowledge the interrupt
    /// and transfer control to the interrupt handler.
    ///
    /// Interrupts also clear the halt flag, allowing the CPU to resume execution.
    ///
    /// Note: After STI, interrupt recognition is delayed by one instruction
    fn check_interrupts(&mut self, mem: &mut MemoryBus) {
        use crate::cpu::execute::control_flow::enter_interrupt;

        // If interrupt recognition is delayed (after STI), skip this check
        // and clear the delay flag for next instruction
        if self.delay_interrupt {
            self.delay_interrupt = false;
            return;
        }

        // Only process interrupts if the interrupt flag is set
        if !self.get_flag(Self::IF) {
            return;
        }

        // Check if PIC has a pending interrupt
        if !mem.pic().intr_out() {
            return;
        }

        // Clear halt flag - interrupt wakes CPU from halt
        self.halted = false;

        // Acknowledge interrupt and get vector number
        let vector = mem.pic_mut().inta();

        #[cfg(debug_assertions)]
        {
            let cs = self.read_seg(1);
            let ip = self.ip;
            println!(
                "[INT] Hardware interrupt 0x{:02X} (IRQ{}) at {:04X}:{:04X}",
                vector,
                vector.wrapping_sub(0x08),
                cs,
                ip
            );
        }

        // Use common interrupt entry sequence
        enter_interrupt(self, mem, vector);
    }

    /// Set the interrupt delay flag (called by STI)
    ///
    /// This delays hardware interrupt recognition by one instruction,
    /// allowing STI;IRET sequences to work correctly
    pub fn set_interrupt_delay(&mut self) {
        self.delay_interrupt = true;
    }
}
