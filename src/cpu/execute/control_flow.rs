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

/// JO - Jump if overflow
/// Opcode: 0x70
///
/// If OF=1 then IP = IP + rel8
pub fn jo(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if cpu.get_flag(Cpu::OF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// JNO - Jump if not overflow
/// Opcode: 0x71
///
/// If OF=0 then IP = IP + rel8
pub fn jno(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if !cpu.get_flag(Cpu::OF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// JBE/JNA - Jump if below or equal/not above
/// Opcode: 0x76
///
/// If CF=1 or ZF=1 then IP = IP + rel8
pub fn jbe(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if cpu.get_flag(Cpu::CF) || cpu.get_flag(Cpu::ZF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// JA/JNBE - Jump if above/not below or equal
/// Opcode: 0x77
///
/// If CF=0 and ZF=0 then IP = IP + rel8
pub fn ja(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if !cpu.get_flag(Cpu::CF) && !cpu.get_flag(Cpu::ZF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// JP/JPE - Jump if parity/parity even
/// Opcode: 0x7A
///
/// If PF=1 then IP = IP + rel8
pub fn jp(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if cpu.get_flag(Cpu::PF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// JNP/JPO - Jump if not parity/parity odd
/// Opcode: 0x7B
///
/// If PF=0 then IP = IP + rel8
pub fn jnp(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if !cpu.get_flag(Cpu::PF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// JL/JNGE - Jump if less/not greater or equal
/// Opcode: 0x7C
///
/// If SF != OF then IP = IP + rel8
pub fn jl(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if cpu.get_flag(Cpu::SF) != cpu.get_flag(Cpu::OF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// JGE/JNL - Jump if greater or equal/not less
/// Opcode: 0x7D
///
/// If SF = OF then IP = IP + rel8
pub fn jge(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if cpu.get_flag(Cpu::SF) == cpu.get_flag(Cpu::OF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// JLE/JNG - Jump if less or equal/not greater
/// Opcode: 0x7E
///
/// If ZF=1 or SF != OF then IP = IP + rel8
pub fn jle(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if cpu.get_flag(Cpu::ZF) || (cpu.get_flag(Cpu::SF) != cpu.get_flag(Cpu::OF)) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// JG/JNLE - Jump if greater/not less or equal
/// Opcode: 0x7F
///
/// If ZF=0 and SF = OF then IP = IP + rel8
pub fn jg(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    if !cpu.get_flag(Cpu::ZF) && (cpu.get_flag(Cpu::SF) == cpu.get_flag(Cpu::OF)) {
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

/// LOOP - Loop while CX not zero
/// Opcode: 0xE2
///
/// Decrements CX and jumps if CX != 0
/// If CX-1 != 0 then IP = IP + rel8
pub fn loop_rel8(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    // Decrement CX
    cpu.regs[1] = cpu.regs[1].wrapping_sub(1); // CX

    // Jump if CX != 0
    if cpu.regs[1] != 0 {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// LOOPE/LOOPZ - Loop while CX not zero and ZF=1
/// Opcode: 0xE1
///
/// Decrements CX and jumps if CX != 0 AND ZF = 1
/// If CX-1 != 0 and ZF=1 then IP = IP + rel8
pub fn loope(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    // Decrement CX
    cpu.regs[1] = cpu.regs[1].wrapping_sub(1); // CX

    // Jump if CX != 0 AND ZF = 1
    if cpu.regs[1] != 0 && cpu.get_flag(Cpu::ZF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// LOOPNE/LOOPNZ - Loop while CX not zero and ZF=0
/// Opcode: 0xE0
///
/// Decrements CX and jumps if CX != 0 AND ZF = 0
/// If CX-1 != 0 and ZF=0 then IP = IP + rel8
pub fn loopne(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    // Decrement CX
    cpu.regs[1] = cpu.regs[1].wrapping_sub(1); // CX

    // Jump if CX != 0 AND ZF = 0
    if cpu.regs[1] != 0 && !cpu.get_flag(Cpu::ZF) {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// JCXZ - Jump if CX is zero
/// Opcode: 0xE3
///
/// Jumps if CX = 0 (does not modify CX)
/// If CX=0 then IP = IP + rel8
pub fn jcxz(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    // Jump if CX = 0
    if cpu.regs[1] == 0 {
        let offset = instr.src.value as i16;
        cpu.ip = cpu.ip.wrapping_add(offset as u16);
    }
}

/// Common interrupt entry sequence for both software and hardware interrupts
///
/// Stack operation: PUSH FLAGS, PUSH CS, PUSH IP, then load CS:IP from IVT
/// The interrupt vector table (IVT) is located at 0000:0000
/// Each entry is 4 bytes: offset (word) followed by segment (word)
/// Vector for interrupt n is at address n*4
pub(crate) fn enter_interrupt(cpu: &mut Cpu, mem: &mut MemoryBus, vector: u8) {
    use super::stack::push_word;

    // Push FLAGS register
    let flags = cpu.get_flags();
    push_word(cpu, mem, flags);

    // Clear TF and IF flags (disable interrupts during handler)
    cpu.set_flag(Cpu::TF, false);
    cpu.set_flag(Cpu::IF, false);

    // Push CS
    let return_cs = cpu.read_seg(1); // CS
    push_word(cpu, mem, return_cs);

    // Push IP (return address)
    let return_ip = cpu.ip;
    push_word(cpu, mem, return_ip);

    // Load new CS:IP from interrupt vector table
    // IVT is at 0000:0000, each entry is 4 bytes
    // Entry n is at address n*4: [offset_low, offset_high, segment_low, segment_high]
    let ivt_addr = (vector as u32) * 4;
    let new_ip = mem.read_u16(ivt_addr);
    let new_cs = mem.read_u16(ivt_addr + 2);

    // Set new CS:IP
    cpu.write_seg(1, new_cs); // CS
    cpu.ip = new_ip;
}

/// INT n - Software interrupt with 8-bit interrupt number
/// Opcode: 0xCD
///
/// Stack operation: PUSH FLAGS, PUSH CS, PUSH IP, then load CS:IP from IVT
/// The interrupt vector table (IVT) is located at 0000:0000
/// Each entry is 4 bytes: offset (word) followed by segment (word)
/// Vector for interrupt n is at address n*4
pub fn int_n(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    // Get interrupt number from immediate operand
    let int_num = instr.src.value as u8;

    // Use common interrupt entry sequence
    enter_interrupt(cpu, mem, int_num);
}

/// INT3 - Breakpoint interrupt
/// Opcode: 0xCC
///
/// This is a special 1-byte form of INT 3, commonly used for breakpoints
/// Behavior is identical to INT 3 but encoded in a single byte
pub fn int3(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    // Use common interrupt entry sequence with vector 3
    enter_interrupt(cpu, mem, 3);
}

/// IRET - Return from interrupt
/// Opcode: 0xCF
///
/// Stack operation: IP = POP(), CS = POP(), FLAGS = POP()
pub fn iret(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    use super::stack::pop_word;

    // Pop IP first (pushed last), then CS, then FLAGS
    cpu.ip = pop_word(cpu, mem);
    let new_cs = pop_word(cpu, mem);
    cpu.write_seg(1, new_cs); // CS

    // Pop and restore FLAGS
    let flags = pop_word(cpu, mem);
    cpu.set_flags(flags);
}
