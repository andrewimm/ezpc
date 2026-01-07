//! 8088 Instruction Timing Tables
//!
//! Cycle counts based on Intel 8088 documentation.
//! All values are in clock cycles at 4.77 MHz.
//!
//! The 8088 timing model consists of:
//! - Base cycles: Fixed cost per opcode (simplest variant)
//! - EA cycles: Added for memory addressing modes
//! - Transfer penalty: 8088 has 8-bit bus, 16-bit transfers cost extra
//! - Segment override: +2 cycles when present

use crate::cpu::decode::{Operand, OperandType};

/// Base cycle counts for each opcode
///
/// These are the minimum cycles for each instruction (typically reg-reg variant).
/// Memory operands add EA calculation cycles and transfer penalties.
///
/// Special values:
/// - 0: Invalid/unimplemented opcode or prefix (no cycles consumed by prefix itself)
/// - Values represent reg-reg timing where applicable
///
/// For group opcodes (0x80-0x83, 0xF6-0xF7, 0xFE-0xFF), we store the most common
/// timing and handlers adjust for specific operations.
pub static BASE_CYCLES: [u8; 256] = [
    // 0x00-0x0F: ADD, OR, PUSH ES, POP ES
    3, 3, 3, 3, 4, 4, 14, 12, // ADD variants, PUSH/POP ES
    3, 3, 3, 3, 4, 4, 14, 0, // OR variants, PUSH CS, 0x0F invalid
    // 0x10-0x1F: ADC, SBB, PUSH SS, POP SS, PUSH DS, POP DS
    3, 3, 3, 3, 4, 4, 14, 12, // ADC variants, PUSH/POP SS
    3, 3, 3, 3, 4, 4, 14, 12, // SBB variants, PUSH/POP DS
    // 0x20-0x2F: AND, DAA, SUB, DAS
    3, 3, 3, 3, 4, 4, 0, 4, // AND variants, ES prefix, DAA
    3, 3, 3, 3, 4, 4, 0, 4, // SUB variants, CS prefix, DAS
    // 0x30-0x3F: XOR, AAA, CMP, AAS
    3, 3, 3, 3, 4, 4, 0, 4, // XOR variants, SS prefix, AAA
    3, 3, 3, 3, 4, 4, 0, 4, // CMP variants, DS prefix, AAS
    // 0x40-0x4F: INC/DEC r16
    2, 2, 2, 2, 2, 2, 2, 2, // INC r16
    2, 2, 2, 2, 2, 2, 2, 2, // DEC r16
    // 0x50-0x5F: PUSH/POP r16
    15, 15, 15, 15, 15, 15, 15, 15, // PUSH r16
    12, 12, 12, 12, 12, 12, 12, 12, // POP r16
    // 0x60-0x6F: Invalid on 8088
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    // 0x70-0x7F: Jcc short (4 not taken, 16 taken - handlers adjust)
    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
    // 0x80-0x8F: Group arithmetic, TEST, XCHG, MOV, LEA
    4, 4, 4, 4, 5, 5, 4, 4, // Groups 0x80-83, TEST, XCHG
    2, 2, 2, 2, 2, 2, 2, 0, // MOV r/m,r and r,r/m, MOV sreg, LEA, POP r/m
    // 0x90-0x9F: NOP, XCHG AX, CBW, CWD, CALL far, WAIT, PUSHF, POPF, SAHF, LAHF
    3, 3, 3, 3, 3, 3, 3, 3, // NOP, XCHG AX,r16
    2, 5, 36, 0, 14, 12, 4, 4, // CBW, CWD, CALL far, WAIT, PUSHF, POPF, SAHF, LAHF
    // 0xA0-0xAF: MOV moffs, string ops
    14, 14, 14, 14, 18, 26, 22, 30, // MOV moffs (14), MOVSB/W, CMPSB/W
    4, 4, 11, 15, 12, 16, 15, 19, // TEST acc,imm, STOSB/W, LODSB/W, SCASB/W
    // 0xB0-0xBF: MOV r, imm
    4, 4, 4, 4, 4, 4, 4, 4, // MOV r8, imm8
    4, 4, 4, 4, 4, 4, 4, 4, // MOV r16, imm16
    // 0xC0-0xCF: Shifts (invalid), RET, LES, LDS, MOV r/m,imm, INT, IRET
    0, 0, 24, 20, 24, 24, 4, 4, // Invalid, RET imm, RET, LES, LDS, MOV r/m,imm
    0, 0, 33, 34, 52, 51, 0, 44, // Invalid, RETF imm, RETF, INT 3, INT n, INTO, IRET
    // 0xD0-0xDF: Shifts, AAM, AAD, XLAT, ESC (FPU)
    2, 2, 8, 8, 83, 60, 0, 11, // Shift by 1, Shift by CL, AAM, AAD, SALC, XLAT
    0, 0, 0, 0, 0, 0, 0, 0, // ESC (FPU) - not implemented
    // 0xE0-0xEF: LOOP, IN, OUT, CALL, JMP
    // LOOP family uses not-taken timing as base (like Jcc), handlers add extra for taken
    5, 5, 5, 6, 10, 14, 10, 14, // LOOPNE, LOOPE, LOOP, JCXZ, IN imm, OUT imm
    23, 15, 15, 15, 8, 12, 8, 12, // CALL near, JMP near, JMP far, JMP short, IN DX, OUT DX
    // 0xF0-0xFF: LOCK, REP, HLT, CMC, Groups, Flags
    0, 0, 0, 0, 2, 2, 5, 5, // LOCK, INT1, REPNE, REP, HLT, CMC, Group F6, Group F7
    2, 2, 2, 2, 2, 2, 3, 0, // CLC, STC, CLI, STI, CLD, STD, Group FE, Group FF
];

