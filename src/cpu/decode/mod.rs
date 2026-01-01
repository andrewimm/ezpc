//! Instruction decoding module
//!
//! This module handles decoding of 8088 instructions, including:
//! - ModR/M byte parsing
//! - Operand decoding
//! - Instruction caching for tier 2 execution

pub mod instruction;
pub mod modrm;
pub mod operands;

pub use instruction::DecodedInstruction;
pub use modrm::{AddressingMode, ModRM};
pub use operands::{Operand, OperandType};
