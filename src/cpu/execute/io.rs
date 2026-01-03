//! IO instruction handlers (IN/OUT)

use crate::cpu::Cpu;
use crate::cpu::decode::DecodedInstruction;
use crate::memory::MemoryBus;

/// IN AL, imm8 - Read byte from immediate port to AL
/// Opcode: 0xE4
pub fn in_al_imm8(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let port = instr.src.value as u16;
    let value = mem.io_read_u8(port);
    cpu.write_reg8(0, value as u8); // AL = reg 0
    cpu.current_instruction_cycles += 10;
}

/// IN AX, imm8 - Read word from immediate port to AX
/// Opcode: 0xE5
pub fn in_ax_imm8(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let port = instr.src.value as u16;
    let value = mem.io_read_u16(port);
    cpu.write_reg16(0, value); // AX = reg 0
    cpu.current_instruction_cycles += 14;
}

/// OUT imm8, AL - Write AL to immediate port
/// Opcode: 0xE6
pub fn out_imm8_al(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let port = instr.dst.value as u16;
    let value = cpu.read_reg8(0); // AL = reg 0
    mem.io_write_u8(port, value);
    cpu.current_instruction_cycles += 10;
}

/// OUT imm8, AX - Write AX to immediate port
/// Opcode: 0xE7
pub fn out_imm8_ax(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let port = instr.dst.value as u16;
    let value = cpu.read_reg16(0); // AX = reg 0
    mem.io_write_u16(port, value);
    cpu.current_instruction_cycles += 14;
}

/// IN AL, DX - Read byte from DX port to AL
/// Opcode: 0xEC
pub fn in_al_dx(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let port = cpu.read_reg16(3); // DX = reg 3
    let value = mem.io_read_u8(port);
    cpu.write_reg8(0, value as u8); // AL = reg 0
    cpu.current_instruction_cycles += 8;
}

/// IN AX, DX - Read word from DX port to AX
/// Opcode: 0xED
pub fn in_ax_dx(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let port = cpu.read_reg16(3); // DX = reg 3
    let value = mem.io_read_u16(port);
    cpu.write_reg16(0, value); // AX = reg 0
    cpu.current_instruction_cycles += 12;
}

/// OUT DX, AL - Write AL to DX port
/// Opcode: 0xEE
pub fn out_dx_al(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let port = cpu.read_reg16(3); // DX = reg 3
    let value = cpu.read_reg8(0); // AL = reg 0
    mem.io_write_u8(port, value);
    cpu.current_instruction_cycles += 8;
}

/// OUT DX, AX - Write AX to DX port
/// Opcode: 0xEF
pub fn out_dx_ax(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let port = cpu.read_reg16(3); // DX = reg 3
    let value = cpu.read_reg16(0); // AX = reg 0
    mem.io_write_u16(port, value);
    cpu.current_instruction_cycles += 12;
}
