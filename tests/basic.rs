//! Basic instruction execution tests
//!
//! Tests basic CPU instructions using the test harness

use ezpc::cpu::CpuHarness;

#[test]
fn test_nop() {
    let mut harness = CpuHarness::new();
    harness.load_program(&[0x90], 0); // NOP

    // Execute NOP
    harness.step();

    // IP should have advanced by 1
    assert_eq!(harness.cpu.ip, 1);
}

#[test]
fn test_mov_r16_imm() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1234; MOV CX, 0x5678
    harness.load_program(&[0xB8, 0x34, 0x12, 0xB9, 0x78, 0x56], 0);

    // Execute MOV AX, 0x1234
    harness.step();
    assert_eq!(harness.cpu.regs[0], 0x1234); // AX
    assert_eq!(harness.cpu.ip, 3);

    // Execute MOV CX, 0x5678
    harness.step();
    assert_eq!(harness.cpu.regs[1], 0x5678); // CX
    assert_eq!(harness.cpu.ip, 6);
}

#[test]
fn test_mov_r8_imm() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x12; MOV AH, 0x34
    harness.load_program(&[0xB0, 0x12, 0xB4, 0x34], 0);

    // Execute MOV AL, 0x12
    harness.step();
    assert_eq!(harness.cpu.read_reg8(0), 0x12); // AL
    assert_eq!(harness.cpu.ip, 2);

    // Execute MOV AH, 0x34
    harness.step();
    assert_eq!(harness.cpu.read_reg8(4), 0x34); // AH
    assert_eq!(harness.cpu.regs[0], 0x3412); // AX should be 0x3412
    assert_eq!(harness.cpu.ip, 4);
}

#[test]
fn test_mov_r16_r16() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1234; MOV BX, AX
    harness.load_program(&[0xB8, 0x34, 0x12, 0x8B, 0xD8], 0);

    // Execute MOV AX, 0x1234
    harness.step();
    assert_eq!(harness.cpu.regs[0], 0x1234); // AX

    // Execute MOV BX, AX (8B D8: MOV r16, r/m16 with ModR/M=D8)
    harness.step();
    assert_eq!(harness.cpu.regs[3], 0x1234); // BX
}

#[test]
fn test_xchg_ax_r16() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1111; MOV CX, 0x2222; XCHG AX, CX
    harness.load_program(&[0xB8, 0x11, 0x11, 0xB9, 0x22, 0x22, 0x91], 0);

    harness.step(); // MOV AX, 0x1111
    harness.step(); // MOV CX, 0x2222
    harness.step(); // XCHG AX, CX

    assert_eq!(harness.cpu.regs[0], 0x2222); // AX
    assert_eq!(harness.cpu.regs[1], 0x1111); // CX
}

#[test]
fn test_push_pop() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x1000; MOV AX, 0x1234; PUSH AX; MOV AX, 0; POP AX
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
            0x50, // PUSH AX
            0xB8, 0x00, 0x00, // MOV AX, 0
            0x58, // POP AX
        ],
        0,
    );

    harness.step(); // MOV SP, 0x1000
    assert_eq!(harness.cpu.regs[4], 0x1000); // SP

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234); // AX

    harness.step(); // PUSH AX
    assert_eq!(harness.cpu.regs[4], 0x0FFE); // SP should decrement by 2

    harness.step(); // MOV AX, 0
    assert_eq!(harness.cpu.regs[0], 0x0000); // AX cleared

    harness.step(); // POP AX
    assert_eq!(harness.cpu.regs[0], 0x1234); // AX restored
    assert_eq!(harness.cpu.regs[4], 0x1000); // SP back to original
}

#[test]
fn test_inc_r16() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1234; INC AX
    harness.load_program(&[0xB8, 0x34, 0x12, 0x40], 0);

    harness.step(); // MOV AX, 0x1234
    harness.step(); // INC AX

    assert_eq!(harness.cpu.regs[0], 0x1235); // AX
}

#[test]
fn test_dec_r16() {
    let mut harness = CpuHarness::new();
    // MOV CX, 0x0005; DEC CX
    harness.load_program(&[0xB9, 0x05, 0x00, 0x49], 0);

    harness.step(); // MOV CX, 0x0005
    harness.step(); // DEC CX

    assert_eq!(harness.cpu.regs[1], 0x0004); // CX
}

#[test]
fn test_add_r16_imm() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x0010; ADD AX, 0x0020
    harness.load_program(&[0xB8, 0x10, 0x00, 0x05, 0x20, 0x00], 0);

    harness.step(); // MOV AX, 0x0010
    harness.step(); // ADD AX, 0x0020

    assert_eq!(harness.cpu.regs[0], 0x0030); // AX
}

#[test]
fn test_add_r8_imm() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x10; ADD AL, 0x20
    harness.load_program(&[0xB0, 0x10, 0x04, 0x20], 0);

    harness.step(); // MOV AL, 0x10
    harness.step(); // ADD AL, 0x20

    assert_eq!(harness.cpu.read_reg8(0), 0x30); // AL
}

#[test]
fn test_jmp_short() {
    let mut harness = CpuHarness::new();
    // JMP +5 (skip 5 bytes forward)
    // At offset 0: EB 05 (JMP short +5)
    // At offset 2-6: 5 bytes to skip
    // At offset 7: B8 34 12 (MOV AX, 0x1234)
    harness.load_program(
        &[
            0xEB, 0x05, // JMP +5
            0x90, 0x90, 0x90, 0x90, 0x90, // 5 NOPs to skip
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // JMP +5
    // After JMP, IP = 2 (after reading JMP instruction) + 5 = 7
    assert_eq!(harness.cpu.ip, 7);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234); // AX
}

#[test]
fn test_jz_taken() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0; ADD AL, 0 (sets ZF); JZ +2
    harness.load_program(
        &[
            0xB8, 0x00, 0x00, // MOV AX, 0
            0x04, 0x00, // ADD AL, 0 (sets ZF)
            0x74, 0x02, // JZ +2
            0x90, 0x90, // 2 NOPs to skip
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // MOV AX, 0
    harness.step(); // ADD AL, 0 (should set ZF)

    // Check that ZF is set
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF));

    harness.step(); // JZ +2 (should be taken)
    // IP should be 7 + 2 = 9
    assert_eq!(harness.cpu.ip, 9);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jnz_not_taken() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0; ADD AL, 0 (sets ZF); JNZ +2 (should NOT be taken)
    harness.load_program(
        &[
            0xB8, 0x00, 0x00, // MOV AX, 0
            0x04, 0x00, // ADD AL, 0 (sets ZF)
            0x75, 0x02, // JNZ +2
            0xB8, 0x34, 0x12, // MOV AX, 0x1234 (should execute)
        ],
        0,
    );

    harness.step(); // MOV AX, 0
    harness.step(); // ADD AL, 0
    harness.step(); // JNZ +2 (not taken)

    // IP should be 7 (not jumped)
    assert_eq!(harness.cpu.ip, 7);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jnz_taken() {
    let mut harness = CpuHarness::new();
    // MOV AX, 1; ADD AL, 0 (clears ZF); JNZ +2 (should be taken)
    harness.load_program(
        &[
            0xB8, 0x01, 0x00, // MOV AX, 1
            0x04, 0x00, // ADD AL, 0 (ZF should be clear)
            0x75, 0x02, // JNZ +2
            0x90, 0x90, // 2 NOPs to skip
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // MOV AX, 1
    harness.step(); // ADD AL, 0
    harness.step(); // JNZ +2 (should be taken)

    // IP should be 7 + 2 = 9
    assert_eq!(harness.cpu.ip, 9);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}
