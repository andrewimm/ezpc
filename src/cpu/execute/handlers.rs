//! Basic instruction handlers and handler utilities

use crate::cpu::decode::DecodedInstruction;
use crate::cpu::Cpu;
use crate::memory::MemoryBus;

/// Handler for invalid/unimplemented opcodes
///
/// This handler is called when an unknown or unimplemented opcode is
/// encountered. It panics with information about the opcode and CPU state.
pub fn invalid_opcode(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    panic!(
        "Invalid opcode: {:#04x} at CS:IP = {:04X}:{:04X}",
        instr.opcode,
        cpu.read_seg(1),        // CS
        cpu.ip.wrapping_sub(1)  // IP was already advanced
    );
}

/// Handler for NOP (0x90) - No operation
///
/// Does nothing. The NOP instruction takes 3 cycles on the 8088.
pub fn nop(_cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    // Do nothing
}

/// Handler for HLT (0xF4) - Halt
///
/// Halts the CPU until an interrupt occurs. When halted, the CPU stops
/// executing instructions but continues to check for interrupts. An interrupt
/// will clear the halt flag and resume execution.
///
/// Note: On real hardware, HLT can only be executed in privileged mode.
/// This implementation doesn't enforce privilege levels yet.
pub fn hlt(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    cpu.halted = true;
}
