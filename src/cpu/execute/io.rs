//! IO instruction handlers (IN/OUT)
//!
//! Cycle timing is handled by BASE_CYCLES table in timing.rs.

use crate::cpu::decode::DecodedInstruction;
use crate::cpu::Cpu;
use crate::memory::MemoryBus;

/// IN AL, imm8 - Read byte from immediate port to AL
/// Opcode: 0xE4 (10 cycles)
pub fn in_al_imm8(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let port = instr.src.value as u16;
    let value = mem.io_read_u8(port);
    cpu.write_reg8(0, value as u8); // AL = reg 0
}

/// IN AX, imm8 - Read word from immediate port to AX
/// Opcode: 0xE5 (14 cycles)
pub fn in_ax_imm8(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let port = instr.src.value as u16;
    let value = mem.io_read_u16(port);
    cpu.write_reg16(0, value); // AX = reg 0
}

/// OUT imm8, AL - Write AL to immediate port
/// Opcode: 0xE6 (10 cycles)
pub fn out_imm8_al(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let port = instr.dst.value as u16;
    let value = cpu.read_reg8(0); // AL = reg 0
    mem.io_write_u8(port, value);
}

/// OUT imm8, AX - Write AX to immediate port
/// Opcode: 0xE7 (14 cycles)
pub fn out_imm8_ax(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let port = instr.dst.value as u16;
    let value = cpu.read_reg16(0); // AX = reg 0
    mem.io_write_u16(port, value);
}

/// IN AL, DX - Read byte from DX port to AL
/// Opcode: 0xEC (8 cycles)
pub fn in_al_dx(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let port = cpu.read_reg16(2); // DX = reg 2
    let value = mem.io_read_u8(port);
    cpu.write_reg8(0, value as u8); // AL = reg 0
}

/// IN AX, DX - Read word from DX port to AX
/// Opcode: 0xED (12 cycles)
pub fn in_ax_dx(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let port = cpu.read_reg16(2); // DX = reg 2
    let value = mem.io_read_u16(port);
    cpu.write_reg16(0, value); // AX = reg 0
}

/// OUT DX, AL - Write AL to DX port
/// Opcode: 0xEE (8 cycles)
pub fn out_dx_al(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let port = cpu.read_reg16(2); // DX = reg 2
    let value = cpu.read_reg8(0); // AL = reg 0
    mem.io_write_u8(port, value);
}

/// OUT DX, AX - Write AX to DX port
/// Opcode: 0xEF (12 cycles)
pub fn out_dx_ax(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let port = cpu.read_reg16(2); // DX = reg 2
    let value = cpu.read_reg16(0); // AX = reg 0
    mem.io_write_u16(port, value);
}
