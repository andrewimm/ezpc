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

#[test]
fn test_add_overflow_8bit() {
    let mut harness = CpuHarness::new();
    // MOV AL, 127; ADD AL, 1
    harness.load_program(&[0xB0, 0x7F, 0x04, 0x01], 0);

    harness.step(); // MOV AL, 127
    harness.step(); // ADD AL, 1

    assert_eq!(harness.cpu.read_reg8(0), 0x80); // AL = -128 in two's complement
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be set
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set (negative)
}

#[test]
fn test_add_no_overflow_8bit() {
    let mut harness = CpuHarness::new();
    // MOV AL, 1; ADD AL, 1
    harness.load_program(&[0xB0, 0x01, 0x04, 0x01], 0);

    harness.step(); // MOV AL, 1
    harness.step(); // ADD AL, 1

    assert_eq!(harness.cpu.read_reg8(0), 0x02); // AL = 2
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear (positive)
}

#[test]
fn test_add_overflow_negative_8bit() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x80 (-128); ADD AL, 0xFF (-1)
    harness.load_program(&[0xB0, 0x80, 0x04, 0xFF], 0);

    harness.step(); // MOV AL, 0x80
    harness.step(); // ADD AL, 0xFF

    assert_eq!(harness.cpu.read_reg8(0), 0x7F); // AL = 127
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be set
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be set (carry)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear (positive)
}

#[test]
fn test_add_overflow_16bit() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x7FFF; ADD AX, 1
    harness.load_program(&[0xB8, 0xFF, 0x7F, 0x05, 0x01, 0x00], 0);

    harness.step(); // MOV AX, 0x7FFF
    harness.step(); // ADD AX, 1

    assert_eq!(harness.cpu.regs[0], 0x8000); // AX = -32768 in two's complement
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be set
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set (negative)
}

#[test]
fn test_add_carry_no_overflow_8bit() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; ADD AL, 1
    harness.load_program(&[0xB0, 0xFF, 0x04, 0x01], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // ADD AL, 1

    assert_eq!(harness.cpu.read_reg8(0), 0x00); // AL = 0
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear (unsigned overflow only)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be set
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set
}

#[test]
fn test_inc_overflow_16bit() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x7FFF; INC AX
    harness.load_program(&[0xB8, 0xFF, 0x7F, 0x40], 0);

    harness.step(); // MOV AX, 0x7FFF
    harness.step(); // INC AX

    assert_eq!(harness.cpu.regs[0], 0x8000); // AX should be 0x8000
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be set
}

#[test]
fn test_dec_overflow_16bit() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x8000; DEC AX
    harness.load_program(&[0xB8, 0x00, 0x80, 0x48], 0);

    harness.step(); // MOV AX, 0x8000
    harness.step(); // DEC AX

    assert_eq!(harness.cpu.regs[0], 0x7FFF); // AX = 0x7FFF
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be set
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear (positive)
}
