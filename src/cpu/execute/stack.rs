//! Stack operation handlers (PUSH, POP, etc.)

use crate::cpu::Cpu;
use crate::cpu::decode::{DecodedInstruction, Operand, OperandType};
use crate::memory::MemoryBus;
