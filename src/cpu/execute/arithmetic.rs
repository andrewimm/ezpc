//! Arithmetic instruction handlers (ADD, SUB, INC, DEC, etc.)

use crate::cpu::decode::{DecodedInstruction, OperandType};
use crate::cpu::state::FlagOp;
use crate::cpu::Cpu;
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
        let dst8 = dst_value as u8;
        let src8 = src_value as u8;
        let result = dst8 as u32 + src8 as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_add8_of_af(dst8, src8, result);
        cpu.set_lazy_flags(result, FlagOp::Add8);
    } else {
        let dst16 = dst_value;
        let src16 = src_value;
        let result = dst16 as u32 + src16 as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_add16_of_af(dst16, src16, result);
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
        let dst8 = dst_value as u8;
        let src8 = src_value as u8;
        let result = dst8 as u32 + src8 as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_add8_of_af(dst8, src8, result);
        cpu.set_lazy_flags(result, FlagOp::Add8);
    } else {
        let dst16 = dst_value;
        let src16 = src_value;
        let result = dst16 as u32 + src16 as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_add16_of_af(dst16, src16, result);
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
        let dst8 = dst_value as u8;
        let imm8 = imm_value as u8;
        let result = dst8 as u32 + imm8 as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_add8_of_af(dst8, imm8, result);
        cpu.set_lazy_flags(result, FlagOp::Add8);
    } else {
        let dst16 = dst_value;
        let imm16 = imm_value;
        let result = dst16 as u32 + imm16 as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_add16_of_af(dst16, imm16, result);
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
        let dst8 = dst_value as u8;
        let imm8 = imm_value as u8;
        let result = dst8 as u32 + imm8 as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_add8_of_af(dst8, imm8, result);
        cpu.set_lazy_flags(result, FlagOp::Add8);
    } else {
        let dst16 = dst_value;
        let imm16 = imm_value;
        let result = dst16 as u32 + imm16 as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_add16_of_af(dst16, imm16, result);
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
    cpu.set_inc16_of_af(value, result);
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
        let val8 = value as u8;
        let result = val8.wrapping_add(1);
        cpu.write_operand(mem, &instr.dst, result as u16);
        cpu.set_inc8_of_af(val8, result);
        cpu.set_lazy_flags(result as u32, FlagOp::Inc8);
    } else {
        let result = value.wrapping_add(1);
        cpu.write_operand(mem, &instr.dst, result);
        cpu.set_inc16_of_af(value, result);
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
    cpu.set_dec16_of_af(value, result);
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
        let val8 = value as u8;
        let result = val8.wrapping_sub(1);
        cpu.write_operand(mem, &instr.dst, result as u16);
        cpu.set_dec8_of_af(val8, result);
        cpu.set_lazy_flags(result as u32, FlagOp::Dec8);
    } else {
        let result = value.wrapping_sub(1);
        cpu.write_operand(mem, &instr.dst, result);
        cpu.set_dec16_of_af(value, result);
        cpu.set_lazy_flags(result as u32, FlagOp::Dec16);
    }
}

/// SUB r/m, r - Subtract register from register/memory
/// Handles both byte (0x28) and word (0x29) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn sub_rm_r(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let dst8 = dst_value as u8;
        let src8 = src_value as u8;
        let result = (dst8 as u32).wrapping_sub(src8 as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_sub8_of_af(dst8, src8, result);
        cpu.set_lazy_flags(result, FlagOp::Sub8);
    } else {
        let dst16 = dst_value;
        let src16 = src_value;
        let result = (dst16 as u32).wrapping_sub(src16 as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_sub16_of_af(dst16, src16, result);
        cpu.set_lazy_flags(result, FlagOp::Sub16);
    }
}

/// SUB r, r/m - Subtract register/memory from register
/// Handles both byte (0x2A) and word (0x2B) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn sub_r_rm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let dst8 = dst_value as u8;
        let src8 = src_value as u8;
        let result = (dst8 as u32).wrapping_sub(src8 as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_sub8_of_af(dst8, src8, result);
        cpu.set_lazy_flags(result, FlagOp::Sub8);
    } else {
        let dst16 = dst_value;
        let src16 = src_value;
        let result = (dst16 as u32).wrapping_sub(src16 as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_sub16_of_af(dst16, src16, result);
        cpu.set_lazy_flags(result, FlagOp::Sub16);
    }
}

/// SUB AL/AX, imm - Subtract immediate from AL or AX
/// Handles byte (0x2C) and word (0x2D) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn sub_acc_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let dst8 = dst_value as u8;
        let imm8 = imm_value as u8;
        let result = (dst8 as u32).wrapping_sub(imm8 as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_sub8_of_af(dst8, imm8, result);
        cpu.set_lazy_flags(result, FlagOp::Sub8);
    } else {
        let dst16 = dst_value;
        let imm16 = imm_value;
        let result = (dst16 as u32).wrapping_sub(imm16 as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_sub16_of_af(dst16, imm16, result);
        cpu.set_lazy_flags(result, FlagOp::Sub16);
    }
}