/// EA (Effective Address) calculation cycle costs
///
/// Index by r/m field when mod != 11 (register direct):
/// - 0: [BX+SI] = 7 cycles
/// - 1: [BX+DI] = 8 cycles
/// - 2: [BP+SI] = 8 cycles
/// - 3: [BP+DI] = 7 cycles
/// - 4: [SI] = 5 cycles
/// - 5: [DI] = 5 cycles
/// - 6: [BP] = 5 cycles (or direct address if mod=00, which is 6 cycles)
/// - 7: [BX] = 5 cycles
pub static EA_CYCLES: [u8; 8] = [
    7, // [BX+SI]
    8, // [BX+DI]
    8, // [BP+SI]
    7, // [BP+DI]
    5, // [SI]
    5, // [DI]
    5, // [BP] (with displacement)
    5, // [BX]
];

/// Direct address [disp16] EA calculation cycles
pub const DIRECT_ADDRESS_CYCLES: u8 = 6;

/// Additional cycles when displacement is present
pub const DISP_CYCLES: u8 = 4;

/// Segment override prefix adds 2 cycles
pub const SEGMENT_OVERRIDE_CYCLES: u8 = 2;

/// 16-bit memory transfer penalty on 8088's 8-bit bus
/// Each 16-bit memory access requires an extra bus cycle
pub const WORD_TRANSFER_PENALTY: u8 = 4;

