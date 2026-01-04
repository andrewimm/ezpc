//! Shift and rotate instruction handlers (SHL, SHR, SAR, ROL, ROR, RCL, RCR)

use crate::cpu::decode::{DecodedInstruction, OperandType};
use crate::cpu::state::FlagOp;
use crate::cpu::Cpu;
use crate::memory::MemoryBus;

/// ROL - Rotate Left
/// Rotates the bits in the destination left by the specified count.
/// The leftmost bit is copied to the rightmost bit and to CF.
///
/// Flags: CF is set to the last bit rotated out
///        OF is set if count=1 and the sign bit changed
///        Other flags are undefined for count != 1
pub fn rol(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction, count: u8) {
    if count == 0 {
        return;
    }

    let value = cpu.read_operand(mem, &instr.dst);
    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    let (result, new_cf) = if is_byte {
        let val = value as u8;
        let count = count & 0x07; // Only low 3 bits for 8-bit
        if count == 0 {
            return;
        }
        let result = val.rotate_left(count as u32);
        let cf = result & 1; // Rightmost bit after rotation
        (result as u16, cf != 0)
    } else {
        let val = value;
        let count = count & 0x0F; // Only low 4 bits for 16-bit
        if count == 0 {
            return;
        }
        let result = val.rotate_left(count as u32);
        let cf = result & 1; // Rightmost bit after rotation
        (result, cf != 0)
    };

    cpu.write_operand(mem, &instr.dst, result);
    cpu.set_flag(Cpu::CF, new_cf);

    // OF is only defined for count=1
    if count & (if is_byte { 0x07 } else { 0x0F }) == 1 {
        let msb = if is_byte {
            (result & 0x80) != 0
        } else {
            (result & 0x8000) != 0
        };
        cpu.set_flag(Cpu::OF, msb != new_cf);
    }
}

/// ROR - Rotate Right
/// Rotates the bits in the destination right by the specified count.
/// The rightmost bit is copied to the leftmost bit and to CF.
///
/// Flags: CF is set to the last bit rotated out
///        OF is set if count=1 and the sign bit changed
pub fn ror(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction, count: u8) {
    if count == 0 {
        return;
    }

    let value = cpu.read_operand(mem, &instr.dst);
    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    let (result, new_cf) = if is_byte {
        let val = value as u8;
        let count = count & 0x07;
        if count == 0 {
            return;
        }
        let result = val.rotate_right(count as u32);
        let cf = (result & 0x80) != 0; // Leftmost bit after rotation
        (result as u16, cf)
    } else {
        let val = value;
        let count = count & 0x0F;
        if count == 0 {
            return;
        }
        let result = val.rotate_right(count as u32);
        let cf = (result & 0x8000) != 0; // Leftmost bit after rotation
        (result, cf)
    };

    cpu.write_operand(mem, &instr.dst, result);
    cpu.set_flag(Cpu::CF, new_cf);

    // OF is only defined for count=1
    if count & (if is_byte { 0x07 } else { 0x0F }) == 1 {
        let msb = if is_byte {
            (result & 0x80) != 0
        } else {
            (result & 0x8000) != 0
        };
        let next_msb = if is_byte {
            (result & 0x40) != 0
        } else {
            (result & 0x4000) != 0
        };
        cpu.set_flag(Cpu::OF, msb != next_msb);
    }
}

/// RCL - Rotate Through Carry Left
/// Rotates the bits in the destination and CF left by the specified count.
/// CF is treated as part of the value being rotated.
pub fn rcl(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction, count: u8) {
    if count == 0 {
        return;
    }

    let value = cpu.read_operand(mem, &instr.dst);
    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    let mut cf = cpu.get_flag(Cpu::CF);
    let mut result = value;

    if is_byte {
        let count = count & 0x1F; // Modulo 9 for 8-bit + CF
        for _ in 0..count {
            let new_cf = (result & 0x80) != 0;
            result = ((result << 1) & 0xFF) | (if cf { 1 } else { 0 });
            cf = new_cf;
        }
    } else {
        let count = count & 0x1F; // Modulo 17 for 16-bit + CF
        for _ in 0..count {
            let new_cf = (result & 0x8000) != 0;
            result = ((result << 1) & 0xFFFF) | (if cf { 1 } else { 0 });
            cf = new_cf;
        }
    }

    cpu.write_operand(mem, &instr.dst, result);
    cpu.set_flag(Cpu::CF, cf);

    // OF is only defined for count=1
    if count == 1 {
        let msb = if is_byte {
            (result & 0x80) != 0
        } else {
            (result & 0x8000) != 0
        };
        cpu.set_flag(Cpu::OF, msb != cf);
    }
}

