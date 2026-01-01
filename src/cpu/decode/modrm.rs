//! ModR/M byte decoder for 8088 addressing modes
//!
//! The ModR/M byte encodes addressing modes and register operands.
//! Format: [mod:2][reg:3][r/m:3]
//! - mod: addressing mode (00, 01, 10, 11)
//! - reg: register operand (or opcode extension for some instructions)
//! - r/m: register or memory operand

/// Addressing mode variants decoded from ModR/M byte
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddressingMode {
    /// Register-to-register mode (mod=11)
    /// The r/m field specifies a register
    RegisterDirect { rm_reg: u8 },

    /// Memory indirect mode (mod=00)
    /// Address is calculated from base/index registers
    /// No displacement (except special case [r/m=110] which is direct address)
    MemoryIndirect { base_index: u8 },

    /// Memory with 8-bit displacement (mod=01)
    /// Address = base/index + sign-extended 8-bit displacement
    MemoryDisp8 { base_index: u8, disp: i8 },

    /// Memory with 16-bit displacement (mod=10)
    /// Address = base/index + 16-bit displacement
    MemoryDisp16 { base_index: u8, disp: i16 },

    /// Direct memory address (mod=00, r/m=110)
    /// 16-bit address follows the ModR/M byte
    DirectAddress { addr: u16 },
}

/// Decoded ModR/M byte
#[derive(Debug, Clone, Copy)]
pub struct ModRM {
    /// The raw ModR/M byte
    pub raw: u8,

    /// Mod field (bits 7-6)
    pub mod_bits: u8,

    /// Reg field (bits 5-3) - register or opcode extension
    pub reg: u8,

    /// R/M field (bits 2-0) - register or memory operand
    pub rm: u8,

    /// Decoded addressing mode
    pub mode: AddressingMode,
}

impl ModRM {
    /// Decode a ModR/M byte (without displacement/address - those come from instruction stream)
    ///
    /// This method only decodes the ModR/M byte itself. The displacement or direct
    /// address must be read separately based on the addressing mode.
    pub fn decode(modrm: u8) -> Self {
        let mod_bits = (modrm >> 6) & 0b11;
        let reg = (modrm >> 3) & 0b111;
        let rm = modrm & 0b111;

        // Determine addressing mode from mod and r/m fields
        let mode = match mod_bits {
            0b11 => {
                // Register direct mode
                AddressingMode::RegisterDirect { rm_reg: rm }
            }
            0b00 => {
                if rm == 0b110 {
                    // Special case: direct address (will be read from instruction stream)
                    AddressingMode::DirectAddress { addr: 0 }
                } else {
                    // Memory indirect, no displacement
                    AddressingMode::MemoryIndirect { base_index: rm }
                }
            }
            0b01 => {
                // Memory with 8-bit displacement (will be read from instruction stream)
                AddressingMode::MemoryDisp8 {
                    base_index: rm,
                    disp: 0,
                }
            }
            0b10 => {
                // Memory with 16-bit displacement (will be read from instruction stream)
                AddressingMode::MemoryDisp16 {
                    base_index: rm,
                    disp: 0,
                }
            }
            _ => unreachable!(),
        };

        Self {
            raw: modrm,
            mod_bits,
            reg,
            rm,
            mode,
        }
    }

    /// Update the addressing mode with a displacement or address value
    /// This should be called after reading displacement/address bytes from the instruction stream
    pub fn with_disp8(mut self, disp: i8) -> Self {
        if let AddressingMode::MemoryDisp8 { base_index, .. } = self.mode {
            self.mode = AddressingMode::MemoryDisp8 { base_index, disp };
        }
        self
    }

    /// Update the addressing mode with a 16-bit displacement
    pub fn with_disp16(mut self, disp: i16) -> Self {
        if let AddressingMode::MemoryDisp16 { base_index, .. } = self.mode {
            self.mode = AddressingMode::MemoryDisp16 { base_index, disp };
        }
        self
    }

