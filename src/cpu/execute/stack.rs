//! Stack operation handlers (PUSH, POP, etc.)

use crate::cpu::Cpu;
use crate::cpu::decode::DecodedInstruction;
use crate::memory::MemoryBus;

/// PUSH r16 - Push 16-bit register onto stack
/// Handles opcodes 0x50-0x57
///
/// Stack operation: SP -= 2, [SS:SP] = operand
pub fn push_r16(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let value = cpu.read_operand(mem, &instr.src);
    push_word(cpu, mem, value);
}

/// PUSH r/m16 - Push 16-bit register/memory onto stack
/// Part of opcode 0xFF (reg field = 110)
///
/// Stack operation: SP -= 2, [SS:SP] = operand
pub fn push_rm16(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let value = cpu.read_operand(mem, &instr.src);
    push_word(cpu, mem, value);
}

/// POP r16 - Pop 16-bit value from stack into register
/// Handles opcodes 0x58-0x5F
///
/// Stack operation: operand = [SS:SP], SP += 2
pub fn pop_r16(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let value = pop_word(cpu, mem);
    cpu.write_operand(mem, &instr.dst, value);
}

/// POP r/m16 - Pop 16-bit value from stack into register/memory
/// Part of opcode 0x8F (reg field = 000)
///
/// Stack operation: operand = [SS:SP], SP += 2
pub fn pop_rm16(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let value = pop_word(cpu, mem);
    cpu.write_operand(mem, &instr.dst, value);
}

/// Helper: Push a 16-bit value onto the stack
/// The stack grows downward (SP decrements before write)
#[inline(always)]
pub fn push_word(cpu: &mut Cpu, mem: &mut MemoryBus, value: u16) {
    let sp = cpu.read_reg16(4).wrapping_sub(2); // SP -= 2
    cpu.write_reg16(4, sp);
    let ss = cpu.read_seg(2); // SS
    cpu.write_mem16(mem, ss, sp, value);
}

/// Helper: Pop a 16-bit value from the stack
/// The stack grows downward (SP increments after read)
#[inline(always)]
pub fn pop_word(cpu: &mut Cpu, mem: &MemoryBus) -> u16 {
    let sp = cpu.read_reg16(4); // SP
    let ss = cpu.read_seg(2); // SS
    let value = cpu.read_mem16(mem, ss, sp);
    cpu.write_reg16(4, sp.wrapping_add(2)); // SP += 2
    value
}
