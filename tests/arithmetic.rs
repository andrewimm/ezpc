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

#[test]
fn test_sub_al_imm8() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x50; SUB AL, 0x20
    harness.load_program(&[0xB0, 0x50, 0x2C, 0x20], 0);

    harness.step(); // MOV AL, 0x50
    harness.step(); // SUB AL, 0x20

    assert_eq!(harness.cpu.read_reg8(0), 0x30); // AL = 0x50 - 0x20 = 0x30
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // No borrow
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // No overflow
}

#[test]
fn test_sub_ax_imm16() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1234; SUB AX, 0x0234
    harness.load_program(&[0xB8, 0x34, 0x12, 0x2D, 0x34, 0x02], 0);

    harness.step(); // MOV AX, 0x1234
    harness.step(); // SUB AX, 0x0234

    assert_eq!(harness.cpu.regs[0], 0x1000); // AX = 0x1234 - 0x0234 = 0x1000
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // No borrow
}

#[test]
fn test_sub_r8_r8() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x50; MOV BL, 0x20; SUB AL, BL
    harness.load_program(&[0xB0, 0x50, 0xB3, 0x20, 0x28, 0xD8], 0);

    harness.step(); // MOV AL, 0x50
    harness.step(); // MOV BL, 0x20
    harness.step(); // SUB AL, BL (0x28 0xD8: ModRM = 11 011 000 = reg BL, r/m AL)

    assert_eq!(harness.cpu.read_reg8(0), 0x30); // AL = 0x50 - 0x20 = 0x30
}

#[test]
fn test_sub_r16_r16() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x2000; MOV BX, 0x1000; SUB AX, BX
    harness.load_program(&[0xB8, 0x00, 0x20, 0xBB, 0x00, 0x10, 0x29, 0xD8], 0);

    harness.step(); // MOV AX, 0x2000
    harness.step(); // MOV BX, 0x1000
    harness.step(); // SUB AX, BX (0x29 0xD8: ModRM = 11 011 000)

    assert_eq!(harness.cpu.regs[0], 0x1000); // AX = 0x2000 - 0x1000 = 0x1000
}

#[test]
fn test_sub_r8_rm8() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x50; MOV BL, 0x20; SUB BL, AL
    harness.load_program(&[0xB0, 0x50, 0xB3, 0x20, 0x2A, 0xD8], 0);

    harness.step(); // MOV AL, 0x50
    harness.step(); // MOV BL, 0x20
    harness.step(); // SUB BL, AL (0x2A 0xD8: ModRM = 11 011 000 = reg BL, r/m AL)

    assert_eq!(harness.cpu.read_reg8(3), 0xD0); // BL = 0x20 - 0x50 = 0xD0 (underflow)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // Borrow occurred
}

#[test]
fn test_sub_r16_rm16() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x2000; MOV BX, 0x1000; SUB BX, AX
    harness.load_program(&[0xB8, 0x00, 0x20, 0xBB, 0x00, 0x10, 0x2B, 0xD8], 0);

    harness.step(); // MOV AX, 0x2000
    harness.step(); // MOV BX, 0x1000
    harness.step(); // SUB BX, AX (0x2B 0xD8)

    assert_eq!(harness.cpu.regs[3], 0xF000); // BX = 0x1000 - 0x2000 = 0xF000 (underflow)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // Borrow occurred
}

#[test]
fn test_sub_borrow_8bit() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x10; SUB AL, 0x20
    harness.load_program(&[0xB0, 0x10, 0x2C, 0x20], 0);

    harness.step(); // MOV AL, 0x10
    harness.step(); // SUB AL, 0x20

    assert_eq!(harness.cpu.read_reg8(0), 0xF0); // AL = 0x10 - 0x20 = 0xF0 (underflow)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be set (borrow)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set (negative)
}

#[test]
fn test_sub_zero_result() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x42; SUB AL, 0x42
    harness.load_program(&[0xB0, 0x42, 0x2C, 0x42], 0);

    harness.step(); // MOV AL, 0x42
    harness.step(); // SUB AL, 0x42

    assert_eq!(harness.cpu.read_reg8(0), 0x00); // AL = 0
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_sub_overflow_8bit() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x80 (-128); SUB AL, 1
    harness.load_program(&[0xB0, 0x80, 0x2C, 0x01], 0);

    harness.step(); // MOV AL, 0x80
    harness.step(); // SUB AL, 1

    assert_eq!(harness.cpu.read_reg8(0), 0x7F); // AL = 127
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be set (signed overflow)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear (no borrow)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear (positive result)
}

#[test]
fn test_sub_overflow_16bit() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x8000; SUB AX, 1
    harness.load_program(&[0xB8, 0x00, 0x80, 0x2D, 0x01, 0x00], 0);

    harness.step(); // MOV AX, 0x8000
    harness.step(); // SUB AX, 1

    assert_eq!(harness.cpu.regs[0], 0x7FFF); // AX = 32767
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be set
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
}

#[test]
fn test_sub_no_overflow_8bit() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x7F; SUB AL, 1
    harness.load_program(&[0xB0, 0x7F, 0x2C, 0x01], 0);

    harness.step(); // MOV AL, 0x7F
    harness.step(); // SUB AL, 1

    assert_eq!(harness.cpu.read_reg8(0), 0x7E); // AL = 126
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
}

#[test]
fn test_sub_rm8_imm8() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x50; SUB AL, 0x20 (using 0x80 /5)
    harness.load_program(&[0xB0, 0x50, 0x80, 0xE8, 0x20], 0);

    harness.step(); // MOV AL, 0x50
    harness.step(); // SUB AL, 0x20 (0x80 0xE8 0x20: ModRM = 11 101 000 = /5 AL)

    assert_eq!(harness.cpu.read_reg8(0), 0x30); // AL = 0x30
}

#[test]
fn test_sub_rm16_imm16() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1234; SUB AX, 0x0234 (using 0x81 /5)
    harness.load_program(&[0xB8, 0x34, 0x12, 0x81, 0xE8, 0x34, 0x02], 0);

    harness.step(); // MOV AX, 0x1234
    harness.step(); // SUB AX, 0x0234 (0x81 0xE8 0x34 0x02: ModRM = 11 101 000 = /5 AX)

    assert_eq!(harness.cpu.regs[0], 0x1000); // AX = 0x1000
}

#[test]
fn test_sub_rm16_imm8_sign_extended() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x0100; SUB AX, 0x10 (using 0x83 /5, sign-extended)
    harness.load_program(&[0xB8, 0x00, 0x01, 0x83, 0xE8, 0x10], 0);

    harness.step(); // MOV AX, 0x0100
    harness.step(); // SUB AX, 0x10 (0x83 0xE8 0x10: sign-extended to 0x0010)

    assert_eq!(harness.cpu.regs[0], 0x00F0); // AX = 0x0100 - 0x0010 = 0x00F0
}

#[test]
fn test_sub_rm16_imm8_negative_sign_extended() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x0100; SUB AX, -1 (0xFF sign-extended to 0xFFFF)
    harness.load_program(&[0xB8, 0x00, 0x01, 0x83, 0xE8, 0xFF], 0);

    harness.step(); // MOV AX, 0x0100
    harness.step(); // SUB AX, 0xFF (sign-extended to 0xFFFF)

    // 0x0100 - 0xFFFF = 0x0100 + 0x0001 = 0x0101 (with borrow)
    assert_eq!(harness.cpu.regs[0], 0x0101); // AX = 0x0101
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // Borrow occurred
}
