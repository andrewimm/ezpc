//! Decoded instruction representation for caching

use super::operands::{Operand, OperandType};
use crate::cpu::Cpu;
use crate::memory::MemoryBus;

/// Instruction handler function signature
/// Takes mutable CPU state, memory bus, and the decoded instruction
/// Returns the number of cycles consumed (or unit for now)
pub type InstructionHandler = fn(&mut Cpu, &mut MemoryBus, &DecodedInstruction);

/// A fully decoded instruction ready for execution
///
/// This struct caches all information needed to execute an instruction
/// without re-decoding it. Used by tier 2 (decode cache) and tier 3
/// (basic block compilation).
#[derive(Clone)]
pub struct DecodedInstruction {
    /// The opcode byte
    pub opcode: u8,

    /// First operand (destination for most instructions)
    pub dst: Operand,

    /// Second operand (source for most instructions)
    pub src: Operand,

    /// Total length of the instruction in bytes (including opcode, modrm, displacement, immediate)
    pub length: u8,

    /// Function pointer to the instruction handler
    pub handler: InstructionHandler,

    /// Base cycles for this instruction (from timing table)
    pub base_cycles: u8,

    /// EA (Effective Address) calculation cycles for memory operands
    pub ea_cycles: u8,
}

impl DecodedInstruction {
    /// Create a new decoded instruction
    pub fn new(opcode: u8, handler: InstructionHandler) -> Self {
        Self {
            opcode,
            dst: Operand::none(),
            src: Operand::none(),
            length: 1, // Minimum length is 1 (just the opcode)
            handler,
            base_cycles: 0,
            ea_cycles: 0,
        }
    }

    /// Set the destination operand
    pub fn with_dst(mut self, dst: Operand) -> Self {
        self.dst = dst;
        self
    }

    /// Set the source operand
    pub fn with_src(mut self, src: Operand) -> Self {
        self.src = src;
        self
    }

    /// Set the instruction length
    pub fn with_length(mut self, length: u8) -> Self {
        self.length = length;
        self
    }

    /// Set the base cycles for this instruction
    pub fn with_base_cycles(mut self, cycles: u8) -> Self {
        self.base_cycles = cycles;
        self
    }

    /// Set the EA calculation cycles for this instruction
    pub fn with_ea_cycles(mut self, cycles: u8) -> Self {
        self.ea_cycles = cycles;
        self
    }

    /// Set both base and EA cycles
    pub fn with_timing(mut self, base_cycles: u8, ea_cycles: u8) -> Self {
        self.base_cycles = base_cycles;
        self.ea_cycles = ea_cycles;
        self
    }

    /// Get total cycles for this instruction (base + EA)
    ///
    /// Note: This doesn't include transfer penalties or segment override costs,
    /// which must be added by the handler or step() function.
    #[inline(always)]
    pub fn total_cycles(&self) -> u8 {
        self.base_cycles.saturating_add(self.ea_cycles)
    }

    /// Check if instruction has a destination operand
    pub fn has_dst(&self) -> bool {
        self.dst.op_type != OperandType::None
    }

    /// Check if instruction has a source operand
    pub fn has_src(&self) -> bool {
        self.src.op_type != OperandType::None
    }

    /// Execute the instruction
    #[inline(always)]
    pub fn execute(&self, cpu: &mut Cpu, mem: &mut MemoryBus) {
        (self.handler)(cpu, mem, self);
    }
}

// Manual Debug implementation since function pointers don't implement Debug
impl std::fmt::Debug for DecodedInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecodedInstruction")
            .field("opcode", &format_args!("{:#04x}", self.opcode))
            .field("dst", &self.dst)
            .field("src", &self.src)
            .field("length", &self.length)
            .field("handler", &"<fn>")
            .field("base_cycles", &self.base_cycles)
            .field("ea_cycles", &self.ea_cycles)
            .finish()
    }
}