/// SUB r/m, imm - Subtract immediate from register/memory
/// Handles byte (0x80 /5, 0x82 /5) and word (0x81 /5, 0x83 /5) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn sub_rm_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let dst8 = dst_value as u8;
        let imm8 = imm_value as u8;
        let result = (dst8 as u32).wrapping_sub(imm8 as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_sub8_of_af(dst8, imm8, result);
        cpu.set_lazy_flags(result, FlagOp::Sub8);
    } else {
        let dst16 = dst_value;
        let imm16 = imm_value;
        let result = (dst16 as u32).wrapping_sub(imm16 as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_sub16_of_af(dst16, imm16, result);
        cpu.set_lazy_flags(result, FlagOp::Sub16);
    }
}

/// ADC r/m, r - Add with carry register to register/memory
/// Handles both byte (0x10) and word (0x11) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn adc_rm_r(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);
    let carry = if cpu.get_flag(Cpu::CF) { 1 } else { 0 };

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let dst8 = dst_value as u8;
        let src8 = src_value as u8;
        let result = dst8 as u32 + src8 as u32 + carry as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_adc8_of_af(dst8, src8, carry as u8, result);
        cpu.set_lazy_flags(result, FlagOp::Adc8);
    } else {
        let dst16 = dst_value;
        let src16 = src_value;
        let result = dst16 as u32 + src16 as u32 + carry as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_adc16_of_af(dst16, src16, carry as u16, result);
        cpu.set_lazy_flags(result, FlagOp::Adc16);
    }
}

/// ADC r, r/m - Add with carry register/memory to register
/// Handles both byte (0x12) and word (0x13) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn adc_r_rm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);
    let carry = if cpu.get_flag(Cpu::CF) { 1 } else { 0 };

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let dst8 = dst_value as u8;
        let src8 = src_value as u8;
        let result = dst8 as u32 + src8 as u32 + carry as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_adc8_of_af(dst8, src8, carry as u8, result);
        cpu.set_lazy_flags(result, FlagOp::Adc8);
    } else {
        let dst16 = dst_value;
        let src16 = src_value;
        let result = dst16 as u32 + src16 as u32 + carry as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_adc16_of_af(dst16, src16, carry as u16, result);
        cpu.set_lazy_flags(result, FlagOp::Adc16);
    }
}

/// ADC AL/AX, imm - Add with carry immediate to AL or AX
/// Handles byte (0x14) and word (0x15) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn adc_acc_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);
    let carry = if cpu.get_flag(Cpu::CF) { 1 } else { 0 };

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let dst8 = dst_value as u8;
        let imm8 = imm_value as u8;
        let result = dst8 as u32 + imm8 as u32 + carry as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_adc8_of_af(dst8, imm8, carry as u8, result);
        cpu.set_lazy_flags(result, FlagOp::Adc8);
    } else {
        let dst16 = dst_value;
        let imm16 = imm_value;
        let result = dst16 as u32 + imm16 as u32 + carry as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_adc16_of_af(dst16, imm16, carry as u16, result);
        cpu.set_lazy_flags(result, FlagOp::Adc16);
    }
}

/// ADC r/m, imm - Add with carry immediate to register/memory
/// Handles byte (0x80 /2, 0x82 /2) and word (0x81 /2, 0x83 /2) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn adc_rm_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);
    let carry = if cpu.get_flag(Cpu::CF) { 1 } else { 0 };

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let dst8 = dst_value as u8;
        let imm8 = imm_value as u8;
        let result = dst8 as u32 + imm8 as u32 + carry as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_adc8_of_af(dst8, imm8, carry as u8, result);
        cpu.set_lazy_flags(result, FlagOp::Adc8);
    } else {
        let dst16 = dst_value;
        let imm16 = imm_value;
        let result = dst16 as u32 + imm16 as u32 + carry as u32;
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_adc16_of_af(dst16, imm16, carry as u16, result);
        cpu.set_lazy_flags(result, FlagOp::Adc16);
    }
}

/// SBB r/m, r - Subtract with borrow register from register/memory
/// Handles both byte (0x18) and word (0x19) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn sbb_rm_r(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);
    let borrow = if cpu.get_flag(Cpu::CF) { 1 } else { 0 };

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let dst8 = dst_value as u8;
        let src8 = src_value as u8;
        let result = (dst8 as u32)
            .wrapping_sub(src8 as u32)
            .wrapping_sub(borrow as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_sbb8_of_af(dst8, src8, borrow as u8, result);
        cpu.set_lazy_flags(result, FlagOp::Sbb8);
    } else {
        let dst16 = dst_value;
        let src16 = src_value;
        let result = (dst16 as u32)
            .wrapping_sub(src16 as u32)
            .wrapping_sub(borrow as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_sbb16_of_af(dst16, src16, borrow as u16, result);
        cpu.set_lazy_flags(result, FlagOp::Sbb16);
    }
}

/// SBB r, r/m - Subtract with borrow register/memory from register
/// Handles both byte (0x1A) and word (0x1B) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn sbb_r_rm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);
    let borrow = if cpu.get_flag(Cpu::CF) { 1 } else { 0 };

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let dst8 = dst_value as u8;
        let src8 = src_value as u8;
        let result = (dst8 as u32)
            .wrapping_sub(src8 as u32)
            .wrapping_sub(borrow as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_sbb8_of_af(dst8, src8, borrow as u8, result);
        cpu.set_lazy_flags(result, FlagOp::Sbb8);
    } else {
        let dst16 = dst_value;
        let src16 = src_value;
        let result = (dst16 as u32)
            .wrapping_sub(src16 as u32)
            .wrapping_sub(borrow as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_sbb16_of_af(dst16, src16, borrow as u16, result);
        cpu.set_lazy_flags(result, FlagOp::Sbb16);
    }
}

