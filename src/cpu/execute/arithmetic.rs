//! Arithmetic instruction handlers (ADD, SUB, INC, DEC, etc.)

use crate::cpu::Cpu;
use crate::cpu::decode::{DecodedInstruction, OperandType};
use crate::cpu::state::FlagOp;
use crate::memory::MemoryBus;

/// ADD r/m, r - Add register to register/memory
/// Handles both byte (0x00) and word (0x01) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn add_rm_r(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        // For 8-bit add, we need to capture carry in bit 8
        let result = (dst_value as u8) as u32 + (src_value as u8) as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_lazy_flags(result, FlagOp::Add8);
    } else {
        // For 16-bit add, we need to capture carry in bit 16
        let result = dst_value as u32 + src_value as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_lazy_flags(result, FlagOp::Add16);
    }
}

/// ADD r, r/m - Add register/memory to register
/// Handles both byte (0x02) and word (0x03) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn add_r_rm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let result = (dst_value as u8) as u32 + (src_value as u8) as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_lazy_flags(result, FlagOp::Add8);
    } else {
        let result = dst_value as u32 + src_value as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_lazy_flags(result, FlagOp::Add16);
    }
}

/// ADD AL/AX, imm - Add immediate to AL or AX
/// Handles byte (0x04) and word (0x05) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn add_acc_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let result = (dst_value as u8) as u32 + (imm_value as u8) as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_lazy_flags(result, FlagOp::Add8);
    } else {
        let result = dst_value as u32 + imm_value as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_lazy_flags(result, FlagOp::Add16);
    }
}

/// ADD r/m, imm - Add immediate to register/memory
/// Handles byte (0x80 /0, 0x82 /0) and word (0x81 /0, 0x83 /0) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn add_rm_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let result = (dst_value as u8) as u32 + (imm_value as u8) as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_lazy_flags(result, FlagOp::Add8);
    } else {
        let result = dst_value as u32 + imm_value as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_lazy_flags(result, FlagOp::Add16);
    }
}

/// INC r16 - Increment 16-bit register
/// Handles opcodes 0x40-0x47
///
/// Flags affected: PF, AF, ZF, SF, OF (CF is NOT affected)
pub fn inc_r16(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let value = cpu.read_operand(mem, &instr.dst);
    let result = value.wrapping_add(1);
    cpu.write_operand(mem, &instr.dst, result);
    cpu.set_lazy_flags(result as u32, FlagOp::Inc16);
}

/// INC r/m - Increment register/memory
/// Handles byte (0xFE /0) and word (0xFF /0) variants
///
/// Flags affected: PF, AF, ZF, SF, OF (CF is NOT affected)
pub fn inc_rm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let value = cpu.read_operand(mem, &instr.dst);
    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let result = (value as u8).wrapping_add(1);
        cpu.write_operand(mem, &instr.dst, result as u16);
        cpu.set_lazy_flags(result as u32, FlagOp::Inc8);
    } else {
        let result = value.wrapping_add(1);
        cpu.write_operand(mem, &instr.dst, result);
        cpu.set_lazy_flags(result as u32, FlagOp::Inc16);
    }
}

/// DEC r16 - Decrement 16-bit register
/// Handles opcodes 0x48-0x4F
///
/// Flags affected: PF, AF, ZF, SF, OF (CF is NOT affected)
pub fn dec_r16(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let value = cpu.read_operand(mem, &instr.dst);
    let result = value.wrapping_sub(1);
    cpu.write_operand(mem, &instr.dst, result);
    cpu.set_lazy_flags(result as u32, FlagOp::Dec16);
}

/// DEC r/m - Decrement register/memory
/// Handles byte (0xFE /1) and word (0xFF /1) variants
///
/// Flags affected: PF, AF, ZF, SF, OF (CF is NOT affected)
pub fn dec_rm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let value = cpu.read_operand(mem, &instr.dst);
    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let result = (value as u8).wrapping_sub(1);
        cpu.write_operand(mem, &instr.dst, result as u16);
        cpu.set_lazy_flags(result as u32, FlagOp::Dec8);
    } else {
        let result = value.wrapping_sub(1);
        cpu.write_operand(mem, &instr.dst, result);
        cpu.set_lazy_flags(result as u32, FlagOp::Dec16);
    }
}
