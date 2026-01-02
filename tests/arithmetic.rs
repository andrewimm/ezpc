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

// === ADC (Add with Carry) Tests ===

#[test]
fn test_adc_r8_r8_no_carry() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x05; MOV BL, 0x03; ADC AL, BL
    harness.load_program(&[0xB0, 0x05, 0xB3, 0x03, 0x12, 0xC3], 0);

    harness.step(); // MOV AL, 0x05
    harness.step(); // MOV BL, 0x03
    harness.step(); // ADC AL, BL (AL = 0x05 + 0x03 + 0 = 0x08)

    assert_eq!(harness.cpu.read_reg8(0), 0x08); // AL
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false); // No carry
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), false);
}

#[test]
fn test_adc_r8_r8_with_carry() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; ADD AL, 0x01; MOV BL, 0x05; ADC BL, AL
    harness.load_program(&[0xB0, 0xFF, 0x04, 0x01, 0xB3, 0x05, 0x10, 0xC3], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // ADD AL, 0x01 (AL = 0x00, CF = 1)

    // Verify carry is set
    assert_eq!(harness.cpu.read_reg8(0), 0x00); // AL
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);

    harness.step(); // MOV BL, 0x05
    harness.step(); // ADC BL, AL (BL = 0x05 + 0x00 + 1 = 0x06)

    assert_eq!(harness.cpu.read_reg8(3), 0x06); // BL
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_adc_r16_r16_no_carry() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1234; MOV BX, 0x5678; ADC AX, BX
    harness.load_program(&[0xB8, 0x34, 0x12, 0xBB, 0x78, 0x56, 0x13, 0xC3], 0);

    harness.step(); // MOV AX, 0x1234
    harness.step(); // MOV BX, 0x5678
    harness.step(); // ADC AX, BX (0x13 0xC3: ModR/M = 11 000 011 = reg AX, r/m BX)

    assert_eq!(harness.cpu.regs[0], 0x68AC); // AX = 0x1234 + 0x5678 + 0 = 0x68AC
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_adc_r16_r16_with_carry() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; ADD AX, 0x0001; MOV BX, 0x1234; ADC BX, AX
    harness.load_program(
        &[
            0xB8, 0xFF, 0xFF, 0x05, 0x01, 0x00, 0xBB, 0x34, 0x12, 0x11, 0xC3,
        ],
        0,
    );

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // ADD AX, 0x0001 (AX = 0x0000, CF = 1)

    assert_eq!(harness.cpu.regs[0], 0x0000); // AX
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);

    harness.step(); // MOV BX, 0x1234
    harness.step(); // ADC BX, AX (BX = 0x1234 + 0x0000 + 1 = 0x1235)

    assert_eq!(harness.cpu.regs[3], 0x1235); // BX
}

#[test]
fn test_adc_al_imm8_no_carry() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x10; ADC AL, 0x20
    harness.load_program(&[0xB0, 0x10, 0x14, 0x20], 0);

    harness.step(); // MOV AL, 0x10
    harness.step(); // ADC AL, 0x20 (AL = 0x10 + 0x20 + 0 = 0x30)

    assert_eq!(harness.cpu.read_reg8(0), 0x30); // AL
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_adc_al_imm8_with_carry() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; ADD AL, 0x01; ADC AL, 0x05
    harness.load_program(&[0xB0, 0xFF, 0x04, 0x01, 0x14, 0x05], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // ADD AL, 0x01 (AL = 0x00, CF = 1)
    harness.step(); // ADC AL, 0x05 (AL = 0x00 + 0x05 + 1 = 0x06)

    assert_eq!(harness.cpu.read_reg8(0), 0x06); // AL
}

#[test]
fn test_adc_ax_imm16_no_carry() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1000; ADC AX, 0x2000
    harness.load_program(&[0xB8, 0x00, 0x10, 0x15, 0x00, 0x20], 0);

    harness.step(); // MOV AX, 0x1000
    harness.step(); // ADC AX, 0x2000 (AX = 0x1000 + 0x2000 + 0 = 0x3000)

    assert_eq!(harness.cpu.regs[0], 0x3000); // AX
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_adc_ax_imm16_with_carry() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; ADD AX, 0x0001; ADC AX, 0x1234
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0x05, 0x01, 0x00, 0x15, 0x34, 0x12], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // ADD AX, 0x0001 (AX = 0x0000, CF = 1)
    harness.step(); // ADC AX, 0x1234 (AX = 0x0000 + 0x1234 + 1 = 0x1235)

    assert_eq!(harness.cpu.regs[0], 0x1235); // AX
}