/// SBB AL/AX, imm - Subtract with borrow immediate from AL or AX
/// Handles byte (0x1C) and word (0x1D) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn sbb_acc_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);
    let borrow = if cpu.get_flag(Cpu::CF) { 1 } else { 0 };

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let dst8 = dst_value as u8;
        let imm8 = imm_value as u8;
        let result = (dst8 as u32)
            .wrapping_sub(imm8 as u32)
            .wrapping_sub(borrow as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_sbb8_of_af(dst8, imm8, borrow as u8, result);
        cpu.set_lazy_flags(result, FlagOp::Sbb8);
    } else {
        let dst16 = dst_value;
        let imm16 = imm_value;
        let result = (dst16 as u32)
            .wrapping_sub(imm16 as u32)
            .wrapping_sub(borrow as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_sbb16_of_af(dst16, imm16, borrow as u16, result);
        cpu.set_lazy_flags(result, FlagOp::Sbb16);
    }
}

/// SBB r/m, imm - Subtract with borrow immediate from register/memory
/// Handles byte (0x80 /3, 0x82 /3) and word (0x81 /3, 0x83 /3) variants
///
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn sbb_rm_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);
    let borrow = if cpu.get_flag(Cpu::CF) { 1 } else { 0 };

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let dst8 = dst_value as u8;
        let imm8 = imm_value as u8;
        let result = (dst8 as u32)
            .wrapping_sub(imm8 as u32)
            .wrapping_sub(borrow as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);
        cpu.set_sbb8_of_af(dst8, imm8, borrow as u8, result);
        cpu.set_lazy_flags(result, FlagOp::Sbb8);
    } else {
        let dst16 = dst_value;
        let imm16 = imm_value;
        let result = (dst16 as u32)
            .wrapping_sub(imm16 as u32)
            .wrapping_sub(borrow as u32);
        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);
        cpu.set_sbb16_of_af(dst16, imm16, borrow as u16, result);
        cpu.set_lazy_flags(result, FlagOp::Sbb16);
    }
}

/// CMP r/m, r - Compare register with register/memory
/// Handles both byte (0x38) and word (0x39) variants
///
/// Performs dst - src and sets flags without storing the result
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn cmp_rm_r(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let dst8 = dst_value as u8;
        let src8 = src_value as u8;
        let result = (dst8 as u32).wrapping_sub(src8 as u32);
        // CMP does not write the result, only sets flags
        cpu.set_sub8_of_af(dst8, src8, result);
        cpu.set_lazy_flags(result, FlagOp::Sub8);
    } else {
        let dst16 = dst_value;
        let src16 = src_value;
        let result = (dst16 as u32).wrapping_sub(src16 as u32);
        // CMP does not write the result, only sets flags
        cpu.set_sub16_of_af(dst16, src16, result);
        cpu.set_lazy_flags(result, FlagOp::Sub16);
    }
}

/// CMP r, r/m - Compare register/memory with register
/// Handles both byte (0x3A) and word (0x3B) variants
///
/// Performs dst - src and sets flags without storing the result
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn cmp_r_rm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let dst8 = dst_value as u8;
        let src8 = src_value as u8;
        let result = (dst8 as u32).wrapping_sub(src8 as u32);
        // CMP does not write the result, only sets flags
        cpu.set_sub8_of_af(dst8, src8, result);
        cpu.set_lazy_flags(result, FlagOp::Sub8);
    } else {
        let dst16 = dst_value;
        let src16 = src_value;
        let result = (dst16 as u32).wrapping_sub(src16 as u32);
        // CMP does not write the result, only sets flags
        cpu.set_sub16_of_af(dst16, src16, result);
        cpu.set_lazy_flags(result, FlagOp::Sub16);
    }
}

/// CMP AL/AX, imm - Compare immediate with AL or AX
/// Handles byte (0x3C) and word (0x3D) variants
///
/// Performs AL/AX - imm and sets flags without storing the result
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn cmp_acc_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8;

    if is_byte {
        let dst8 = dst_value as u8;
        let imm8 = imm_value as u8;
        let result = (dst8 as u32).wrapping_sub(imm8 as u32);
        // CMP does not write the result, only sets flags
        cpu.set_sub8_of_af(dst8, imm8, result);
        cpu.set_lazy_flags(result, FlagOp::Sub8);
    } else {
        let dst16 = dst_value;
        let imm16 = imm_value;
        let result = (dst16 as u32).wrapping_sub(imm16 as u32);
        // CMP does not write the result, only sets flags
        cpu.set_sub16_of_af(dst16, imm16, result);
        cpu.set_lazy_flags(result, FlagOp::Sub16);
    }
}

/// CMP r/m, imm - Compare immediate with register/memory
/// Handles byte (0x80 /7, 0x82 /7) and word (0x81 /7, 0x83 /7) variants
///
/// Performs r/m - imm and sets flags without storing the result
/// Flags affected: CF, PF, AF, ZF, SF, OF
pub fn cmp_rm_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        let dst8 = dst_value as u8;
        let imm8 = imm_value as u8;
        let result = (dst8 as u32).wrapping_sub(imm8 as u32);
        // CMP does not write the result, only sets flags
        cpu.set_sub8_of_af(dst8, imm8, result);
        cpu.set_lazy_flags(result, FlagOp::Sub8);
    } else {
        let dst16 = dst_value;
        let imm16 = imm_value;
        let result = (dst16 as u32).wrapping_sub(imm16 as u32);
        // CMP does not write the result, only sets flags
        cpu.set_sub16_of_af(dst16, imm16, result);
        cpu.set_lazy_flags(result, FlagOp::Sub16);
    }
}

