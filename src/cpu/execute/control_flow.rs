//! Control flow instruction handlers (JMP, CALL, RET, Jcc, etc.)

use crate::cpu::Cpu;
use crate::cpu::decode::DecodedInstruction;
use crate::memory::MemoryBus;

/// JMP short - Jump with 8-bit relative offset
/// Opcode: 0xEB
///
/// IP = IP + rel8 (sign-extended)
pub fn jmp_short(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    // The offset is stored as a sign-extended value in operand.value
    let offset = instr.src.value as i16;
    cpu.ip = cpu.ip.wrapping_add(offset as u16);
}

/// JMP near - Jump with 16-bit relative offset
/// Opcode: 0xE9
///
/// IP = IP + rel16
pub fn jmp_near(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let offset = instr.src.value as i16;
    cpu.ip = cpu.ip.wrapping_add(offset as u16);
}

/// JZ/JE - Jump if zero/equal
/// Opcode: 0x74
///
/// If ZF=1 then IP = IP + rel8
pub fn jz(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if cpu.get_flag(Cpu::ZF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// JNZ/JNE - Jump if not zero/not equal
/// Opcode: 0x75
///
/// If ZF=0 then IP = IP + rel8
pub fn jnz(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if !cpu.get_flag(Cpu::ZF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// JS - Jump if sign
/// Opcode: 0x78
///
/// If SF=1 then IP = IP + rel8
pub fn js(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if cpu.get_flag(Cpu::SF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// JNS - Jump if not sign
/// Opcode: 0x79
///
/// If SF=0 then IP = IP + rel8
pub fn jns(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if !cpu.get_flag(Cpu::SF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// JC/JB/JNAE - Jump if carry/below/not above or equal
/// Opcode: 0x72
///
/// If CF=1 then IP = IP + rel8
pub fn jc(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if cpu.get_flag(Cpu::CF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// JNC/JAE/JNB - Jump if not carry/above or equal/not below
/// Opcode: 0x73
///
/// If CF=0 then IP = IP + rel8
pub fn jnc(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if !cpu.get_flag(Cpu::CF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}
