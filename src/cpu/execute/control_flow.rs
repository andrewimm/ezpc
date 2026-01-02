//! Control flow instruction handlers (JMP, CALL, RET, Jcc, etc.)

use crate::cpu::Cpu;
use crate::cpu::decode::DecodedInstruction;
use crate::cpu::execute::{arithmetic, stack};
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

/// CALL near relative - Call procedure with 16-bit relative offset
/// Opcode: 0xE8
///
/// Stack operation: PUSH IP, then IP = IP + rel16
pub fn call_near(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    use super::stack::push_word;

    // Push return address (current IP)
    let return_addr = cpu.ip;
    push_word(cpu, mem, return_addr);

    // Jump to target (IP + offset)
    let offset = instr.src.value as i16;
    cpu.ip = cpu.ip.wrapping_add(offset as u16);
}

/// CALL far direct - Call procedure in another segment
/// Opcode: 0x9A
///
/// Stack operation: PUSH CS, PUSH IP, then CS:IP = new_seg:new_offset
pub fn call_far(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    use super::stack::push_word;

    // Push return address (CS:IP)
    let return_cs = cpu.read_seg(1); // CS
    let return_ip = cpu.ip;
    push_word(cpu, mem, return_cs);
    push_word(cpu, mem, return_ip);

    // Load new CS:IP from operands
    // src contains offset, dst contains segment
    let new_ip = instr.src.value;
    let new_cs = instr.dst.value;
    cpu.write_seg(1, new_cs); // CS
    cpu.ip = new_ip;
}

/// CALL r/m16 near indirect - Call procedure at address in register/memory
/// Opcode: 0xFF /2
///
/// Stack operation: PUSH IP, then IP = r/m16
pub fn call_rm16(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    use super::stack::push_word;

    // Read target address from operand
    let target = cpu.read_operand(mem, &instr.dst);

    // Push return address
    let return_addr = cpu.ip;
    push_word(cpu, mem, return_addr);

    // Jump to target
    cpu.ip = target;
}

/// CALL m16:16 far indirect - Call far procedure at address in memory
/// Opcode: 0xFF /3
///
/// Stack operation: PUSH CS, PUSH IP, then CS:IP = [m16:16]
pub fn call_m16_16(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    use super::stack::push_word;
    use crate::cpu::decode::operands::OperandType;

    // For far indirect calls, we need to read both offset and segment from memory
    // The operand gives us the memory address where offset:segment is stored
    // Memory layout: [offset_low, offset_high, segment_low, segment_high]

    // Calculate the effective address
    let (seg_idx, ea) = match instr.dst.op_type {
        OperandType::Mem16 => {
            let base_index = instr.dst.value as u8;
            cpu.calculate_ea_from_operand(&instr.dst, base_index)
        }
        _ => panic!("CALL m16:16 requires memory operand"),
    };

    // Get the actual segment (considering segment overrides)
    let segment = if instr.dst.segment != 0xFF {
        cpu.read_seg(instr.dst.segment)
    } else {
        cpu.segments[seg_idx as usize]
    };

    // Read offset and segment from memory
    let new_ip = cpu.read_mem16(mem, segment, ea);
    let new_cs = cpu.read_mem16(mem, segment, ea.wrapping_add(2));

    // Push return address (CS:IP)
    let return_cs = cpu.read_seg(1); // CS
    let return_ip = cpu.ip;
    push_word(cpu, mem, return_cs);
    push_word(cpu, mem, return_ip);

    // Load new CS:IP
    cpu.write_seg(1, new_cs); // CS
    cpu.ip = new_ip;
}

/// Group handler for opcode 0xFF
/// Handles INC/DEC/CALL/JMP/PUSH r/m16 based on reg field
pub fn group_ff(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    // The reg field is stored in the value field of dst operand during group decoding
    let reg = (instr.dst.value >> 8) as u8; // High byte stores the reg field

    match reg {
        0 => arithmetic::inc_rm(cpu, mem, instr), // INC r/m16
        1 => arithmetic::dec_rm(cpu, mem, instr), // DEC r/m16
        2 => call_rm16(cpu, mem, instr),          // CALL r/m16 (near)
        3 => call_m16_16(cpu, mem, instr),        // CALL m16:16 (far)
        4 => jmp_rm16(cpu, mem, instr),           // JMP r/m16 (near)
        5 => jmp_m16_16(cpu, mem, instr),         // JMP m16:16 (far)
        6 => stack::push_rm16(cpu, mem, instr),   // PUSH r/m16
        _ => panic!("Invalid reg field {} for opcode 0xFF", reg),
    }
}

/// JMP r/m16 near indirect - Jump to address in register/memory
/// Part of opcode 0xFF /4
pub fn jmp_rm16(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let target = cpu.read_operand(mem, &instr.dst);
    cpu.ip = target;
}

/// JMP m16:16 far indirect - Jump to far address in memory
/// Part of opcode 0xFF /5
pub fn jmp_m16_16(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    use crate::cpu::decode::operands::OperandType;

    // Calculate the effective address
    let (seg_idx, ea) = match instr.dst.op_type {
        OperandType::Mem16 => {
            let base_index = instr.dst.value as u8;
            cpu.calculate_ea_from_operand(&instr.dst, base_index)
        }
        _ => panic!("JMP m16:16 requires memory operand"),
    };

    // Get the actual segment (considering segment overrides)
    let segment = if instr.dst.segment != 0xFF {
        cpu.read_seg(instr.dst.segment)
    } else {
        cpu.segments[seg_idx as usize]
    };

    // Read offset and segment from memory
    let new_ip = cpu.read_mem16(mem, segment, ea);
    let new_cs = cpu.read_mem16(mem, segment, ea.wrapping_add(2));

    // Load new CS:IP
    cpu.write_seg(1, new_cs); // CS
    cpu.ip = new_ip;
}

/// RET near - Return from near procedure
/// Opcode: 0xC3
///
/// Stack operation: IP = POP()
pub fn ret_near(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    use super::stack::pop_word;
    cpu.ip = pop_word(cpu, mem);
}

/// RET near imm16 - Return from near procedure with stack cleanup
/// Opcode: 0xC2
///
/// Stack operation: IP = POP(), then SP = SP + imm16
pub fn ret_near_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    use super::stack::pop_word;

    // Pop return address
    cpu.ip = pop_word(cpu, mem);

    // Clean up stack by adding immediate to SP
    let cleanup = instr.src.value;
    cpu.regs[4] = cpu.regs[4].wrapping_add(cleanup); // SP
}

/// RETF - Return from far procedure
/// Opcode: 0xCB
///
/// Stack operation: IP = POP(), CS = POP()
pub fn ret_far(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    use super::stack::pop_word;

    // Pop IP first (pushed last), then CS
    cpu.ip = pop_word(cpu, mem);
    let new_cs = pop_word(cpu, mem);
    cpu.write_seg(1, new_cs); // CS
}

/// RETF imm16 - Return from far procedure with stack cleanup
/// Opcode: 0xCA
///
/// Stack operation: IP = POP(), CS = POP(), then SP = SP + imm16
pub fn ret_far_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    use super::stack::pop_word;

    // Pop IP and CS
    cpu.ip = pop_word(cpu, mem);
    let new_cs = pop_word(cpu, mem);
    cpu.write_seg(1, new_cs); // CS

    // Clean up stack by adding immediate to SP
    let cleanup = instr.src.value;
    cpu.regs[4] = cpu.regs[4].wrapping_add(cleanup); // SP
}