/// Group handler for opcode 0x80 and 0x82 - Arithmetic r/m8, imm8
/// Dispatches to ADD, OR, ADC, SBB, AND, SUB, XOR, or CMP based on reg field
pub fn group_80(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let reg = (instr.dst.value >> 8) as u8; // High byte stores the reg field

    match reg {
        0 => add_rm_imm(cpu, mem, instr),               // ADD r/m8, imm8
        1 => super::logic::or_rm_imm(cpu, mem, instr),  // OR r/m8, imm8
        2 => adc_rm_imm(cpu, mem, instr),               // ADC r/m8, imm8
        3 => sbb_rm_imm(cpu, mem, instr),               // SBB r/m8, imm8
        4 => super::logic::and_rm_imm(cpu, mem, instr), // AND r/m8, imm8
        5 => sub_rm_imm(cpu, mem, instr),               // SUB r/m8, imm8
        6 => super::logic::xor_rm_imm(cpu, mem, instr), // XOR r/m8, imm8
        7 => cmp_rm_imm(cpu, mem, instr),               // CMP r/m8, imm8
        _ => unreachable!(),
    }
}

/// Group handler for opcode 0x81 - Arithmetic r/m16, imm16
/// Dispatches to ADD, OR, ADC, SBB, AND, SUB, XOR, or CMP based on reg field
pub fn group_81(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let reg = (instr.dst.value >> 8) as u8;

    match reg {
        0 => add_rm_imm(cpu, mem, instr),               // ADD r/m16, imm16
        1 => super::logic::or_rm_imm(cpu, mem, instr),  // OR r/m16, imm16
        2 => adc_rm_imm(cpu, mem, instr),               // ADC r/m16, imm16
        3 => sbb_rm_imm(cpu, mem, instr),               // SBB r/m16, imm16
        4 => super::logic::and_rm_imm(cpu, mem, instr), // AND r/m16, imm16
        5 => sub_rm_imm(cpu, mem, instr),               // SUB r/m16, imm16
        6 => super::logic::xor_rm_imm(cpu, mem, instr), // XOR r/m16, imm16
        7 => cmp_rm_imm(cpu, mem, instr),               // CMP r/m16, imm16
        _ => unreachable!(),
    }
}

/// Group handler for opcode 0x83 - Arithmetic r/m16, imm8 (sign-extended)
/// Dispatches to ADD, OR, ADC, SBB, AND, SUB, XOR, or CMP based on reg field
pub fn group_83(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let reg = (instr.dst.value >> 8) as u8;

    match reg {
        0 => add_rm_imm(cpu, mem, instr), // ADD r/m16, imm8 (sign-extended)
        1 => super::logic::or_rm_imm(cpu, mem, instr), // OR r/m16, imm8 (sign-extended)
        2 => adc_rm_imm(cpu, mem, instr), // ADC r/m16, imm8 (sign-extended)
        3 => sbb_rm_imm(cpu, mem, instr), // SBB r/m16, imm8 (sign-extended)
        4 => super::logic::and_rm_imm(cpu, mem, instr), // AND r/m16, imm8 (sign-extended)
        5 => sub_rm_imm(cpu, mem, instr), // SUB r/m16, imm8 (sign-extended)
        6 => super::logic::xor_rm_imm(cpu, mem, instr), // XOR r/m16, imm8 (sign-extended)
        7 => cmp_rm_imm(cpu, mem, instr), // CMP r/m16, imm8 (sign-extended)
        _ => unreachable!(),
    }
}

/// DAA - Decimal Adjust After Addition
/// Opcode: 0x27
///
/// Adjusts AL after a binary addition to maintain BCD (Binary Coded Decimal) format.
/// This instruction is used after adding two packed BCD values.
///
/// Algorithm:
/// 1. If ((AL & 0x0F) > 9) OR (AF == 1):
///    - AL = AL + 6
///    - AF = 1
/// 2. If (old_AL > 0x99) OR (old_CF == 1):
///    - AL = AL + 0x60
///    - CF = 1
///
/// Flags affected: SF, ZF, PF, CF, AF (OF is undefined)
pub fn daa(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let mut al = cpu.read_reg8(0); // Read AL
    let old_al = al;
    let old_cf = cpu.get_flag(Cpu::CF);
    let old_af = cpu.get_flag(Cpu::AF);

    // Step 1: Adjust low nibble if needed
    let new_af = if (al & 0x0F) > 9 || old_af {
        al = al.wrapping_add(6);
        true
    } else {
        false
    };

    // Step 2: Adjust high nibble if needed
    let new_cf = if old_al > 0x99 || old_cf {
        al = al.wrapping_add(0x60);
        true
    } else {
        false
    };

    // Write result back to AL
    cpu.write_reg8(0, al);

    // Manually set flags instead of using lazy evaluation
    // to avoid overwriting CF and AF
    let mut flags = cpu.get_flags();

    // Clear flags we're about to set
    flags &= !(Cpu::SF | Cpu::ZF | Cpu::PF | Cpu::CF | Cpu::AF);

    // Set SF (sign flag) if bit 7 is set
    if al & 0x80 != 0 {
        flags |= Cpu::SF;
    }

    // Set ZF (zero flag) if result is zero
    if al == 0 {
        flags |= Cpu::ZF;
    }

    // Set PF (parity flag) if even number of 1 bits in low byte
    if al.count_ones() % 2 == 0 {
        flags |= Cpu::PF;
    }

    // Set CF and AF as computed above
    if new_cf {
        flags |= Cpu::CF;
    }
    if new_af {
        flags |= Cpu::AF;
    }

    cpu.set_flags(flags);
}

