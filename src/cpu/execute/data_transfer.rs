//! Data transfer instruction handlers (MOV, XCHG, etc.)

use crate::cpu::Cpu;
use crate::cpu::decode::{DecodedInstruction, Operand, OperandType};
use crate::memory::MemoryBus;