#[test]
fn test_adc_produces_carry() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x80; ADC AL, 0x80 (should produce carry)
    harness.load_program(&[0xB0, 0x80, 0x14, 0x80], 0);

    harness.step(); // MOV AL, 0x80
    harness.step(); // ADC AL, 0x80 (AL = 0x80 + 0x80 + 0 = 0x00, CF = 1)

    assert_eq!(harness.cpu.read_reg8(0), 0x00); // AL
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), true);
}

#[test]
fn test_adc_chain_operation() {
    let mut harness = CpuHarness::new();
    // Simulate multi-precision addition: (0x0001:0xFFFF) + (0x0000:0x0002) = (0x0002:0x0001)
    // MOV AX, 0xFFFF; MOV BX, 0x0001; MOV CX, 0x0002; MOV DX, 0x0000
    // ADD AX, CX; ADC BX, DX
    harness.load_program(
        &[
            0xB8, 0xFF, 0xFF, // MOV AX, 0xFFFF
            0xBB, 0x01, 0x00, // MOV BX, 0x0001
            0xB9, 0x02, 0x00, // MOV CX, 0x0002
            0xBA, 0x00, 0x00, // MOV DX, 0x0000
            0x01, 0xC8, // ADD AX, CX
            0x11, 0xD3, // ADC BX, DX
        ],
        0,
    );

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // MOV BX, 0x0001
    harness.step(); // MOV CX, 0x0002
    harness.step(); // MOV DX, 0x0000
    harness.step(); // ADD AX, CX (AX = 0xFFFF + 0x0002 = 0x0001, CF = 1)

    assert_eq!(harness.cpu.regs[0], 0x0001); // AX
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);

    harness.step(); // ADC BX, DX (BX = 0x0001 + 0x0000 + 1 = 0x0002)

    assert_eq!(harness.cpu.regs[3], 0x0002); // BX (high word)
    assert_eq!(harness.cpu.regs[0], 0x0001); // AX (low word)
}

// SBB tests

#[test]
fn test_sbb_r8_r8_no_borrow() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x10; MOV BL, 0x05; SBB AL, BL
    harness.load_program(&[0xB0, 0x10, 0xB3, 0x05, 0x1A, 0xC3], 0);

    harness.step(); // MOV AL, 0x10
    harness.step(); // MOV BL, 0x05
    harness.step(); // SBB AL, BL (AL = 0x10 - 0x05 - 0 = 0x0B)

    assert_eq!(harness.cpu.read_reg8(0), 0x0B); // AL
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false); // No borrow
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), false);
}

#[test]
fn test_sbb_r8_r8_with_borrow() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x00; SUB AL, 0x01; MOV BL, 0x05; SBB BL, AL
    harness.load_program(&[0xB0, 0x00, 0x2C, 0x01, 0xB3, 0x05, 0x18, 0xC3], 0);

    harness.step(); // MOV AL, 0x00
    harness.step(); // SUB AL, 0x01 (AL = 0xFF, CF = 1)

    // Verify borrow is set
    assert_eq!(harness.cpu.read_reg8(0), 0xFF); // AL
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);

    harness.step(); // MOV BL, 0x05
    harness.step(); // SBB BL, AL (BL = 0x05 - 0xFF - 1 = 0x05)

    assert_eq!(harness.cpu.read_reg8(3), 0x05); // BL
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);
}

#[test]
fn test_sbb_r16_r16_no_borrow() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x5678; MOV BX, 0x1234; SBB AX, BX
    harness.load_program(&[0xB8, 0x78, 0x56, 0xBB, 0x34, 0x12, 0x1B, 0xC3], 0);

    harness.step(); // MOV AX, 0x5678
    harness.step(); // MOV BX, 0x1234
    harness.step(); // SBB AX, BX (0x1B 0xC3: ModR/M = 11 000 011 = reg AX, r/m BX)

    assert_eq!(harness.cpu.regs[0], 0x4444); // AX = 0x5678 - 0x1234 - 0 = 0x4444
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_sbb_r16_r16_with_borrow() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x0000; SUB AX, 0x0001; MOV BX, 0x1234; SBB BX, AX
    harness.load_program(
        &[
            0xB8, 0x00, 0x00, 0x2D, 0x01, 0x00, 0xBB, 0x34, 0x12, 0x19, 0xC3,
        ],
        0,
    );

    harness.step(); // MOV AX, 0x0000
    harness.step(); // SUB AX, 0x0001 (AX = 0xFFFF, CF = 1)

    assert_eq!(harness.cpu.regs[0], 0xFFFF); // AX
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);

    harness.step(); // MOV BX, 0x1234
    harness.step(); // SBB BX, AX (BX = 0x1234 - 0xFFFF - 1 = 0x1234)

    assert_eq!(harness.cpu.regs[3], 0x1234); // BX
}