/// DAS - Decimal Adjust After Subtraction
/// Opcode: 0x2F
///
/// Adjusts AL after a binary subtraction to maintain BCD (Binary Coded Decimal) format.
/// This instruction is used after subtracting two packed BCD values.
///
/// Algorithm:
/// 1. If ((AL & 0x0F) > 9) OR (AF == 1):
///    - AL = AL - 6
///    - AF = 1
/// 2. If (old_AL > 0x99) OR (old_CF == 1):
///    - AL = AL - 0x60
///    - CF = 1
///
/// Flags affected: SF, ZF, PF, CF, AF (OF is undefined)
pub fn das(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let mut al = cpu.read_reg8(0); // Read AL
    let old_al = al;
    let old_cf = cpu.get_flag(Cpu::CF);
    let old_af = cpu.get_flag(Cpu::AF);

    // Step 1: Adjust low nibble if needed
    let new_af = if (al & 0x0F) > 9 || old_af {
        al = al.wrapping_sub(6);
        true
    } else {
        false
    };

    // Step 2: Adjust high nibble if needed
    let new_cf = if old_al > 0x99 || old_cf {
        al = al.wrapping_sub(0x60);
        true
    } else {
        false
    };

    // Write result back to AL
    cpu.write_reg8(0, al);

    // Manually set flags instead of using lazy evaluation
    // to avoid overwriting CF and AF
    let mut flags = cpu.get_flags();

    // Clear flags we're about to set
    flags &= !(Cpu::SF | Cpu::ZF | Cpu::PF | Cpu::CF | Cpu::AF);

    // Set SF (sign flag) if bit 7 is set
    if al & 0x80 != 0 {
        flags |= Cpu::SF;
    }

    // Set ZF (zero flag) if result is zero
    if al == 0 {
        flags |= Cpu::ZF;
    }

    // Set PF (parity flag) if even number of 1 bits in low byte
    if al.count_ones() % 2 == 0 {
        flags |= Cpu::PF;
    }

    // Set CF and AF as computed above
    if new_cf {
        flags |= Cpu::CF;
    }
    if new_af {
        flags |= Cpu::AF;
    }

    cpu.set_flags(flags);
}

/// AAA - ASCII Adjust After Addition
/// Opcode: 0x37
///
/// Adjusts AL after adding two unpacked BCD digits.
/// If the low nibble of AL is greater than 9 or AF is set:
/// - AL = AL + 6
/// - AH = AH + 1
/// - AF = 1, CF = 1
/// Then AL is masked to keep only the low nibble (AL &= 0x0F)
///
/// Flags affected: AF, CF (SF, ZF, PF, OF are undefined)
pub fn aaa(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let mut al = cpu.read_reg8(0); // Read AL
    let mut ah = cpu.read_reg8(4); // Read AH
    let old_af = cpu.get_flag(Cpu::AF);

    // Check if adjustment is needed
    if (al & 0x0F) > 9 || old_af {
        // Adjust AL and AH
        al = al.wrapping_add(6);
        ah = ah.wrapping_add(1);

        // Set AF and CF
        cpu.set_flag(Cpu::AF, true);
        cpu.set_flag(Cpu::CF, true);
    } else {
        // Clear AF and CF
        cpu.set_flag(Cpu::AF, false);
        cpu.set_flag(Cpu::CF, false);
    }

    // Mask AL to keep only low nibble
    al &= 0x0F;

    // Write results back
    cpu.write_reg8(0, al); // AL
    cpu.write_reg8(4, ah); // AH
}

/// AAS - ASCII Adjust After Subtraction
/// Opcode: 0x3F
///
/// Adjusts AL after subtracting two unpacked BCD digits.
/// If the low nibble of AL is greater than 9 or AF is set:
/// - AL = AL - 6
/// - AH = AH - 1
/// - AF = 1, CF = 1
/// Then AL is masked to keep only the low nibble (AL &= 0x0F)
///
/// Flags affected: AF, CF (SF, ZF, PF, OF are undefined)
pub fn aas(cpu: &mut Cpu, _mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    let mut al = cpu.read_reg8(0); // Read AL
    let mut ah = cpu.read_reg8(4); // Read AH
    let old_af = cpu.get_flag(Cpu::AF);

    // Check if adjustment is needed
    if (al & 0x0F) > 9 || old_af {
        // Adjust AL and AH
        al = al.wrapping_sub(6);
        ah = ah.wrapping_sub(1);

        // Set AF and CF
        cpu.set_flag(Cpu::AF, true);
        cpu.set_flag(Cpu::CF, true);
    } else {
        // Clear AF and CF
        cpu.set_flag(Cpu::AF, false);
        cpu.set_flag(Cpu::CF, false);
    }

    // Mask AL to keep only low nibble
    al &= 0x0F;

    // Write results back
    cpu.write_reg8(0, al); // AL
    cpu.write_reg8(4, ah); // AH
}

