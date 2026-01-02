//! Arithmetic instruction tests (ADD, INC, DEC, etc.)

use ezpc::cpu::CpuHarness;

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
