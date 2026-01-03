//! Flag manipulation instruction handlers
//!
//! This module contains handlers for instructions that directly manipulate CPU flags:
//! - CLC/STC: Clear/Set Carry Flag
//! - CLI/STI: Clear/Set Interrupt Flag
//! - CLD/STD: Clear/Set Direction Flag

use crate::cpu::Cpu;
use crate::cpu::decode::DecodedInstruction;
use crate::memory::MemoryBus;

/// Handler for CLC (0xF8) - Clear Carry Flag
///
/// Clears the carry flag (CF) to 0.
/// Takes 2 cycles on the 8088.
#[inline(always)]
pub fn clc(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    cpu.set_flag(Cpu::CF, false);
}

/// Handler for STC (0xF9) - Set Carry Flag
///
/// Sets the carry flag (CF) to 1.
/// Takes 2 cycles on the 8088.
#[inline(always)]
pub fn stc(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    cpu.set_flag(Cpu::CF, true);
}

/// Handler for CLI (0xFA) - Clear Interrupt Flag
///
/// Clears the interrupt enable flag (IF) to 0.
/// When IF is cleared, maskable hardware interrupts are disabled.
/// Takes 2 cycles on the 8088.
#[inline(always)]
pub fn cli(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    cpu.set_flag(Cpu::IF, false);
}

/// Handler for STI (0xFB) - Set Interrupt Flag
///
/// Sets the interrupt enable flag (IF) to 1.
/// When IF is set, maskable hardware interrupts are enabled.
/// Takes 2 cycles on the 8088.
#[inline(always)]
pub fn sti(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    cpu.set_flag(Cpu::IF, true);
}

/// Handler for CLD (0xFC) - Clear Direction Flag
///
/// Clears the direction flag (DF) to 0.
/// When DF is 0, string operations increment SI/DI (forward direction).
/// Takes 2 cycles on the 8088.
#[inline(always)]
pub fn cld(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    cpu.set_flag(Cpu::DF, false);
}

/// Handler for STD (0xFD) - Set Direction Flag
///
/// Sets the direction flag (DF) to 1.
/// When DF is 1, string operations decrement SI/DI (backward direction).
/// Takes 2 cycles on the 8088.
#[inline(always)]
pub fn std(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    cpu.set_flag(Cpu::DF, true);
}
