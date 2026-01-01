//! Arithmetic instruction handlers (ADD, SUB, INC, DEC, etc.)

use crate::cpu::Cpu;
use crate::cpu::decode::{DecodedInstruction, Operand, OperandType};
use crate::cpu::state::FlagOp;
use crate::memory::MemoryBus;