#[test]
fn test_sbb_al_imm8_no_borrow() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x30; SBB AL, 0x10
    harness.load_program(&[0xB0, 0x30, 0x1C, 0x10], 0);

    harness.step(); // MOV AL, 0x30
    harness.step(); // SBB AL, 0x10 (AL = 0x30 - 0x10 - 0 = 0x20)

    assert_eq!(harness.cpu.read_reg8(0), 0x20); // AL
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_sbb_al_imm8_with_borrow() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x00; SUB AL, 0x01; SBB AL, 0x05
    harness.load_program(&[0xB0, 0x00, 0x2C, 0x01, 0x1C, 0x05], 0);

    harness.step(); // MOV AL, 0x00
    harness.step(); // SUB AL, 0x01 (AL = 0xFF, CF = 1)
    harness.step(); // SBB AL, 0x05 (AL = 0xFF - 0x05 - 1 = 0xF9)

    assert_eq!(harness.cpu.read_reg8(0), 0xF9); // AL
}

#[test]
fn test_sbb_ax_imm16_no_borrow() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x3000; SBB AX, 0x1000
    harness.load_program(&[0xB8, 0x00, 0x30, 0x1D, 0x00, 0x10], 0);

    harness.step(); // MOV AX, 0x3000
    harness.step(); // SBB AX, 0x1000 (AX = 0x3000 - 0x1000 - 0 = 0x2000)

    assert_eq!(harness.cpu.regs[0], 0x2000); // AX
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_sbb_ax_imm16_with_borrow() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x0000; SUB AX, 0x0001; SBB AX, 0x1234
    harness.load_program(&[0xB8, 0x00, 0x00, 0x2D, 0x01, 0x00, 0x1D, 0x34, 0x12], 0);

    harness.step(); // MOV AX, 0x0000
    harness.step(); // SUB AX, 0x0001 (AX = 0xFFFF, CF = 1)
    harness.step(); // SBB AX, 0x1234 (AX = 0xFFFF - 0x1234 - 1 = 0xEDCA)

    assert_eq!(harness.cpu.regs[0], 0xEDCA); // AX
}

#[test]
fn test_sbb_produces_borrow() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x05; SBB AL, 0x10 (should produce borrow)
    harness.load_program(&[0xB0, 0x05, 0x1C, 0x10], 0);

    harness.step(); // MOV AL, 0x05
    harness.step(); // SBB AL, 0x10 (AL = 0x05 - 0x10 - 0 = 0xF5, CF = 1)

    assert_eq!(harness.cpu.read_reg8(0), 0xF5); // AL
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), false);
}

#[test]
fn test_sbb_chain_operation() {
    let mut harness = CpuHarness::new();
    // Simulate multi-precision subtraction: (0x0002:0x0001) - (0x0000:0x0002) = (0x0001:0xFFFF)
    // MOV AX, 0x0001; MOV BX, 0x0002; MOV CX, 0x0002; MOV DX, 0x0000
    // SUB AX, CX; SBB BX, DX
    harness.load_program(
        &[
            0xB8, 0x01, 0x00, // MOV AX, 0x0001
            0xBB, 0x02, 0x00, // MOV BX, 0x0002
            0xB9, 0x02, 0x00, // MOV CX, 0x0002
            0xBA, 0x00, 0x00, // MOV DX, 0x0000
            0x29, 0xC8, // SUB AX, CX
            0x19, 0xD3, // SBB BX, DX
        ],
        0,
    );

    harness.step(); // MOV AX, 0x0001
    harness.step(); // MOV BX, 0x0002
    harness.step(); // MOV CX, 0x0002
    harness.step(); // MOV DX, 0x0000

    harness.step(); // SUB AX, CX (AX = 0x0001 - 0x0002 = 0xFFFF, CF = 1)

    assert_eq!(harness.cpu.regs[0], 0xFFFF); // AX (low word)
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);

    harness.step(); // SBB BX, DX (BX = 0x0002 - 0x0000 - 1 = 0x0001)

    assert_eq!(harness.cpu.regs[3], 0x0001); // BX (high word)
    assert_eq!(harness.cpu.regs[0], 0xFFFF); // AX (low word)
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_sbb_zero_result() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x10; SBB AL, 0x10 (should produce zero with ZF set)
    harness.load_program(&[0xB0, 0x10, 0x1C, 0x10], 0);

    harness.step(); // MOV AL, 0x10
    harness.step(); // SBB AL, 0x10 (AL = 0x10 - 0x10 - 0 = 0x00, ZF = 1)

    assert_eq!(harness.cpu.read_reg8(0), 0x00); // AL
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), true);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}
