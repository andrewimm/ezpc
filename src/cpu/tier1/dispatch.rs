//! Dispatch table for tier 1 execution
//!
//! Maps all 256 opcodes to their handler functions

use crate::cpu::decode::instruction::InstructionHandler;
use crate::cpu::execute::*;

/// Dispatch table with 256 entries, one for each possible opcode
///
/// Each entry is a function pointer that handles the opcode.
/// Unimplemented opcodes point to the invalid_opcode handler.
pub static DISPATCH_TABLE: [InstructionHandler; 256] = [
    // 0x00-0x0F: ADD, OR, ADC, SBB, AND, SUB, XOR, CMP, and segment prefixes
    arithmetic::add_rm_r,    // 0x00: ADD r/m8, r8
    arithmetic::add_rm_r,    // 0x01: ADD r/m16, r16
    arithmetic::add_r_rm,    // 0x02: ADD r8, r/m8
    arithmetic::add_r_rm,    // 0x03: ADD r16, r/m16
    arithmetic::add_acc_imm, // 0x04: ADD AL, imm8
    arithmetic::add_acc_imm, // 0x05: ADD AX, imm16
    stack::push_seg,         // 0x06: PUSH ES
    stack::pop_seg,          // 0x07: POP ES
    logic::or_rm_r,          // 0x08: OR r/m8, r8
    logic::or_rm_r,          // 0x09: OR r/m16, r16
    logic::or_r_rm,          // 0x0A: OR r8, r/m8
    logic::or_r_rm,          // 0x0B: OR r16, r/m16
    logic::or_acc_imm,       // 0x0C: OR AL, imm8
    logic::or_acc_imm,       // 0x0D: OR AX, imm16
    stack::push_seg,         // 0x0E: PUSH CS
    invalid_opcode,          // 0x0F: (two-byte escape on 80186+, invalid on 8088)
    // 0x10-0x1F: ADC, SBB, and segment prefixes
    arithmetic::adc_rm_r,    // 0x10: ADC r/m8, r8
    arithmetic::adc_rm_r,    // 0x11: ADC r/m16, r16
    arithmetic::adc_r_rm,    // 0x12: ADC r8, r/m8
    arithmetic::adc_r_rm,    // 0x13: ADC r16, r/m16
    arithmetic::adc_acc_imm, // 0x14: ADC AL, imm8
    arithmetic::adc_acc_imm, // 0x15: ADC AX, imm16
    stack::push_seg,         // 0x16: PUSH SS
    stack::pop_seg,          // 0x17: POP SS
    arithmetic::sbb_rm_r,    // 0x18: SBB r/m8, r8
    arithmetic::sbb_rm_r,    // 0x19: SBB r/m16, r16
    arithmetic::sbb_r_rm,    // 0x1A: SBB r8, r/m8
    arithmetic::sbb_r_rm,    // 0x1B: SBB r16, r/m16
    arithmetic::sbb_acc_imm, // 0x1C: SBB AL, imm8
    arithmetic::sbb_acc_imm, // 0x1D: SBB AX, imm16
    stack::push_seg,         // 0x1E: PUSH DS
    stack::pop_seg,          // 0x1F: POP DS
    // 0x20-0x2F: AND, SUB
    logic::and_rm_r,         // 0x20: AND r/m8, r8
    logic::and_rm_r,         // 0x21: AND r/m16, r16
    logic::and_r_rm,         // 0x22: AND r8, r/m8
    logic::and_r_rm,         // 0x23: AND r16, r/m16
    logic::and_acc_imm,      // 0x24: AND AL, imm8
    logic::and_acc_imm,      // 0x25: AND AX, imm16
    prefix::seg_es,          // 0x26: ES segment override prefix
    arithmetic::daa,         // 0x27: DAA
    arithmetic::sub_rm_r,    // 0x28: SUB r/m8, r8
    arithmetic::sub_rm_r,    // 0x29: SUB r/m16, r16
    arithmetic::sub_r_rm,    // 0x2A: SUB r8, r/m8
    arithmetic::sub_r_rm,    // 0x2B: SUB r16, r/m16
    arithmetic::sub_acc_imm, // 0x2C: SUB AL, imm8
    arithmetic::sub_acc_imm, // 0x2D: SUB AX, imm16
    prefix::seg_cs,          // 0x2E: CS segment override prefix
    arithmetic::das,         // 0x2F: DAS
    // 0x30-0x3F: XOR, CMP
    logic::xor_rm_r,         // 0x30: XOR r/m8, r8
    logic::xor_rm_r,         // 0x31: XOR r/m16, r16
    logic::xor_r_rm,         // 0x32: XOR r8, r/m8
    logic::xor_r_rm,         // 0x33: XOR r16, r/m16
    logic::xor_acc_imm,      // 0x34: XOR AL, imm8
    logic::xor_acc_imm,      // 0x35: XOR AX, imm16
    prefix::seg_ss,          // 0x36: SS segment override prefix
    invalid_opcode,          // 0x37: AAA (not implemented yet)
    arithmetic::cmp_rm_r,    // 0x38: CMP r/m8, r8
    arithmetic::cmp_rm_r,    // 0x39: CMP r/m16, r16
    arithmetic::cmp_r_rm,    // 0x3A: CMP r8, r/m8
    arithmetic::cmp_r_rm,    // 0x3B: CMP r16, r/m16
    arithmetic::cmp_acc_imm, // 0x3C: CMP AL, imm8
    arithmetic::cmp_acc_imm, // 0x3D: CMP AX, imm16
    prefix::seg_ds,          // 0x3E: DS segment override prefix
    invalid_opcode,          // 0x3F: AAS (not implemented yet)
    // 0x40-0x4F: INC and DEC 16-bit registers
    arithmetic::inc_r16, // 0x40: INC AX
    arithmetic::inc_r16, // 0x41: INC CX
    arithmetic::inc_r16, // 0x42: INC DX
    arithmetic::inc_r16, // 0x43: INC BX
    arithmetic::inc_r16, // 0x44: INC SP
    arithmetic::inc_r16, // 0x45: INC BP
    arithmetic::inc_r16, // 0x46: INC SI
    arithmetic::inc_r16, // 0x47: INC DI
    arithmetic::dec_r16, // 0x48: DEC AX
    arithmetic::dec_r16, // 0x49: DEC CX
    arithmetic::dec_r16, // 0x4A: DEC DX
    arithmetic::dec_r16, // 0x4B: DEC BX
    arithmetic::dec_r16, // 0x4C: DEC SP
    arithmetic::dec_r16, // 0x4D: DEC BP
    arithmetic::dec_r16, // 0x4E: DEC SI
    arithmetic::dec_r16, // 0x4F: DEC DI
    // 0x50-0x5F: PUSH and POP 16-bit registers
    stack::push_r16, // 0x50: PUSH AX
    stack::push_r16, // 0x51: PUSH CX
    stack::push_r16, // 0x52: PUSH DX
    stack::push_r16, // 0x53: PUSH BX
    stack::push_r16, // 0x54: PUSH SP
    stack::push_r16, // 0x55: PUSH BP
    stack::push_r16, // 0x56: PUSH SI
    stack::push_r16, // 0x57: PUSH DI
    stack::pop_r16,  // 0x58: POP AX
    stack::pop_r16,  // 0x59: POP CX
    stack::pop_r16,  // 0x5A: POP DX
    stack::pop_r16,  // 0x5B: POP BX
    stack::pop_r16,  // 0x5C: POP SP
    stack::pop_r16,  // 0x5D: POP BP
    stack::pop_r16,  // 0x5E: POP SI
    stack::pop_r16,  // 0x5F: POP DI
    // 0x60-0x6F: PUSHA, POPA, BOUND, ARPL (80186+), and segment prefixes
    invalid_opcode, // 0x60: PUSHA (80186+, not on 8088)
    invalid_opcode, // 0x61: POPA (80186+, not on 8088)
    invalid_opcode, // 0x62: BOUND (80186+, not on 8088)
    invalid_opcode, // 0x63: ARPL (80286+, not on 8088)
    invalid_opcode, // 0x64: FS segment prefix (80386+, not on 8088)
    invalid_opcode, // 0x65: GS segment prefix (80386+, not on 8088)
    invalid_opcode, // 0x66: Operand size prefix (80386+, not on 8088)
    invalid_opcode, // 0x67: Address size prefix (80386+, not on 8088)
    invalid_opcode, // 0x68: PUSH imm16 (80186+, not on 8088)
    invalid_opcode, // 0x69: IMUL r16, r/m16, imm16 (80186+, not on 8088)
    invalid_opcode, // 0x6A: PUSH imm8 (80186+, not on 8088)
    invalid_opcode, // 0x6B: IMUL r16, r/m16, imm8 (80186+, not on 8088)
    invalid_opcode, // 0x6C: INSB (80186+, not on 8088)
    invalid_opcode, // 0x6D: INSW (80186+, not on 8088)
    invalid_opcode, // 0x6E: OUTSB (80186+, not on 8088)
    invalid_opcode, // 0x6F: OUTSW (80186+, not on 8088)
    // 0x70-0x7F: Conditional jumps (short)
    control_flow::jo,  // 0x70: JO
    control_flow::jno, // 0x71: JNO
    control_flow::jc,  // 0x72: JC/JB/JNAE
    control_flow::jnc, // 0x73: JNC/JAE/JNB
    control_flow::jz,  // 0x74: JZ/JE
    control_flow::jnz, // 0x75: JNZ/JNE
    control_flow::jbe, // 0x76: JBE/JNA
    control_flow::ja,  // 0x77: JA/JNBE
    control_flow::js,  // 0x78: JS
    control_flow::jns, // 0x79: JNS
    control_flow::jp,  // 0x7A: JP/JPE
    control_flow::jnp, // 0x7B: JNP/JPO
    control_flow::jl,  // 0x7C: JL/JNGE
    control_flow::jge, // 0x7D: JGE/JNL
    control_flow::jle, // 0x7E: JLE/JNG
    control_flow::jg,  // 0x7F: JG/JNLE
    // 0x80-0x8F: Arithmetic and MOV instructions with immediate/ModR/M
    arithmetic::group_80,       // 0x80: Arithmetic r/m8, imm8 (group)
    arithmetic::group_81,       // 0x81: Arithmetic r/m16, imm16 (group)
    arithmetic::group_80,       // 0x82: Arithmetic r/m8, imm8 (alias of 0x80)
    arithmetic::group_83,       // 0x83: Arithmetic r/m16, imm8 (sign-extended, group)
    logic::test_rm_r,           // 0x84: TEST r/m8, r8
    logic::test_rm_r,           // 0x85: TEST r/m16, r16
    data_transfer::xchg_rm_r,   // 0x86: XCHG r/m8, r8
    data_transfer::xchg_rm_r,   // 0x87: XCHG r/m16, r16
    data_transfer::mov_rm_r,    // 0x88: MOV r/m8, r8
    data_transfer::mov_rm_r,    // 0x89: MOV r/m16, r16
    data_transfer::mov_r_rm,    // 0x8A: MOV r8, r/m8
    data_transfer::mov_r_rm,    // 0x8B: MOV r16, r/m16
    data_transfer::mov_rm_sreg, // 0x8C: MOV r/m16, Sreg
    data_transfer::lea,         // 0x8D: LEA r16, m - Load Effective Address
    data_transfer::mov_sreg_rm, // 0x8E: MOV Sreg, r/m16
    invalid_opcode,             // 0x8F: POP r/m16 (group, not implemented yet)
    // 0x90-0x9F: XCHG, CBW, CWD, CALL, WAIT, PUSHF, POPF, SAHF, LAHF
    nop,                        // 0x90: NOP (XCHG AX, AX)
    data_transfer::xchg_ax_r16, // 0x91: XCHG AX, CX
    data_transfer::xchg_ax_r16, // 0x92: XCHG AX, DX
    data_transfer::xchg_ax_r16, // 0x93: XCHG AX, BX
    data_transfer::xchg_ax_r16, // 0x94: XCHG AX, SP
    data_transfer::xchg_ax_r16, // 0x95: XCHG AX, BP
    data_transfer::xchg_ax_r16, // 0x96: XCHG AX, SI
    data_transfer::xchg_ax_r16, // 0x97: XCHG AX, DI
    invalid_opcode,             // 0x98: CBW (not implemented yet)
    invalid_opcode,             // 0x99: CWD (not implemented yet)
    control_flow::call_far,     // 0x9A: CALL far
    invalid_opcode,             // 0x9B: WAIT (not implemented yet)
    flags::pushf,               // 0x9C: PUSHF - Push FLAGS register
    flags::popf,                // 0x9D: POPF - Pop FLAGS register
    flags::sahf,                // 0x9E: SAHF - Store AH into Flags
    flags::lahf,                // 0x9F: LAHF - Load AH from Flags
    // 0xA0-0xAF: MOV, string operations
    invalid_opcode,      // 0xA0: MOV AL, moffs8 (not implemented yet)
    invalid_opcode,      // 0xA1: MOV AX, moffs16 (not implemented yet)
    invalid_opcode,      // 0xA2: MOV moffs8, AL (not implemented yet)
    invalid_opcode,      // 0xA3: MOV moffs16, AX (not implemented yet)
    string::movsb,       // 0xA4: MOVSB - Move byte from DS:SI to ES:DI
    string::movsw,       // 0xA5: MOVSW - Move word from DS:SI to ES:DI
    string::cmpsb,       // 0xA6: CMPSB - Compare bytes at DS:SI and ES:DI
    string::cmpsw,       // 0xA7: CMPSW - Compare words at DS:SI and ES:DI
    logic::test_acc_imm, // 0xA8: TEST AL, imm8
    logic::test_acc_imm, // 0xA9: TEST AX, imm16
    string::stosb,       // 0xAA: STOSB - Store AL to ES:DI
    string::stosw,       // 0xAB: STOSW - Store AX to ES:DI
    string::lodsb,       // 0xAC: LODSB - Load DS:SI into AL
    string::lodsw,       // 0xAD: LODSW - Load DS:SI into AX
    string::scasb,       // 0xAE: SCASB - Scan byte: compare AL with ES:DI
    string::scasw,       // 0xAF: SCASW - Scan word: compare AX with ES:DI
    // 0xB0-0xBF: MOV immediate to register
    data_transfer::mov_r_imm, // 0xB0: MOV AL, imm8
    data_transfer::mov_r_imm, // 0xB1: MOV CL, imm8
    data_transfer::mov_r_imm, // 0xB2: MOV DL, imm8
    data_transfer::mov_r_imm, // 0xB3: MOV BL, imm8
    data_transfer::mov_r_imm, // 0xB4: MOV AH, imm8
    data_transfer::mov_r_imm, // 0xB5: MOV CH, imm8
    data_transfer::mov_r_imm, // 0xB6: MOV DH, imm8
    data_transfer::mov_r_imm, // 0xB7: MOV BH, imm8
    data_transfer::mov_r_imm, // 0xB8: MOV AX, imm16
    data_transfer::mov_r_imm, // 0xB9: MOV CX, imm16
    data_transfer::mov_r_imm, // 0xBA: MOV DX, imm16
    data_transfer::mov_r_imm, // 0xBB: MOV BX, imm16
    data_transfer::mov_r_imm, // 0xBC: MOV SP, imm16
    data_transfer::mov_r_imm, // 0xBD: MOV BP, imm16
    data_transfer::mov_r_imm, // 0xBE: MOV SI, imm16
    data_transfer::mov_r_imm, // 0xBF: MOV DI, imm16
    // 0xC0-0xCF: Shifts, RET, LES, LDS, MOV, ENTER, LEAVE, RETF, INT
    invalid_opcode,             // 0xC0: Shift r/m8, imm8 (80186+, not on 8088)
    invalid_opcode,             // 0xC1: Shift r/m16, imm8 (80186+, not on 8088)
    control_flow::ret_near_imm, // 0xC2: RET imm16
    control_flow::ret_near,     // 0xC3: RET
    invalid_opcode,             // 0xC4: LES r16, m16:16 (not implemented yet)
    invalid_opcode,             // 0xC5: LDS r16, m16:16 (not implemented yet)
    data_transfer::mov_rm_imm,  // 0xC6: MOV r/m8, imm8
    data_transfer::mov_rm_imm,  // 0xC7: MOV r/m16, imm16
    invalid_opcode,             // 0xC8: ENTER (80186+, not on 8088)
    invalid_opcode,             // 0xC9: LEAVE (80186+, not on 8088)
    control_flow::ret_far_imm,  // 0xCA: RETF imm16
    control_flow::ret_far,      // 0xCB: RETF
    control_flow::int3,         // 0xCC: INT 3
    control_flow::int_n,        // 0xCD: INT imm8
    invalid_opcode,             // 0xCE: INTO (not implemented yet)
    control_flow::iret,         // 0xCF: IRET
    // 0xD0-0xDF: Shifts and rotates
    invalid_opcode, // 0xD0: Shift r/m8, 1 (not implemented yet)
    invalid_opcode, // 0xD1: Shift r/m16, 1 (not implemented yet)
    invalid_opcode, // 0xD2: Shift r/m8, CL (not implemented yet)
    invalid_opcode, // 0xD3: Shift r/m16, CL (not implemented yet)
    invalid_opcode, // 0xD4: AAM (not implemented yet)
    invalid_opcode, // 0xD5: AAD (not implemented yet)
    invalid_opcode, // 0xD6: SALC (undocumented, not implemented)
    invalid_opcode, // 0xD7: XLAT (not implemented yet)
    invalid_opcode, // 0xD8: ESC (FPU, not implemented)
    invalid_opcode, // 0xD9: ESC (FPU, not implemented)
    invalid_opcode, // 0xDA: ESC (FPU, not implemented)
    invalid_opcode, // 0xDB: ESC (FPU, not implemented)
    invalid_opcode, // 0xDC: ESC (FPU, not implemented)
    invalid_opcode, // 0xDD: ESC (FPU, not implemented)
    invalid_opcode, // 0xDE: ESC (FPU, not implemented)
    invalid_opcode, // 0xDF: ESC (FPU, not implemented)
    // 0xE0-0xEF: LOOP, IN, OUT, CALL, JMP
    control_flow::loopne,    // 0xE0: LOOPNE/LOOPNZ
    control_flow::loope,     // 0xE1: LOOPE/LOOPZ
    control_flow::loop_rel8, // 0xE2: LOOP
    control_flow::jcxz,      // 0xE3: JCXZ
    io::in_al_imm8,          // 0xE4: IN AL, imm8
    io::in_ax_imm8,          // 0xE5: IN AX, imm8
    io::out_imm8_al,         // 0xE6: OUT imm8, AL
    io::out_imm8_ax,         // 0xE7: OUT imm8, AX
    control_flow::call_near, // 0xE8: CALL near
    control_flow::jmp_near,  // 0xE9: JMP near
    control_flow::jmp_far,   // 0xEA: JMP far
    control_flow::jmp_short, // 0xEB: JMP short
    io::in_al_dx,            // 0xEC: IN AL, DX
    io::in_ax_dx,            // 0xED: IN AX, DX
    io::out_dx_al,           // 0xEE: OUT DX, AL
    io::out_dx_ax,           // 0xEF: OUT DX, AX
    // 0xF0-0xFF: LOCK, INT1, REP, HLT, CMC, and groups
    invalid_opcode,         // 0xF0: LOCK prefix (not implemented yet)
    invalid_opcode,         // 0xF1: INT1 (undocumented, not implemented)
    prefix::repne,          // 0xF2: REPNE/REPNZ prefix
    prefix::rep,            // 0xF3: REP/REPE/REPZ prefix
    hlt,                    // 0xF4: HLT
    invalid_opcode,         // 0xF5: CMC (not implemented yet)
    invalid_opcode, // 0xF6: TEST/NOT/NEG/MUL/IMUL/DIV/IDIV r/m8 (group, not implemented yet)
    invalid_opcode, // 0xF7: TEST/NOT/NEG/MUL/IMUL/DIV/IDIV r/m16 (group, not implemented yet)
    flags::clc,     // 0xF8: CLC - Clear Carry Flag
    flags::stc,     // 0xF9: STC - Set Carry Flag
    flags::cli,     // 0xFA: CLI - Clear Interrupt Flag
    flags::sti,     // 0xFB: STI - Set Interrupt Flag
    flags::cld,     // 0xFC: CLD - Clear Direction Flag
    flags::std,     // 0xFD: STD - Set Direction Flag
    invalid_opcode, // 0xFE: INC/DEC r/m8 (group, not implemented yet)
    control_flow::group_ff, // 0xFF: INC/DEC/CALL/JMP/PUSH r/m16 (group)
];
