//! Control flow instruction handlers (JMP, CALL, RET, Jcc, etc.)

use crate::cpu::Cpu;
use crate::cpu::decode::{DecodedInstruction, Operand, OperandType};
use crate::memory::MemoryBus;
