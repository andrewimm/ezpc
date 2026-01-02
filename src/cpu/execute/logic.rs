//! Logical operation handlers (AND, OR, XOR, NOT, etc.)

use crate::cpu::Cpu;
use crate::cpu::decode::{DecodedInstruction, OperandType};
use crate::cpu::state::FlagOp;
use crate::memory::MemoryBus;

/// AND r/m, r - Logical AND register with register/memory
/// Handles both byte (0x20) and word (0x21) variants
///
/// Flags affected: CF=0, OF=0, SF, ZF, PF (AF undefined)
pub fn and_rm_r(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let result = (dst_value as u8) & (src_value as u8);
        cpu.write_operand(mem, &instr.dst, result as u16);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::And8);
    } else {
        let result = dst_value & src_value;
        cpu.write_operand(mem, &instr.dst, result);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::And16);
    }
}

/// AND r, r/m - Logical AND register/memory with register
/// Handles both byte (0x22) and word (0x23) variants
///
/// Flags affected: CF=0, OF=0, SF, ZF, PF (AF undefined)
pub fn and_r_rm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let result = (dst_value as u8) & (src_value as u8);
        cpu.write_operand(mem, &instr.dst, result as u16);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::And8);
    } else {
        let result = dst_value & src_value;
        cpu.write_operand(mem, &instr.dst, result);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::And16);
    }
}

/// AND AL/AX, imm - Logical AND immediate with AL or AX
/// Handles byte (0x24) and word (0x25) variants
///
/// Flags affected: CF=0, OF=0, SF, ZF, PF (AF undefined)
pub fn and_acc_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let result = (dst_value as u8) & (imm_value as u8);
        cpu.write_operand(mem, &instr.dst, result as u16);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::And8);
    } else {
        let result = dst_value & imm_value;
        cpu.write_operand(mem, &instr.dst, result);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::And16);
    }
}