/// AAM - ASCII Adjust AX after Multiply
/// Opcode: 0xD4 imm8
///
/// Adjusts AX after multiplying two unpacked BCD digits.
/// Divides AL by the immediate operand (usually 10) and stores:
/// - Quotient in AH
/// - Remainder in AL
///
/// Algorithm:
/// - AH = AL / imm8
/// - AL = AL % imm8
///
/// Flags affected: SF, ZF, PF (CF, AF, OF are undefined)
pub fn aam(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    // Read the immediate byte (base, usually 0x0A for decimal)
    let base = cpu.read_operand(mem, &instr.src) as u8;

    // Division by zero causes interrupt 0
    if base == 0 {
        panic!("AAM: Division by zero");
    }

    let al = cpu.read_reg8(0); // Read AL

    // Perform the division
    let ah = al / base;
    let al_new = al % base;

    // Write results back to AH and AL
    cpu.write_reg8(4, ah); // AH
    cpu.write_reg8(0, al_new); // AL

    // Set flags based on the full AX value
    let ax = cpu.regs[0];
    cpu.set_lazy_flags(ax as u32, FlagOp::And16);
}

/// AAD - ASCII Adjust AX before Division
/// Opcode: 0xD5 imm8
///
/// Adjusts AX before dividing two unpacked BCD digits.
/// Converts unpacked BCD in AH:AL to binary in AL.
///
/// Algorithm:
/// - AL = AH * imm8 + AL
/// - AH = 0
///
/// Flags affected: SF, ZF, PF (CF, AF, OF are undefined)
pub fn aad(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    // Read the immediate byte (base, usually 0x0A for decimal)
    let base = cpu.read_operand(mem, &instr.src) as u8;

    let al = cpu.read_reg8(0); // Read AL
    let ah = cpu.read_reg8(4); // Read AH

    // Perform the conversion: AL = AH * base + AL
    let al_new = (ah.wrapping_mul(base)).wrapping_add(al);

    // Write results back
    cpu.write_reg8(0, al_new); // AL
    cpu.write_reg8(4, 0); // AH = 0

    // Set flags based on AL value
    cpu.set_lazy_flags(al_new as u32, FlagOp::And8);
}

/// Group handler for opcode 0xFE
/// Handles INC/DEC r/m8 based on reg field
pub fn group_fe(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    // The reg field is stored in the value field of dst operand during group decoding
    let reg = (instr.dst.value >> 8) as u8; // High byte stores the reg field

    match reg {
        0 => inc_rm(cpu, mem, instr), // INC r/m8
        1 => dec_rm(cpu, mem, instr), // DEC r/m8
        _ => panic!("Invalid reg field {} for opcode 0xFE", reg),
    }
}

/// MUL r/m - Unsigned multiply
/// Handles both byte (0xF6 /4) and word (0xF7 /4) variants
///
/// For byte operation: AL * r/m8 → AX
/// For word operation: AX * r/m16 → DX:AX
///
/// Flags affected: CF, OF (set if upper half is non-zero)
///                 SF, ZF, PF, AF are undefined
pub fn mul(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let operand_value = cpu.read_operand(mem, &instr.dst);
    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        // 8-bit multiply: AL * r/m8 → AX
        let al = cpu.read_reg8(0); // Read AL
        let result = (al as u16) * (operand_value as u8 as u16);
        cpu.regs[0] = result; // Store full 16-bit result in AX

        // CF and OF are set if AH is non-zero (upper 8 bits)
        let upper_half_nonzero = (result & 0xFF00) != 0;
        if upper_half_nonzero {
            cpu.set_flag(Cpu::CF, true);
            cpu.set_flag(Cpu::OF, true);
        } else {
            cpu.set_flag(Cpu::CF, false);
            cpu.set_flag(Cpu::OF, false);
        }
    } else {
        // 16-bit multiply: AX * r/m16 → DX:AX
        let ax = cpu.regs[0]; // Read AX
        let result = (ax as u32) * (operand_value as u32);

        cpu.regs[0] = (result & 0xFFFF) as u16; // Store low word in AX
        cpu.regs[2] = (result >> 16) as u16; // Store high word in DX

        // CF and OF are set if DX is non-zero (upper 16 bits)
        let upper_half_nonzero = (result & 0xFFFF0000) != 0;
        if upper_half_nonzero {
            cpu.set_flag(Cpu::CF, true);
            cpu.set_flag(Cpu::OF, true);
        } else {
            cpu.set_flag(Cpu::CF, false);
            cpu.set_flag(Cpu::OF, false);
        }
    }
}