/// RCR - Rotate Through Carry Right
/// Rotates the bits in the destination and CF right by the specified count.
/// CF is treated as part of the value being rotated.
pub fn rcr(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction, count: u8) {
    if count == 0 {
        return;
    }

    let value = cpu.read_operand(mem, &instr.dst);
    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    let mut cf = cpu.get_flag(Cpu::CF);
    let mut result = value;

    if is_byte {
        let count = count & 0x1F; // Modulo 9 for 8-bit + CF
        for _ in 0..count {
            let new_cf = (result & 1) != 0;
            result = (result >> 1) | (if cf { 0x80 } else { 0 });
            cf = new_cf;
        }
    } else {
        let count = count & 0x1F; // Modulo 17 for 16-bit + CF
        for _ in 0..count {
            let new_cf = (result & 1) != 0;
            result = (result >> 1) | (if cf { 0x8000 } else { 0 });
            cf = new_cf;
        }
    }

    cpu.write_operand(mem, &instr.dst, result);
    cpu.set_flag(Cpu::CF, cf);

    // OF is only defined for count=1
    if count == 1 {
        let msb = if is_byte {
            (result & 0x80) != 0
        } else {
            (result & 0x8000) != 0
        };
        let next_msb = if is_byte {
            (result & 0x40) != 0
        } else {
            (result & 0x4000) != 0
        };
        cpu.set_flag(Cpu::OF, msb != next_msb);
    }
}

/// SHL/SAL - Shift Left (Logical/Arithmetic)
/// Shifts the bits in the destination left by the specified count.
/// Zeros are shifted in from the right. The last bit shifted out goes to CF.
///
/// Flags: CF is set to the last bit shifted out
///        OF is set if count=1 and the sign bit changed
///        ZF, SF, PF are set according to the result
///        AF is undefined
pub fn shl(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction, count: u8) {
    if count == 0 {
        return;
    }

    let value = cpu.read_operand(mem, &instr.dst);
    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    let (result, new_cf) = if is_byte {
        let val = value as u8;
        let count = if count >= 8 { 8 } else { count };

        let cf = if count > 0 {
            // CF is the last bit shifted out
            if count <= 8 {
                (val >> (8 - count)) & 1 != 0
            } else {
                false
            }
        } else {
            false
        };

        let result = if count >= 8 { 0 } else { (val << count) & 0xFF };
        ((result as u16), cf)
    } else {
        let val = value;
        let count = if count >= 16 { 16 } else { count };

        let cf = if count > 0 {
            if count <= 16 {
                (val >> (16 - count)) & 1 != 0
            } else {
                false
            }
        } else {
            false
        };

        let result = if count >= 16 {
            0
        } else {
            (val << count) & 0xFFFF
        };
        (result, cf)
    };

    cpu.write_operand(mem, &instr.dst, result);
    cpu.set_flag(Cpu::CF, new_cf);

    // Set flags based on result
    if is_byte {
        cpu.set_lazy_flags(result as u32, FlagOp::And8);
    } else {
        cpu.set_lazy_flags(result as u32, FlagOp::And16);
    }
    cpu.clear_of_cf_af();
    cpu.set_flag(Cpu::CF, new_cf);

    // OF is only defined for count=1
    if count == 1 {
        let msb = if is_byte {
            (result & 0x80) != 0
        } else {
            (result & 0x8000) != 0
        };
        cpu.set_flag(Cpu::OF, msb != new_cf);
    }
}

/// SHR - Shift Right (Logical)
/// Shifts the bits in the destination right by the specified count.
/// Zeros are shifted in from the left. The last bit shifted out goes to CF.
///
/// Flags: CF is set to the last bit shifted out
///        OF is set to the MSB of the original value if count=1
///        ZF, SF, PF are set according to the result
///        AF is undefined
pub fn shr(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction, count: u8) {
    if count == 0 {
        return;
    }

    let value = cpu.read_operand(mem, &instr.dst);
    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    let original_msb = if is_byte {
        (value & 0x80) != 0
    } else {
        (value & 0x8000) != 0
    };

    let (result, new_cf) = if is_byte {
        let val = value as u8;
        let count = if count >= 8 { 8 } else { count };

        let cf = if count > 0 && count <= 8 {
            (val >> (count - 1)) & 1 != 0
        } else {
            false
        };

        let result = if count >= 8 { 0 } else { val >> count };
        (result as u16, cf)
    } else {
        let val = value;
        let count = if count >= 16 { 16 } else { count };

        let cf = if count > 0 && count <= 16 {
            (val >> (count - 1)) & 1 != 0
        } else {
            false
        };

        let result = if count >= 16 { 0 } else { val >> count };
        (result, cf)
    };

    cpu.write_operand(mem, &instr.dst, result);
    cpu.set_flag(Cpu::CF, new_cf);

    // Set flags based on result
    if is_byte {
        cpu.set_lazy_flags(result as u32, FlagOp::And8);
    } else {
        cpu.set_lazy_flags(result as u32, FlagOp::And16);
    }
    cpu.clear_of_cf_af();
    cpu.set_flag(Cpu::CF, new_cf);

    // OF is only defined for count=1
    if count == 1 {
        cpu.set_flag(Cpu::OF, original_msb);
    }
}