/// Calculate EA cycles for a memory operand
///
/// Returns 0 for register operands, calculates EA cycles for memory operands
/// based on addressing mode.
#[inline(always)]
pub fn calculate_ea_cycles(operand: &Operand) -> u8 {
    match operand.op_type {
        // Register and immediate operands have no EA cost
        OperandType::Reg8
        | OperandType::Reg16
        | OperandType::SegReg
        | OperandType::Imm8
        | OperandType::Imm16
        | OperandType::None
        | OperandType::Rel8
        | OperandType::Rel16 => 0,

        // Memory operands require EA calculation
        OperandType::Mem8 | OperandType::Mem16 => {
            let base_index = (operand.value & 0xFF) as u8;

            if base_index == 0xFF {
                // Direct address [disp16]
                DIRECT_ADDRESS_CYCLES
            } else if base_index < 8 {
                // Indirect addressing with optional displacement
                let mut cycles = EA_CYCLES[base_index as usize];

                // Displacement adds cycles (both disp8 and disp16 add same cost)
                if operand.disp != 0 {
                    cycles += DISP_CYCLES;
                }

                cycles
            } else {
                // Invalid base_index - shouldn't happen
                0
            }
        }

        // Direct addressing mode
        OperandType::Direct => DIRECT_ADDRESS_CYCLES,
    }
}

/// Calculate memory transfer penalty for 16-bit operands on 8088
///
/// The 8088 has an 8-bit data bus, so 16-bit memory transfers require
/// two bus cycles, adding 4 extra cycles.
#[inline(always)]
pub fn calculate_transfer_penalty(operand: &Operand) -> u8 {
    match operand.op_type {
        OperandType::Mem16 => WORD_TRANSFER_PENALTY,
        _ => 0,
    }
}

/// Calculate total EA cycles for an instruction with dst and src operands
///
/// Returns the sum of EA cycles for both operands.
#[inline(always)]
pub fn calculate_total_ea_cycles(dst: &Operand, src: &Operand) -> u8 {
    // Only one operand can be memory in 8088 instructions
    // (no mem-to-mem except string ops which handle their own timing)
    calculate_ea_cycles(dst) + calculate_ea_cycles(src)
}

/// Calculate total transfer penalty for an instruction
#[inline(always)]
pub fn calculate_total_transfer_penalty(dst: &Operand, src: &Operand) -> u8 {
    calculate_transfer_penalty(dst) + calculate_transfer_penalty(src)
}

/// Check if an operand is a memory operand
#[inline(always)]
pub fn is_memory_operand(operand: &Operand) -> bool {
    matches!(
        operand.op_type,
        OperandType::Mem8 | OperandType::Mem16 | OperandType::Direct
    )
}

/// Additional cycles for instructions with memory destination vs memory source
///
/// Many instructions have different timing when writing to memory vs reading from it.
/// For example:
/// - ADD r, r/m: 3 cycles (reg-reg) or 9+EA (reg-mem read)
/// - ADD r/m, r: 3 cycles (reg-reg) or 16+EA (mem-reg write)
///
/// This returns the additional cycles for memory destination operations.
pub const MEMORY_DEST_EXTRA_CYCLES: u8 = 7; // 16+EA vs 9+EA = 7 extra for mem dest

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ea_cycles_register() {
        let reg = Operand::reg16(0);
        assert_eq!(calculate_ea_cycles(&reg), 0);
    }

    #[test]
    fn test_ea_cycles_direct() {
        let direct = Operand::new(OperandType::Mem16, 0xFF).with_disp(0x1234);
        assert_eq!(calculate_ea_cycles(&direct), DIRECT_ADDRESS_CYCLES);
    }

    #[test]
    fn test_ea_cycles_bx_si() {
        // [BX+SI] = 7 cycles
        let mem = Operand::mem16(0);
        assert_eq!(calculate_ea_cycles(&mem), 7);
    }

    #[test]
    fn test_ea_cycles_with_disp() {
        // [BX+SI+disp] = 7 + 4 = 11 cycles
        let mem = Operand::mem16_disp(0, 0x10);
        assert_eq!(calculate_ea_cycles(&mem), 11);
    }

    #[test]
    fn test_transfer_penalty_word() {
        let mem16 = Operand::mem16(0);
        assert_eq!(calculate_transfer_penalty(&mem16), WORD_TRANSFER_PENALTY);
    }

    #[test]
    fn test_transfer_penalty_byte() {
        let mem8 = Operand::mem8(0);
        assert_eq!(calculate_transfer_penalty(&mem8), 0);
    }
}