/// DIV r/m - Unsigned Divide
/// Opcodes: 0xF6 /6 (8-bit), 0xF7 /6 (16-bit)
///
/// Performs unsigned division:
/// - 8-bit: AX ÷ r/m8 → AL (quotient), AH (remainder)
/// - 16-bit: DX:AX ÷ r/m16 → AX (quotient), DX (remainder)
///
/// Triggers interrupt 0 if:
/// - Divisor is 0 (divide by zero)
/// - Quotient doesn't fit in destination register (overflow)
///
/// Flags: All flags are undefined after DIV
pub fn div(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let divisor = cpu.read_operand(mem, &instr.dst);
    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        // 8-bit divide: AX ÷ r/m8 → AL (quotient), AH (remainder)
        let divisor = divisor as u8;

        // Check for divide by zero
        if divisor == 0 {
            panic!("DIV: Division by zero");
        }

        let ax = cpu.regs[0]; // Read AX (dividend)
        let quotient = ax / (divisor as u16);
        let remainder = ax % (divisor as u16);

        // Check for quotient overflow (quotient must fit in AL)
        if quotient > 0xFF {
            panic!("DIV: Quotient overflow (result doesn't fit in AL)");
        }

        // Store quotient in AL, remainder in AH
        let al = quotient as u8;
        let ah = remainder as u8;
        cpu.regs[0] = ((ah as u16) << 8) | (al as u16);
    } else {
        // 16-bit divide: DX:AX ÷ r/m16 → AX (quotient), DX (remainder)
        let divisor = divisor as u16;

        // Check for divide by zero
        if divisor == 0 {
            panic!("DIV: Division by zero");
        }

        let ax = cpu.regs[0]; // Low word
        let dx = cpu.regs[2]; // High word
        let dividend = ((dx as u32) << 16) | (ax as u32);

        let quotient = dividend / (divisor as u32);
        let remainder = dividend % (divisor as u32);

        // Check for quotient overflow (quotient must fit in AX)
        if quotient > 0xFFFF {
            panic!("DIV: Quotient overflow (result doesn't fit in AX)");
        }

        // Store quotient in AX, remainder in DX
        cpu.regs[0] = quotient as u16;
        cpu.regs[2] = remainder as u16;
    }

    // Flags are undefined after DIV - we don't modify them
}

/// NEG r/m - Two's Complement Negation
/// Opcodes: 0xF6 /3 (8-bit), 0xF7 /3 (16-bit)
///
/// Subtracts the operand from zero (replaces with two's complement).
/// Equivalent to: operand = 0 - operand
///
/// Flags: CF (set if source != 0), OF, SF, ZF, PF, AF
pub fn neg(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let operand_value = cpu.read_operand(mem, &instr.dst);
    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        // 8-bit negate: 0 - value
        let value = operand_value as u8;
        let result = (0u32).wrapping_sub(value as u32);

        cpu.write_operand(mem, &instr.dst, (result & 0xFF) as u16);

        // Set OF and AF using the same logic as SUB (0 - value)
        cpu.set_sub8_of_af(0, value, result);

        // Set SF, ZF, PF using lazy flags
        cpu.set_lazy_flags(result, FlagOp::Sub8);

        // CF is set if source operand is not zero (special case for NEG)
        cpu.set_flag(Cpu::CF, value != 0);
    } else {
        // 16-bit negate: 0 - value
        let value = operand_value;
        let result = (0u32).wrapping_sub(value as u32);

        cpu.write_operand(mem, &instr.dst, (result & 0xFFFF) as u16);

        // Set OF and AF using the same logic as SUB (0 - value)
        cpu.set_sub16_of_af(0, value, result);

        // Set SF, ZF, PF using lazy flags
        cpu.set_lazy_flags(result, FlagOp::Sub16);

        // CF is set if source operand is not zero (special case for NEG)
        cpu.set_flag(Cpu::CF, value != 0);
    }
}

/// IMUL r/m - Signed Multiply
/// Opcodes: 0xF6 /5 (8-bit), 0xF7 /5 (16-bit)
///
/// Performs signed multiplication:
/// - 8-bit: AL * r/m8 → AX (signed)
/// - 16-bit: AX * r/m16 → DX:AX (signed)
///
/// CF and OF are set if the result cannot be represented in the low half
/// (i.e., sign extension of the low half doesn't equal the full result)
pub fn imul(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let operand_value = cpu.read_operand(mem, &instr.dst);
    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        // 8-bit signed multiply: AL * r/m8 → AX
        let al = cpu.read_reg8(0) as i8; // Read AL as signed
        let operand = operand_value as u8 as i8; // Operand as signed
        let result = (al as i16) * (operand as i16);
        cpu.regs[0] = result as u16; // Store full 16-bit result in AX

        // CF and OF are set if the result cannot be represented in AL
        // (i.e., if sign-extending AL doesn't give the full AX result)
        let al_sign_extended = (result as u16 & 0xFF) as i8 as i16;
        let overflow = result != al_sign_extended;
        cpu.set_flag(Cpu::CF, overflow);
        cpu.set_flag(Cpu::OF, overflow);
    } else {
        // 16-bit signed multiply: AX * r/m16 → DX:AX
        let ax = cpu.regs[0] as i16; // Read AX as signed
        let operand = operand_value as i16; // Operand as signed
        let result = (ax as i32) * (operand as i32);

        cpu.regs[0] = (result & 0xFFFF) as u16; // Store low word in AX
        cpu.regs[2] = ((result >> 16) & 0xFFFF) as u16; // Store high word in DX

        // CF and OF are set if the result cannot be represented in AX
        // (i.e., if sign-extending AX doesn't give the full DX:AX result)
        let ax_sign_extended = (result as u32 & 0xFFFF) as i16 as i32;
        let overflow = result != ax_sign_extended;
        cpu.set_flag(Cpu::CF, overflow);
        cpu.set_flag(Cpu::OF, overflow);
    }
}

