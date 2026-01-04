//! String operation instruction handlers
//!
//! String instructions operate on data blocks and auto-increment/decrement
//! SI and DI registers based on the direction flag (DF).
//!
//! When combined with REP prefixes, these instructions repeat while CX != 0.

use crate::cpu::decode::DecodedInstruction;
use crate::cpu::state::RepeatPrefix;
use crate::cpu::Cpu;
use crate::memory::MemoryBus;

/// STOSB (0xAA) - Store AL to [ES:DI], then DI += 1 or DI -= 1
///
/// Stores the byte in AL to ES:DI, then increments or decrements DI
/// based on the direction flag.
pub fn stosb(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    // Store AL to ES:DI
    let es = cpu.read_seg(0);
    let di = cpu.read_reg16(7);
    let al = cpu.read_reg8(0);
    cpu.write_mem8(mem, es, di, al);

    // Update DI based on direction flag
    let new_di = if cpu.get_flag(Cpu::DF) {
        di.wrapping_sub(1)
    } else {
        di.wrapping_add(1)
    };
    cpu.write_reg16(7, new_di);

    // Handle REP prefix
    handle_rep(cpu);
}

/// STOSW (0xAB) - Store AX to [ES:DI], then DI += 2 or DI -= 2
///
/// Stores the word in AX to ES:DI, then increments or decrements DI by 2
/// based on the direction flag.
pub fn stosw(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    // Store AX to ES:DI
    let es = cpu.read_seg(0);
    let di = cpu.read_reg16(7);
    let ax = cpu.read_reg16(0);
    cpu.write_mem16(mem, es, di, ax);

    // Update DI based on direction flag
    let new_di = if cpu.get_flag(Cpu::DF) {
        di.wrapping_sub(2)
    } else {
        di.wrapping_add(2)
    };
    cpu.write_reg16(7, new_di);

    // Handle REP prefix
    handle_rep(cpu);
}

/// MOVSB (0xA4) - Move byte from [DS:SI] to [ES:DI], update SI and DI
///
/// Copies a byte from DS:SI to ES:DI, then increments or decrements both
/// SI and DI based on the direction flag.
pub fn movsb(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    // Read from DS:SI (or segment override)
    let ds = cpu
        .segment_override
        .map(|s| cpu.read_seg(s))
        .unwrap_or_else(|| cpu.read_seg(3));
    let si = cpu.read_reg16(6);
    let byte = cpu.read_mem8(mem, ds, si);

    // Write to ES:DI
    let es = cpu.read_seg(0);
    let di = cpu.read_reg16(7);
    cpu.write_mem8(mem, es, di, byte);

    // Update SI and DI based on direction flag
    if cpu.get_flag(Cpu::DF) {
        cpu.write_reg16(6, si.wrapping_sub(1));
        cpu.write_reg16(7, di.wrapping_sub(1));
    } else {
        cpu.write_reg16(6, si.wrapping_add(1));
        cpu.write_reg16(7, di.wrapping_add(1));
    }

    // Handle REP prefix
    handle_rep(cpu);
}

/// MOVSW (0xA5) - Move word from [DS:SI] to [ES:DI], update SI and DI
///
/// Copies a word from DS:SI to ES:DI, then increments or decrements both
/// SI and DI by 2 based on the direction flag.
pub fn movsw(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    // Read from DS:SI (or segment override)
    let ds = cpu
        .segment_override
        .map(|s| cpu.read_seg(s))
        .unwrap_or_else(|| cpu.read_seg(3));
    let si = cpu.read_reg16(6);
    let word = cpu.read_mem16(mem, ds, si);

    // Write to ES:DI
    let es = cpu.read_seg(0);
    let di = cpu.read_reg16(7);
    cpu.write_mem16(mem, es, di, word);

    // Update SI and DI based on direction flag
    if cpu.get_flag(Cpu::DF) {
        cpu.write_reg16(6, si.wrapping_sub(2));
        cpu.write_reg16(7, di.wrapping_sub(2));
    } else {
        cpu.write_reg16(6, si.wrapping_add(2));
        cpu.write_reg16(7, di.wrapping_add(2));
    }

    // Handle REP prefix
    handle_rep(cpu);
}

