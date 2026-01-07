//! Flag manipulation instruction handlers
//!
//! This module contains handlers for instructions that directly manipulate CPU flags:
//! - CLC/STC: Clear/Set Carry Flag
//! - CLI/STI: Clear/Set Interrupt Flag
//! - CLD/STD: Clear/Set Direction Flag

use crate::cpu::decode::DecodedInstruction;
use crate::cpu::Cpu;
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

/// Handler for CMC (0xF5) - Complement Carry Flag
///
/// Complements (inverts) the carry flag (CF).
/// If CF is 0, it becomes 1. If CF is 1, it becomes 0.
/// Takes 2 cycles on the 8088.
#[inline(always)]
pub fn cmc(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let current_cf = cpu.get_flag(Cpu::CF);
    cpu.set_flag(Cpu::CF, !current_cf);
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
///
/// Note: On the 8088, interrupt recognition is delayed by one instruction
/// after STI to allow STI;IRET sequences to execute before taking an interrupt.
/// This delay only applies when actually transitioning from IF=0 to IF=1.
#[inline(always)]
pub fn sti(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    // Only set delay if interrupts were previously disabled
    let was_disabled = !cpu.get_flag(Cpu::IF);
    cpu.set_flag(Cpu::IF, true);
    if was_disabled {
        cpu.set_interrupt_delay();
    }
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

/// Handler for PUSHF (0x9C) - Push FLAGS register onto stack
///
/// Pushes the FLAGS register onto the stack.
/// Stack operation: SP -= 2, [SS:SP] = FLAGS
/// Takes 10 cycles on the 8088.
#[inline(always)]
pub fn pushf(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let flags = cpu.get_flags();
    crate::cpu::execute::stack::push_word(cpu, mem, flags);
}

/// Handler for POPF (0x9D) - Pop FLAGS register from stack
///
/// Pops FLAGS register from the stack.
/// Stack operation: FLAGS = [SS:SP], SP += 2
/// Takes 8 cycles on the 8088.
#[inline(always)]
pub fn popf(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let flags = crate::cpu::execute::stack::pop_word(cpu, mem);
    cpu.set_flags(flags);
}

/// Handler for SAHF (0x9E) - Store AH into Flags
///
/// Copies AH register into the low byte of FLAGS.
/// Affects: SF, ZF, AF, PF, CF (bits 7, 6, 4, 2, 0 of FLAGS)
/// Does not affect: OF, DF, IF, TF (bits 11, 10, 9, 8)
/// Takes 4 cycles on the 8088.
#[inline(always)]
pub fn sahf(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let ah = cpu.read_reg8(4); // AH
    let current_flags = cpu.get_flags();

    // Clear SF, ZF, AF, PF, CF (bits 7, 6, 4, 2, 0) and bit 1 (always set)
    let high_flags = current_flags & 0xFF00;

    // Set new low byte from AH, ensuring bit 1 is always set
    let new_flags = high_flags | (ah as u16) | 0x0002;
    cpu.set_flags(new_flags);
}

/// Handler for LAHF (0x9F) - Load AH from Flags
///
/// Copies the low byte of FLAGS into AH register.
/// Loads: SF, ZF, AF, PF, CF (bits 7, 6, 4, 2, 0 of FLAGS)
/// Takes 4 cycles on the 8088.
#[inline(always)]
pub fn lahf(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let flags = cpu.get_flags();
    let low_byte = (flags & 0xFF) as u8;
    cpu.write_reg8(4, low_byte); // AH
}
