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
        0 => add_rm_imm(cpu, mem, instr), // ADD r/m8, imm8
        1 => panic!("OR r/m8, imm8 not implemented yet"),
        2 => adc_rm_imm(cpu, mem, instr), // ADC r/m8, imm8
        3 => sbb_rm_imm(cpu, mem, instr), // SBB r/m8, imm8
        4 => panic!("AND r/m8, imm8 not implemented yet"),
        5 => sub_rm_imm(cpu, mem, instr), // SUB r/m8, imm8
        6 => panic!("XOR r/m8, imm8 not implemented yet"),
        7 => cmp_rm_imm(cpu, mem, instr), // CMP r/m8, imm8
        _ => unreachable!(),
    }
}

/// Group handler for opcode 0x81 - Arithmetic r/m16, imm16
/// Dispatches to ADD, OR, ADC, SBB, AND, SUB, XOR, or CMP based on reg field
pub fn group_81(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let reg = (instr.dst.value >> 8) as u8;

    match reg {
        0 => add_rm_imm(cpu, mem, instr), // ADD r/m16, imm16
        1 => panic!("OR r/m16, imm16 not implemented yet"),
        2 => adc_rm_imm(cpu, mem, instr), // ADC r/m16, imm16
        3 => sbb_rm_imm(cpu, mem, instr), // SBB r/m16, imm16
        4 => panic!("AND r/m16, imm16 not implemented yet"),
        5 => sub_rm_imm(cpu, mem, instr), // SUB r/m16, imm16
        6 => panic!("XOR r/m16, imm16 not implemented yet"),
        7 => cmp_rm_imm(cpu, mem, instr), // CMP r/m16, imm16
        _ => unreachable!(),
    }
}

/// Group handler for opcode 0x83 - Arithmetic r/m16, imm8 (sign-extended)
/// Dispatches to ADD, OR, ADC, SBB, AND, SUB, XOR, or CMP based on reg field
pub fn group_83(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let reg = (instr.dst.value >> 8) as u8;

    match reg {
        0 => add_rm_imm(cpu, mem, instr), // ADD r/m16, imm8 (sign-extended)
        1 => panic!("OR r/m16, imm8 not implemented yet"),
        2 => adc_rm_imm(cpu, mem, instr), // ADC r/m16, imm8 (sign-extended)
        3 => sbb_rm_imm(cpu, mem, instr), // SBB r/m16, imm8 (sign-extended)
        4 => panic!("AND r/m16, imm8 not implemented yet"),
        5 => sub_rm_imm(cpu, mem, instr), // SUB r/m16, imm8 (sign-extended)
        6 => panic!("XOR r/m16, imm8 not implemented yet"),
        7 => cmp_rm_imm(cpu, mem, instr), // CMP r/m16, imm8 (sign-extended)
        _ => unreachable!(),
    }
}
