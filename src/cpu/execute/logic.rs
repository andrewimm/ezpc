//! Logical operation handlers (AND, OR, XOR, NOT, etc.)

use crate::cpu::decode::{DecodedInstruction, OperandType};
use crate::cpu::state::FlagOp;
use crate::cpu::Cpu;
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

/// AND r/m, imm - Logical AND immediate with register/memory
/// Handles byte (0x80 /4, 0x82 /4) and word (0x81 /4, 0x83 /4) variants
///
/// For 0x83, the immediate byte is sign-extended to word size.
///
/// Flags affected: CF=0, OF=0, SF, ZF, PF (AF undefined)
pub fn and_rm_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

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

/// OR r/m, r - Logical OR register with register/memory
/// Handles both byte (0x08) and word (0x09) variants
///
/// Flags affected: CF=0, OF=0, SF, ZF, PF (AF undefined)
pub fn or_rm_r(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let result = (dst_value as u8) | (src_value as u8);
        cpu.write_operand(mem, &instr.dst, result as u16);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Or8);
    } else {
        let result = dst_value | src_value;
        cpu.write_operand(mem, &instr.dst, result);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Or16);
    }
}

/// OR r, r/m - Logical OR register/memory with register
/// Handles both byte (0x0A) and word (0x0B) variants
///
/// Flags affected: CF=0, OF=0, SF, ZF, PF (AF undefined)
pub fn or_r_rm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let result = (dst_value as u8) | (src_value as u8);
        cpu.write_operand(mem, &instr.dst, result as u16);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Or8);
    } else {
        let result = dst_value | src_value;
        cpu.write_operand(mem, &instr.dst, result);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Or16);
    }
}

/// OR AL/AX, imm - Logical OR immediate with AL or AX
/// Handles byte (0x0C) and word (0x0D) variants
///
/// Flags affected: CF=0, OF=0, SF, ZF, PF (AF undefined)
pub fn or_acc_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let result = (dst_value as u8) | (imm_value as u8);
        cpu.write_operand(mem, &instr.dst, result as u16);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Or8);
    } else {
        let result = dst_value | imm_value;
        cpu.write_operand(mem, &instr.dst, result);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Or16);
    }
}

/// OR r/m, imm - Logical OR immediate with register/memory
/// Handles byte (0x80 /1, 0x82 /1) and word (0x81 /1, 0x83 /1) variants
///
/// For 0x83, the immediate byte is sign-extended to word size.
///
/// Flags affected: CF=0, OF=0, SF, ZF, PF (AF undefined)
pub fn or_rm_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let result = (dst_value as u8) | (imm_value as u8);
        cpu.write_operand(mem, &instr.dst, result as u16);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Or8);
    } else {
        let result = dst_value | imm_value;
        cpu.write_operand(mem, &instr.dst, result);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Or16);
    }
}

/// XOR r/m, r - Logical XOR register with register/memory
/// Handles both byte (0x30) and word (0x31) variants
///
/// Flags affected: CF=0, OF=0, SF, ZF, PF (AF undefined)
pub fn xor_rm_r(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let result = (dst_value as u8) ^ (src_value as u8);
        cpu.write_operand(mem, &instr.dst, result as u16);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Xor8);
    } else {
        let result = dst_value ^ src_value;
        cpu.write_operand(mem, &instr.dst, result);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Xor16);
    }
}

/// XOR r, r/m - Logical XOR register/memory with register
/// Handles both byte (0x32) and word (0x33) variants
///
/// Flags affected: CF=0, OF=0, SF, ZF, PF (AF undefined)
pub fn xor_r_rm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let result = (dst_value as u8) ^ (src_value as u8);
        cpu.write_operand(mem, &instr.dst, result as u16);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Xor8);
    } else {
        let result = dst_value ^ src_value;
        cpu.write_operand(mem, &instr.dst, result);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Xor16);
    }
}

/// XOR AL/AX, imm - Logical XOR immediate with AL or AX
/// Handles byte (0x34) and word (0x35) variants
///
/// Flags affected: CF=0, OF=0, SF, ZF, PF (AF undefined)
pub fn xor_acc_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let result = (dst_value as u8) ^ (imm_value as u8);
        cpu.write_operand(mem, &instr.dst, result as u16);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Xor8);
    } else {
        let result = dst_value ^ imm_value;
        cpu.write_operand(mem, &instr.dst, result);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Xor16);
    }
}

/// XOR r/m, imm - Logical XOR immediate with register/memory
/// Handles byte (0x80 /6, 0x82 /6) and word (0x81 /6, 0x83 /6) variants
///
/// For 0x83, the immediate byte is sign-extended to word size.
///
/// Flags affected: CF=0, OF=0, SF, ZF, PF (AF undefined)
pub fn xor_rm_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let result = (dst_value as u8) ^ (imm_value as u8);
        cpu.write_operand(mem, &instr.dst, result as u16);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Xor8);
    } else {
        let result = dst_value ^ imm_value;
        cpu.write_operand(mem, &instr.dst, result);
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::Xor16);
    }
}

/// TEST r/m, r - Logical TEST (AND without storing result)
/// Handles both byte (0x84) and word (0x85) variants
///
/// Performs bitwise AND of two operands and sets flags, but does not store the result.
/// This is commonly used to test if specific bits are set.
///
/// Flags affected: CF=0, OF=0, SF, ZF, PF (AF undefined)
pub fn test_rm_r(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let result = (dst_value as u8) & (src_value as u8);
        // TEST doesn't write back the result
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::And8);
    } else {
        let result = dst_value & src_value;
        // TEST doesn't write back the result
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::And16);
    }
}

/// TEST AL/AX, imm - Logical TEST immediate with AL or AX
/// Handles byte (0xA8) and word (0xA9) variants
///
/// Performs bitwise AND of AL/AX with immediate value and sets flags, but does not store the result.
///
/// Flags affected: CF=0, OF=0, SF, ZF, PF (AF undefined)
pub fn test_acc_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let result = (dst_value as u8) & (imm_value as u8);
        // TEST doesn't write back the result
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::And8);
    } else {
        let result = dst_value & imm_value;
        // TEST doesn't write back the result
        cpu.clear_of_cf_af();
        cpu.set_lazy_flags(result as u32, FlagOp::And16);
    }
}