    /// Update the addressing mode with a direct address
    pub fn with_direct_addr(mut self, addr: u16) -> Self {
        if let AddressingMode::DirectAddress { .. } = self.mode {
            self.mode = AddressingMode::DirectAddress { addr };
        }
        self
    }

    /// Calculate the effective address for memory operands
    /// Returns (segment_index, offset)
    /// - segment_index: default segment to use (0=ES, 1=CS, 2=SS, 3=DS)
    /// - offset: the calculated offset
    pub fn calculate_address(&self, cpu: &crate::cpu::Cpu) -> (u8, u16) {
        match self.mode {
            AddressingMode::RegisterDirect { .. } => {
                panic!("calculate_address called on register direct mode");
            }
            AddressingMode::DirectAddress { addr } => {
                // Direct address uses DS segment by default
                (3, addr)
            }
            AddressingMode::MemoryIndirect { base_index }
            | AddressingMode::MemoryDisp8 { base_index, .. }
            | AddressingMode::MemoryDisp16 { base_index, .. } => {
                // Calculate effective address from base/index registers
                let (default_seg, offset) = self.calculate_ea(cpu, base_index);
                (default_seg, offset)
            }
        }
    }

    /// Calculate effective address from base/index encoding
    /// Returns (default_segment_index, offset)
    fn calculate_ea(&self, cpu: &crate::cpu::Cpu, base_index: u8) -> (u8, u16) {
        // Base displacement
        let disp = match self.mode {
            AddressingMode::MemoryDisp8 { disp, .. } => disp as i16 as u16,
            AddressingMode::MemoryDisp16 { disp, .. } => disp as u16,
            _ => 0,
        };

        // Effective address calculation based on r/m field
        // See Intel 8086 manual Table 2-2
        match base_index {
            0b000 => {
                // [BX + SI]
                let ea = cpu
                    .read_reg16(3)
                    .wrapping_add(cpu.read_reg16(6))
                    .wrapping_add(disp);
                (3, ea) // DS default
            }
            0b001 => {
                // [BX + DI]
                let ea = cpu
                    .read_reg16(3)
                    .wrapping_add(cpu.read_reg16(7))
                    .wrapping_add(disp);
                (3, ea) // DS default
            }
            0b010 => {
                // [BP + SI]
                let ea = cpu
                    .read_reg16(5)
                    .wrapping_add(cpu.read_reg16(6))
                    .wrapping_add(disp);
                (2, ea) // SS default
            }
            0b011 => {
                // [BP + DI]
                let ea = cpu
                    .read_reg16(5)
                    .wrapping_add(cpu.read_reg16(7))
                    .wrapping_add(disp);
                (2, ea) // SS default
            }
            0b100 => {
                // [SI]
                let ea = cpu.read_reg16(6).wrapping_add(disp);
                (3, ea) // DS default
            }
            0b101 => {
                // [DI]
                let ea = cpu.read_reg16(7).wrapping_add(disp);
                (3, ea) // DS default
            }
            0b110 => {
                // [BP] (or direct address if mod=00, but that's handled separately)
                let ea = cpu.read_reg16(5).wrapping_add(disp);
                (2, ea) // SS default
            }
            0b111 => {
                // [BX]
                let ea = cpu.read_reg16(3).wrapping_add(disp);
                (3, ea) // DS default
            }
            _ => unreachable!(),
        }
    }

    /// Check if this is register-direct addressing
    pub fn is_register_direct(&self) -> bool {
        matches!(self.mode, AddressingMode::RegisterDirect { .. })
    }

    /// Get the register number for register-direct mode
    pub fn get_rm_register(&self) -> u8 {
        if let AddressingMode::RegisterDirect { rm_reg } = self.mode {
            rm_reg
        } else {
            panic!("get_rm_register called on non-register mode");
        }
    }
}
