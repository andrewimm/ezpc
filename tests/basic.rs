//! Basic data transfer instruction tests (MOV, XCHG, NOP)

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
