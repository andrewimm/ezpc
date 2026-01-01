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
    /// General purpose registers: AX, CX, DX, BX, SP, BP, SI, DI (indices 0-7)
    /// Stored as u16 for word access. Byte access uses low/high byte extraction.
    pub regs: [u16; 8],

    /// Segment registers: ES, CS, SS, DS (indices 0-3)
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
}

/// Operation type for lazy flag evaluation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlagOp {
    None,
    Add8,
    Add16,
    Sub8,
    Sub16,
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
    fn compute_flags(&self) -> u16 {
        let mut flags = self.flags & 0b0010; // Keep bit 1 (always set)

        match self.last_op {
            FlagOp::None => return self.flags,

            FlagOp::Add8
            | FlagOp::Sub8
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
                if matches!(self.last_op, FlagOp::Add8 | FlagOp::Sub8) {
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
            | FlagOp::Sub16
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
                if matches!(self.last_op, FlagOp::Add16 | FlagOp::Sub16) {
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
}
