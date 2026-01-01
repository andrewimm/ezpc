//! Operand type definitions for decoded instructions

/// Types of operands that can appear in 8088 instructions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperandType {
    /// No operand
    None,
    /// 8-bit register (value is register index 0-7)
    Reg8,
    /// 16-bit register (value is register index 0-7)
    Reg16,
    /// Segment register (value is segment index 0-3)
    SegReg,
    /// 8-bit immediate value
    Imm8,
    /// 16-bit immediate value
    Imm16,
    /// 8-bit memory operand (requires addressing mode calculation)
    Mem8,
    /// 16-bit memory operand (requires addressing mode calculation)
    Mem16,
    /// Direct memory address (segment:offset)
    Direct,
    /// Relative offset for jumps
    Rel8,
    /// Relative offset for jumps (16-bit)
    Rel16,
}

/// Decoded operand with type and value
#[derive(Debug, Clone, Copy)]
pub struct Operand {
    /// Type of the operand
    pub op_type: OperandType,
    /// Value of the operand (register index, immediate value, offset, etc.)
    pub value: u16,
    /// For memory operands: segment override (or 0xFF for default)
    pub segment: u8,
    /// For memory operands: displacement value (sign-extended to i16)
    pub disp: i16,
}

impl Operand {
    /// Create a new operand
    pub fn new(op_type: OperandType, value: u16) -> Self {
        Self {
            op_type,
            value,
            segment: 0xFF, // Default: no segment override
            disp: 0,
        }
    }

    /// Create an operand with a segment override
    pub fn with_segment(op_type: OperandType, value: u16, segment: u8) -> Self {
        Self {
            op_type,
            value,
            segment,
            disp: 0,
        }
    }

    /// Set the displacement for a memory operand
    pub fn with_disp(mut self, disp: i16) -> Self {
        self.disp = disp;
        self
    }

    /// Create a "none" operand
    pub fn none() -> Self {
        Self::new(OperandType::None, 0)
    }

    /// Create a register operand (8-bit)
    pub fn reg8(reg: u8) -> Self {
        Self::new(OperandType::Reg8, reg as u16)
    }

    /// Create a register operand (16-bit)
    pub fn reg16(reg: u8) -> Self {
        Self::new(OperandType::Reg16, reg as u16)
    }

    /// Create an immediate operand (8-bit)
    pub fn imm8(value: u8) -> Self {
        Self::new(OperandType::Imm8, value as u16)
    }

    /// Create an immediate operand (16-bit)
    pub fn imm16(value: u16) -> Self {
        Self::new(OperandType::Imm16, value)
    }

    /// Create a memory operand (8-bit)
    pub fn mem8(base_index: u8) -> Self {
        Self::new(OperandType::Mem8, base_index as u16)
    }

    /// Create a memory operand (8-bit) with displacement
    pub fn mem8_disp(base_index: u8, disp: i16) -> Self {
        Self::new(OperandType::Mem8, base_index as u16).with_disp(disp)
    }

    /// Create a memory operand (16-bit)
    pub fn mem16(base_index: u8) -> Self {
        Self::new(OperandType::Mem16, base_index as u16)
    }

    /// Create a memory operand (16-bit) with displacement
    pub fn mem16_disp(base_index: u8, disp: i16) -> Self {
        Self::new(OperandType::Mem16, base_index as u16).with_disp(disp)
    }

    /// Create a relative jump operand (8-bit)
    pub fn rel8(offset: i8) -> Self {
        Self::new(OperandType::Rel8, offset as i16 as u16)
    }

    /// Create a relative jump operand (16-bit)
    pub fn rel16(offset: i16) -> Self {
        Self::new(OperandType::Rel16, offset as u16)
    }

    /// Create a segment register operand
    pub fn seg(seg: u8) -> Self {
        Self::new(OperandType::SegReg, seg as u16)
    }
}
