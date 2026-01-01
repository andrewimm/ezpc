//! Basic instruction handlers and handler utilities

use crate::cpu::Cpu;
use crate::cpu::decode::DecodedInstruction;
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