/// SAR - Shift Right (Arithmetic)
/// Shifts the bits in the destination right by the specified count.
/// The sign bit is preserved (shifted in from the left).
/// The last bit shifted out goes to CF.
///
/// Flags: CF is set to the last bit shifted out
///        OF is cleared if count=1 (sign bit doesn't change in SAR)
///        ZF, SF, PF are set according to the result
///        AF is undefined
pub fn sar(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction, count: u8) {
    if count == 0 {
        return;
    }

    let value = cpu.read_operand(mem, &instr.dst);
    let is_byte = instr.dst.op_type == OperandType::Reg8 || instr.dst.op_type == OperandType::Mem8;

    let (result, new_cf) = if is_byte {
        let val = value as i8;
        let count = if count >= 8 { 8 } else { count };

        let cf = if count > 0 && count <= 8 {
            ((val as u8) >> (count - 1)) & 1 != 0
        } else {
            false
        };

        // Arithmetic shift preserves sign
        let result = (val >> count) as u8;
        (result as u16, cf)
    } else {
        let val = value as i16;
        let count = if count >= 16 { 16 } else { count };

        let cf = if count > 0 && count <= 16 {
            ((val as u16) >> (count - 1)) & 1 != 0
        } else {
            false
        };

        // Arithmetic shift preserves sign
        let result = (val >> count) as u16;
        (result, cf)
    };

    cpu.write_operand(mem, &instr.dst, result);
    cpu.set_flag(Cpu::CF, new_cf);

    // Set flags based on result
    if is_byte {
        cpu.set_lazy_flags(result as u32, FlagOp::And8);
    } else {
        cpu.set_lazy_flags(result as u32, FlagOp::And16);
    }
    cpu.clear_of_cf_af();
    cpu.set_flag(Cpu::CF, new_cf);

    // OF is always cleared for SAR with count=1 (sign doesn't change)
    if count == 1 {
        cpu.set_flag(Cpu::OF, false);
    }
}

/// Group handler for 0xD0: Shift/rotate r/m8, 1
pub fn group_d0(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    // The reg field of the ModR/M byte determines the operation
    // This is encoded in the src operand's value field
    let operation = instr.src.value as u8;

    match operation {
        0 => rol(cpu, mem, instr, 1),
        1 => ror(cpu, mem, instr, 1),
        2 => rcl(cpu, mem, instr, 1),
        3 => rcr(cpu, mem, instr, 1),
        4 | 6 => shl(cpu, mem, instr, 1), // SHL/SAL (same operation)
        5 => shr(cpu, mem, instr, 1),
        7 => sar(cpu, mem, instr, 1),
        _ => unreachable!(),
    }
}

/// Group handler for 0xD1: Shift/rotate r/m16, 1
pub fn group_d1(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    // The reg field of the ModR/M byte determines the operation
    let operation = instr.src.value as u8;

    match operation {
        0 => rol(cpu, mem, instr, 1),
        1 => ror(cpu, mem, instr, 1),
        2 => rcl(cpu, mem, instr, 1),
        3 => rcr(cpu, mem, instr, 1),
        4 | 6 => shl(cpu, mem, instr, 1), // SHL/SAL (same operation)
        5 => shr(cpu, mem, instr, 1),
        7 => sar(cpu, mem, instr, 1),
        _ => unreachable!(),
    }
}

/// Group handler for 0xD2: Shift/rotate r/m8, CL
pub fn group_d2(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    // The reg field of the ModR/M byte determines the operation
    let operation = instr.src.value as u8;
    let count = cpu.read_reg8(1); // CL register

    match operation {
        0 => rol(cpu, mem, instr, count),
        1 => ror(cpu, mem, instr, count),
        2 => rcl(cpu, mem, instr, count),
        3 => rcr(cpu, mem, instr, count),
        4 | 6 => shl(cpu, mem, instr, count), // SHL/SAL (same operation)
        5 => shr(cpu, mem, instr, count),
        7 => sar(cpu, mem, instr, count),
        _ => unreachable!(),
    }
}

/// Group handler for 0xD3: Shift/rotate r/m16, CL
pub fn group_d3(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    // The reg field of the ModR/M byte determines the operation
    let operation = instr.src.value as u8;
    let count = cpu.read_reg8(1); // CL register

    match operation {
        0 => rol(cpu, mem, instr, count),
        1 => ror(cpu, mem, instr, count),
        2 => rcl(cpu, mem, instr, count),
        3 => rcr(cpu, mem, instr, count),
        4 | 6 => shl(cpu, mem, instr, count), // SHL/SAL (same operation)
        5 => shr(cpu, mem, instr, count),
        7 => sar(cpu, mem, instr, count),
        _ => unreachable!(),
    }
}
