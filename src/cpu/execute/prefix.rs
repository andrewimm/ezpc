//! Prefix instruction handlers
//!
//! Prefix bytes modify the behavior of the following instruction:
//! - Segment override prefixes (ES:, CS:, SS:, DS:)
//! - Repeat prefixes (REP, REPNE)

use crate::cpu::Cpu;
use crate::cpu::decode::DecodedInstruction;
use crate::cpu::state::RepeatPrefix;
use crate::memory::MemoryBus;

/// ES: segment override prefix (0x26)
pub fn seg_es(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    cpu.segment_override = Some(0);
}

/// CS: segment override prefix (0x2E)
pub fn seg_cs(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    cpu.segment_override = Some(1);
}

/// SS: segment override prefix (0x36)
pub fn seg_ss(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    cpu.segment_override = Some(2);
}

/// DS: segment override prefix (0x3E)
pub fn seg_ds(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    cpu.segment_override = Some(3);
}

/// REPNE/REPNZ prefix (0xF2)
///
/// Repeats the following string instruction while CX != 0 and ZF == 0.
/// For CMPS/SCAS: repeat while not equal.
pub fn repne(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    cpu.repeat_prefix = RepeatPrefix::RepNe;
    // Save IP of the prefix byte for looping back
    cpu.repeat_ip = cpu.ip.wrapping_sub(1);
}

/// REP/REPE/REPZ prefix (0xF3)
///
/// Repeats the following string instruction while CX != 0.
/// For CMPS/SCAS: repeat while equal (ZF == 1).
pub fn rep(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    cpu.repeat_prefix = RepeatPrefix::Rep;
    // Save IP of the prefix byte for looping back
    cpu.repeat_ip = cpu.ip.wrapping_sub(1);
}