/// LODSB (0xAC) - Load byte from [DS:SI] into AL, then SI += 1 or SI -= 1
///
/// Loads a byte from DS:SI into AL, then increments or decrements SI
/// based on the direction flag.
pub fn lodsb(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    // Read from DS:SI (or segment override)
    let ds = cpu
        .segment_override
        .map(|s| cpu.read_seg(s))
        .unwrap_or_else(|| cpu.read_seg(3));
    let si = cpu.read_reg16(6);
    let byte = cpu.read_mem8(mem, ds, si);

    // Store in AL
    cpu.write_reg8(0, byte);

    // Update SI based on direction flag
    let new_si = if cpu.get_flag(Cpu::DF) {
        si.wrapping_sub(1)
    } else {
        si.wrapping_add(1)
    };
    cpu.write_reg16(6, new_si);

    // Handle REP prefix
    handle_rep(cpu);
}

/// LODSW (0xAD) - Load word from [DS:SI] into AX, then SI += 2 or SI -= 2
///
/// Loads a word from DS:SI into AX, then increments or decrements SI by 2
/// based on the direction flag.
pub fn lodsw(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    // Read from DS:SI (or segment override)
    let ds = cpu
        .segment_override
        .map(|s| cpu.read_seg(s))
        .unwrap_or_else(|| cpu.read_seg(3));
    let si = cpu.read_reg16(6);
    let word = cpu.read_mem16(mem, ds, si);

    // Store in AX
    cpu.write_reg16(0, word);

    // Update SI based on direction flag
    let new_si = if cpu.get_flag(Cpu::DF) {
        si.wrapping_sub(2)
    } else {
        si.wrapping_add(2)
    };
    cpu.write_reg16(6, new_si);

    // Handle REP prefix
    handle_rep(cpu);
}

/// CMPSB (0xA6) - Compare bytes at [DS:SI] and [ES:DI], update SI and DI
///
/// Compares byte at DS:SI with byte at ES:DI by subtracting and setting flags,
/// then increments or decrements SI and DI based on the direction flag.
pub fn cmpsb(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    // Read from DS:SI (or segment override)
    let ds = cpu
        .segment_override
        .map(|s| cpu.read_seg(s))
        .unwrap_or_else(|| cpu.read_seg(3));
    let si = cpu.read_reg16(6);
    let byte1 = cpu.read_mem8(mem, ds, si);

    // Read from ES:DI
    let es = cpu.read_seg(0);
    let di = cpu.read_reg16(7);
    let byte2 = cpu.read_mem8(mem, es, di);

    // Perform subtraction and set flags (like CMP)
    let result = (byte1 as u32).wrapping_sub(byte2 as u32);
    cpu.set_sub8_of_af(byte1, byte2, result);
    cpu.set_lazy_flags(result, crate::cpu::state::FlagOp::Sub8);

    // Update SI and DI based on direction flag
    if cpu.get_flag(Cpu::DF) {
        cpu.write_reg16(6, si.wrapping_sub(1));
        cpu.write_reg16(7, di.wrapping_sub(1));
    } else {
        cpu.write_reg16(6, si.wrapping_add(1));
        cpu.write_reg16(7, di.wrapping_add(1));
    }

    // Handle REPE/REPNE prefix (checking ZF)
    handle_rep_conditional(cpu);
}

/// CMPSW (0xA7) - Compare words at [DS:SI] and [ES:DI], update SI and DI
///
/// Compares word at DS:SI with word at ES:DI by subtracting and setting flags,
/// then increments or decrements SI and DI by 2 based on the direction flag.
pub fn cmpsw(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    // Read from DS:SI (or segment override)
    let ds = cpu
        .segment_override
        .map(|s| cpu.read_seg(s))
        .unwrap_or_else(|| cpu.read_seg(3));
    let si = cpu.read_reg16(6);
    let word1 = cpu.read_mem16(mem, ds, si);

    // Read from ES:DI
    let es = cpu.read_seg(0);
    let di = cpu.read_reg16(7);
    let word2 = cpu.read_mem16(mem, es, di);

    // Perform subtraction and set flags (like CMP)
    let result = (word1 as u32).wrapping_sub(word2 as u32);
    cpu.set_sub16_of_af(word1, word2, result);
    cpu.set_lazy_flags(result, crate::cpu::state::FlagOp::Sub16);

    // Update SI and DI based on direction flag
    if cpu.get_flag(Cpu::DF) {
        cpu.write_reg16(6, si.wrapping_sub(2));
        cpu.write_reg16(7, di.wrapping_sub(2));
    } else {
        cpu.write_reg16(6, si.wrapping_add(2));
        cpu.write_reg16(7, di.wrapping_add(2));
    }

    // Handle REPE/REPNE prefix (checking ZF)
    handle_rep_conditional(cpu);
}