/// IDIV r/m - Signed Divide
/// Opcodes: 0xF6 /7 (8-bit), 0xF7 /7 (16-bit)
///
/// Performs signed division:
/// - 8-bit: AX ÷ r/m8 → AL (quotient), AH (remainder)
/// - 16-bit: DX:AX ÷ r/m16 → AX (quotient), DX (remainder)
///
/// The remainder has the same sign as the dividend.
///
/// Triggers interrupt 0 if:
/// - Divisor is 0 (divide by zero)
/// - Quotient doesn't fit in destination register (overflow)
///
/// Flags: All flags are undefined after IDIV
pub fn idiv(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let divisor = cpu.read_operand(mem, &instr.dst);
    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        // 8-bit signed divide: AX ÷ r/m8 → AL (quotient), AH (remainder)
        let divisor = divisor as u8 as i8;

        // Check for divide by zero
        if divisor == 0 {
            panic!("IDIV: Division by zero");
        }

        let ax = cpu.regs[0] as i16; // Read AX as signed dividend
        let quotient = ax / (divisor as i16);
        let remainder = ax % (divisor as i16);

        // Check for quotient overflow (quotient must fit in signed AL: -128 to 127)
        if quotient < -128 || quotient > 127 {
            panic!("IDIV: Quotient overflow (result doesn't fit in AL)");
        }

        // Store quotient in AL, remainder in AH
        let al = quotient as u8;
        let ah = remainder as u8;
        cpu.regs[0] = ((ah as u16) << 8) | (al as u16);
    } else {
        // 16-bit signed divide: DX:AX ÷ r/m16 → AX (quotient), DX (remainder)
        let divisor = divisor as i16;

        // Check for divide by zero
        if divisor == 0 {
            panic!("IDIV: Division by zero");
        }

        let ax = cpu.regs[0] as i16; // Low word (signed)
        let dx = cpu.regs[2] as i16; // High word (signed)
        let dividend = ((dx as i32) << 16) | (ax as u16 as i32);

        let quotient = dividend / (divisor as i32);
        let remainder = dividend % (divisor as i32);

        // Check for quotient overflow (quotient must fit in signed AX: -32768 to 32767)
        if quotient < -32768 || quotient > 32767 {
            panic!("IDIV: Quotient overflow (result doesn't fit in AX)");
        }

        // Store quotient in AX, remainder in DX
        cpu.regs[0] = quotient as u16;
        cpu.regs[2] = remainder as u16;
    }

    // Flags are undefined after IDIV - we don't modify them
}

/// NOT r/m - Bitwise NOT (one's complement)
/// Opcodes: 0xF6 /2 (8-bit), 0xF7 /2 (16-bit)
///
/// Inverts all bits in the operand (ones become zeros, zeros become ones).
/// No flags are affected.
pub fn not(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let operand_value = cpu.read_operand(mem, &instr.dst);
    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    if is_byte {
        // 8-bit NOT: invert all 8 bits
        let result = !operand_value as u8;
        cpu.write_operand(mem, &instr.dst, result as u16);
    } else {
        // 16-bit NOT: invert all 16 bits
        let result = !operand_value;
        cpu.write_operand(mem, &instr.dst, result);
    }
    // NOT does not affect any flags
}

/// Group handler for opcode 0xF6
/// Handles TEST/NOT/NEG/MUL/IMUL/DIV/IDIV r/m8 based on reg field
pub fn group_f6(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let reg = (instr.dst.value >> 8) as u8; // High byte stores the reg field

    match reg {
        0 | 1 => test_rm_imm(cpu, mem, instr), // TEST r/m8, imm8
        2 => not(cpu, mem, instr),             // NOT r/m8
        3 => neg(cpu, mem, instr),             // NEG r/m8
        4 => mul(cpu, mem, instr),             // MUL r/m8
        5 => imul(cpu, mem, instr),            // IMUL r/m8
        6 => div(cpu, mem, instr),             // DIV r/m8
        7 => idiv(cpu, mem, instr),            // IDIV r/m8
        _ => unreachable!(),
    }
}

/// Group handler for opcode 0xF7
/// Handles TEST/NOT/NEG/MUL/IMUL/DIV/IDIV r/m16 based on reg field
pub fn group_f7(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let reg = (instr.dst.value >> 8) as u8; // High byte stores the reg field

    match reg {
        0 | 1 => test_rm_imm(cpu, mem, instr), // TEST r/m16, imm16
        2 => not(cpu, mem, instr),             // NOT r/m16
        3 => neg(cpu, mem, instr),             // NEG r/m16
        4 => mul(cpu, mem, instr),             // MUL r/m16
        5 => imul(cpu, mem, instr),            // IMUL r/m16
        6 => div(cpu, mem, instr),             // DIV r/m16
        7 => idiv(cpu, mem, instr),            // IDIV r/m16
        _ => unreachable!(),
    }
}

/// TEST r/m, imm - Logical TEST immediate with register/memory
/// Used by Group 0xF6 (reg=0,1) and Group 0xF7 (reg=0,1)
///
/// Performs bitwise AND of r/m with immediate value and sets flags, but does not store the result.
///
/// Flags affected: CF=0, OF=0, SF, ZF, PF (AF undefined)
fn test_rm_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let imm_value = cpu.read_operand(mem, &instr.src);

    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

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