/// SCASB (0xAE) - Scan byte: compare AL with [ES:DI], update DI
///
/// Compares AL with byte at ES:DI by subtracting and setting flags,
/// then increments or decrements DI based on the direction flag.
pub fn scasb(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    // Read AL
    let al = cpu.read_reg8(0);

    // Read from ES:DI
    let es = cpu.read_seg(0);
    let di = cpu.read_reg16(7);
    let byte = cpu.read_mem8(mem, es, di);

    // Perform subtraction and set flags (like CMP)
    let result = (al as u32).wrapping_sub(byte as u32);
    cpu.set_sub8_of_af(al, byte, result);
    cpu.set_lazy_flags(result, crate::cpu::state::FlagOp::Sub8);

    // Update DI based on direction flag
    let new_di = if cpu.get_flag(Cpu::DF) {
        di.wrapping_sub(1)
    } else {
        di.wrapping_add(1)
    };
    cpu.write_reg16(7, new_di);

    // Handle REPE/REPNE prefix (checking ZF)
    handle_rep_conditional(cpu);
}

/// SCASW (0xAF) - Scan word: compare AX with [ES:DI], update DI
///
/// Compares AX with word at ES:DI by subtracting and setting flags,
/// then increments or decrements DI by 2 based on the direction flag.
pub fn scasw(cpu: &mut Cpu, mem: &mut MemoryBus, _instr: &DecodedInstruction) {
    // Read AX
    let ax = cpu.read_reg16(0);

    // Read from ES:DI
    let es = cpu.read_seg(0);
    let di = cpu.read_reg16(7);
    let word = cpu.read_mem16(mem, es, di);

    // Perform subtraction and set flags (like CMP)
    let result = (ax as u32).wrapping_sub(word as u32);
    cpu.set_sub16_of_af(ax, word, result);
    cpu.set_lazy_flags(result, crate::cpu::state::FlagOp::Sub16);

    // Update DI based on direction flag
    let new_di = if cpu.get_flag(Cpu::DF) {
        di.wrapping_sub(2)
    } else {
        di.wrapping_add(2)
    };
    cpu.write_reg16(7, new_di);

    // Handle REPE/REPNE prefix (checking ZF)
    handle_rep_conditional(cpu);
}

/// Helper function to handle REP prefix for string operations
///
/// If a REP prefix is active:
/// 1. Decrement CX
/// 2. If CX != 0, jump back to the REP prefix to repeat
/// Used for MOVS, STOS, LODS (unconditional repeat)
fn handle_rep(cpu: &mut Cpu) {
    if cpu.repeat_prefix != RepeatPrefix::None {
        // Decrement CX
        let cx = cpu.read_reg16(1);
        let new_cx = cx.wrapping_sub(1);
        cpu.write_reg16(1, new_cx);

        // If CX != 0, jump back to repeat
        if new_cx != 0 {
            cpu.ip = cpu.repeat_ip;
        }
    }
}

/// Helper function to handle REPE/REPNE prefix for comparison string operations
///
/// For CMPS and SCAS instructions:
/// 1. Decrement CX
/// 2. Check continuation condition based on prefix type and ZF
/// 3. If conditions met, jump back to repeat
fn handle_rep_conditional(cpu: &mut Cpu) {
    match cpu.repeat_prefix {
        RepeatPrefix::None => {
            // No prefix, nothing to do
        }
        RepeatPrefix::Rep => {
            // REPE: Repeat while ZF=1 (equal) and CX != 0
            let cx = cpu.read_reg16(1);
            let new_cx = cx.wrapping_sub(1);
            cpu.write_reg16(1, new_cx);

            let zf = cpu.get_flag(Cpu::ZF);
            if new_cx != 0 && zf {
                cpu.ip = cpu.repeat_ip;
            }
        }
        RepeatPrefix::RepNe => {
            // REPNE: Repeat while ZF=0 (not equal) and CX != 0
            let cx = cpu.read_reg16(1);
            let new_cx = cx.wrapping_sub(1);
            cpu.write_reg16(1, new_cx);

            let zf = cpu.get_flag(Cpu::ZF);
            if new_cx != 0 && !zf {
                cpu.ip = cpu.repeat_ip;
            }
        }
    }
}
