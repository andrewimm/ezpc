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

// CMP tests

#[test]
fn test_cmp_r8_r8_equal() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x42; MOV BL, 0x42; CMP AL, BL
    harness.load_program(&[0xB0, 0x42, 0xB3, 0x42, 0x3A, 0xC3], 0);

    harness.step(); // MOV AL, 0x42
    harness.step(); // MOV BL, 0x42
    harness.step(); // CMP AL, BL (AL - BL = 0)

    // Result should be zero (equal)
    assert_eq!(harness.cpu.read_reg8(0), 0x42); // AL unchanged
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), true); // Zero flag set
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false); // No borrow
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF), false); // Not negative
}

#[test]
fn test_cmp_r8_r8_greater() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x50; MOV BL, 0x30; CMP AL, BL
    harness.load_program(&[0xB0, 0x50, 0xB3, 0x30, 0x3A, 0xC3], 0);

    harness.step(); // MOV AL, 0x50
    harness.step(); // MOV BL, 0x30
    harness.step(); // CMP AL, BL (AL - BL = 0x20, positive)

    assert_eq!(harness.cpu.read_reg8(0), 0x50); // AL unchanged
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), false); // Not zero
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false); // No borrow
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF), false); // Positive
}

#[test]
fn test_cmp_r8_r8_less() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x10; MOV BL, 0x20; CMP AL, BL
    harness.load_program(&[0xB0, 0x10, 0xB3, 0x20, 0x3A, 0xC3], 0);

    harness.step(); // MOV AL, 0x10
    harness.step(); // MOV BL, 0x20
    harness.step(); // CMP AL, BL (AL - BL would wrap)

    assert_eq!(harness.cpu.read_reg8(0), 0x10); // AL unchanged
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), false); // Not zero
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true); // Borrow occurred
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF), true); // Result negative
}

#[test]
fn test_cmp_r16_r16_equal() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1234; MOV BX, 0x1234; CMP AX, BX
    harness.load_program(&[0xB8, 0x34, 0x12, 0xBB, 0x34, 0x12, 0x3B, 0xC3], 0);

    harness.step(); // MOV AX, 0x1234
    harness.step(); // MOV BX, 0x1234
    harness.step(); // CMP AX, BX

    assert_eq!(harness.cpu.regs[0], 0x1234); // AX unchanged
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), true);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_cmp_r16_r16_greater() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x5000; MOV BX, 0x3000; CMP AX, BX
    harness.load_program(&[0xB8, 0x00, 0x50, 0xBB, 0x00, 0x30, 0x3B, 0xC3], 0);

    harness.step(); // MOV AX, 0x5000
    harness.step(); // MOV BX, 0x3000
    harness.step(); // CMP AX, BX

    assert_eq!(harness.cpu.regs[0], 0x5000); // AX unchanged
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), false);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_cmp_r16_r16_less() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1000; MOV BX, 0x2000; CMP AX, BX
    harness.load_program(&[0xB8, 0x00, 0x10, 0xBB, 0x00, 0x20, 0x3B, 0xC3], 0);

    harness.step(); // MOV AX, 0x1000
    harness.step(); // MOV BX, 0x2000
    harness.step(); // CMP AX, BX

    assert_eq!(harness.cpu.regs[0], 0x1000); // AX unchanged
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), false);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);
}

#[test]
fn test_cmp_al_imm8_equal() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x55; CMP AL, 0x55
    harness.load_program(&[0xB0, 0x55, 0x3C, 0x55], 0);

    harness.step(); // MOV AL, 0x55
    harness.step(); // CMP AL, 0x55

    assert_eq!(harness.cpu.read_reg8(0), 0x55); // AL unchanged
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), true);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_cmp_al_imm8_not_equal() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x55; CMP AL, 0x33
    harness.load_program(&[0xB0, 0x55, 0x3C, 0x33], 0);

    harness.step(); // MOV AL, 0x55
    harness.step(); // CMP AL, 0x33

    assert_eq!(harness.cpu.read_reg8(0), 0x55); // AL unchanged
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), false);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_cmp_ax_imm16_equal() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xABCD; CMP AX, 0xABCD
    harness.load_program(&[0xB8, 0xCD, 0xAB, 0x3D, 0xCD, 0xAB], 0);

    harness.step(); // MOV AX, 0xABCD
    harness.step(); // CMP AX, 0xABCD

    assert_eq!(harness.cpu.regs[0], 0xABCD); // AX unchanged
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), true);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_cmp_ax_imm16_not_equal() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xABCD; CMP AX, 0x1234
    harness.load_program(&[0xB8, 0xCD, 0xAB, 0x3D, 0x34, 0x12], 0);

    harness.step(); // MOV AX, 0xABCD
    harness.step(); // CMP AX, 0x1234

    assert_eq!(harness.cpu.regs[0], 0xABCD); // AX unchanged
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), false);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_cmp_does_not_modify_register() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x99; CMP AL, 0x11; verify AL is still 0x99
    harness.load_program(&[0xB0, 0x99, 0x3C, 0x11], 0);

    harness.step(); // MOV AL, 0x99
    harness.step(); // CMP AL, 0x11

    // Critical: CMP should NOT modify AL
    assert_eq!(harness.cpu.read_reg8(0), 0x99); // AL unchanged!
}

#[test]
fn test_cmp_with_conditional_jump() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x10; CMP AL, 0x20; JB skip; MOV AL, 0xFF; skip: NOP
    harness.load_program(
        &[
            0xB0, 0x10, // MOV AL, 0x10
            0x3C, 0x20, // CMP AL, 0x20
            0x72, 0x02, // JB +2 (skip MOV)
            0xB0, 0xFF, // MOV AL, 0xFF
            0x90, // NOP
        ],
        0,
    );

    harness.step(); // MOV AL, 0x10
    harness.step(); // CMP AL, 0x20 (sets CF because 0x10 < 0x20)

    // CF should be set (0x10 < 0x20)
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);

    harness.step(); // JB should jump (CF=1)
    harness.step(); // NOP

    // AL should still be 0x10 because we jumped over the MOV
    assert_eq!(harness.cpu.read_reg8(0), 0x10);
}

#[test]
fn test_cmp_zero_with_zero() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x00; CMP AL, 0x00
    harness.load_program(&[0xB0, 0x00, 0x3C, 0x00], 0);

    harness.step(); // MOV AL, 0x00
    harness.step(); // CMP AL, 0x00

    assert_eq!(harness.cpu.read_reg8(0), 0x00);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), true);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF), false);
}

// DAA (Decimal Adjust After Addition) tests

#[test]
fn test_daa_no_adjustment() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x12; DAA (both nibbles are valid BCD, no adjustment needed)
    harness.load_program(&[0xB0, 0x12, 0x27], 0);

    harness.step(); // MOV AL, 0x12
    harness.step(); // DAA

    assert_eq!(harness.cpu.read_reg8(0), 0x12); // AL unchanged
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF), false);
}

#[test]
fn test_daa_low_nibble_adjustment() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x1F; DAA (low nibble > 9, add 6)
    // Expected: 0x1F + 0x06 = 0x25
    harness.load_program(&[0xB0, 0x1F, 0x27], 0);

    harness.step(); // MOV AL, 0x1F
    harness.step(); // DAA

    assert_eq!(harness.cpu.read_reg8(0), 0x25); // AL = 0x1F + 6 = 0x25
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF), true);
}

#[test]
fn test_daa_high_nibble_adjustment() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xA5; DAA (high nibble > 9, add 0x60)
    // Expected: 0xA5 + 0x60 = 0x05, CF = 1
    harness.load_program(&[0xB0, 0xA5, 0x27], 0);

    harness.step(); // MOV AL, 0xA5
    harness.step(); // DAA

    assert_eq!(harness.cpu.read_reg8(0), 0x05); // AL = 0xA5 + 0x60 = 0x05 (wraps)
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF), false);
}

#[test]
fn test_daa_both_nibbles_adjustment() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x9F; DAA (both nibbles > 9)
    // Expected: 0x9F + 0x06 = 0xA5, then 0xA5 + 0x60 = 0x05, CF = 1
    harness.load_program(&[0xB0, 0x9F, 0x27], 0);

    harness.step(); // MOV AL, 0x9F
    harness.step(); // DAA

    assert_eq!(harness.cpu.read_reg8(0), 0x05); // AL = 0x9F + 6 + 0x60 = 0x05
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF), true);
}

#[test]
fn test_daa_with_af_set() {
    let mut harness = CpuHarness::new();
    // Set up a scenario where AF is already set
    // MOV AL, 0x08; ADD AL, 0x09 (sets AF); DAA
    // After ADD: AL = 0x11, AF = 1
    // DAA should add 6 because AF is set, even though low nibble = 1
    harness.load_program(&[0xB0, 0x08, 0x04, 0x09, 0x27], 0);

    harness.step(); // MOV AL, 0x08
    harness.step(); // ADD AL, 0x09 (AL = 0x11, AF = 1 because 8 + 9 caused carry from bit 3)

    // Verify AF is set after ADD
    assert_eq!(harness.cpu.read_reg8(0), 0x11);
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF));

    harness.step(); // DAA

    assert_eq!(harness.cpu.read_reg8(0), 0x17); // AL = 0x11 + 6 = 0x17
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF), true);
}

#[test]
fn test_daa_with_cf_set() {
    let mut harness = CpuHarness::new();
    // Set up a scenario where CF is already set
    // MOV AL, 0xFF; ADD AL, 0x01 (sets both CF and AF); DAA
    // After ADD: AL = 0x00, CF = 1, AF = 1 (because 0xF + 0x1 = 0x10, carry from bit 3)
    // DAA adds 6 (due to AF) + 0x60 (due to CF) = 0x66
    harness.load_program(&[0xB0, 0xFF, 0x04, 0x01, 0x27], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // ADD AL, 0x01 (AL = 0x00, CF = 1, AF = 1)

    // Verify CF is set after ADD
    assert_eq!(harness.cpu.read_reg8(0), 0x00);
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF));
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF));

    harness.step(); // DAA

    assert_eq!(harness.cpu.read_reg8(0), 0x66); // AL = 0x00 + 6 + 0x60 = 0x66
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);
}

#[test]
fn test_daa_bcd_addition_example() {
    let mut harness = CpuHarness::new();
    // Classic BCD example: 0x09 + 0x08 = 0x11, DAA -> 0x17 (representing 17 in BCD)
    // MOV AL, 0x09; ADD AL, 0x08; DAA
    harness.load_program(&[0xB0, 0x09, 0x04, 0x08, 0x27], 0);

    harness.step(); // MOV AL, 0x09
    harness.step(); // ADD AL, 0x08 (AL = 0x11, AF = 1)

    assert_eq!(harness.cpu.read_reg8(0), 0x11);

    harness.step(); // DAA

    assert_eq!(harness.cpu.read_reg8(0), 0x17); // Correct BCD result
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_daa_bcd_addition_with_carry() {
    let mut harness = CpuHarness::new();
    // BCD example with carry: 0x99 + 0x01 = 0x9A, DAA -> 0x00, CF = 1
    // MOV AL, 0x99; ADD AL, 0x01; DAA
    harness.load_program(&[0xB0, 0x99, 0x04, 0x01, 0x27], 0);

    harness.step(); // MOV AL, 0x99
    harness.step(); // ADD AL, 0x01 (AL = 0x9A)

    assert_eq!(harness.cpu.read_reg8(0), 0x9A);

    harness.step(); // DAA

    assert_eq!(harness.cpu.read_reg8(0), 0x00); // 0x9A + 6 + 0x60 = 0x00 (wraps)
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true); // Carry to next digit
}

#[test]
fn test_daa_flags_zero() {
    let mut harness = CpuHarness::new();
    // Test that ZF is set when result is zero
    // MOV AL, 0x9A; DAA (should produce 0x00 with ZF = 1)
    harness.load_program(&[0xB0, 0x9A, 0x27], 0);

    harness.step(); // MOV AL, 0x9A
    harness.step(); // DAA

    assert_eq!(harness.cpu.read_reg8(0), 0x00);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), true);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);
}

#[test]
fn test_daa_flags_sign() {
    let mut harness = CpuHarness::new();
    // Test that SF is set when result has bit 7 set
    // MOV AL, 0x7A; DAA (should produce 0x80, SF = 1)
    harness.load_program(&[0xB0, 0x7A, 0x27], 0);

    harness.step(); // MOV AL, 0x7A
    harness.step(); // DAA (0x7A + 0x60 = 0xDA, no wait...)

    // Actually, 0x7A: low nibble = A > 9, so add 6 -> 0x80
    // High nibble: 7 < 9, no adjustment needed
    assert_eq!(harness.cpu.read_reg8(0), 0x80);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF), true); // Sign flag set
}

#[test]
fn test_daa_flags_parity() {
    let mut harness = CpuHarness::new();
    // Test parity flag - result with even number of 1 bits sets PF
    // MOV AL, 0x03; DAA (result = 0x03, binary 0000_0011, 2 ones = even parity)
    harness.load_program(&[0xB0, 0x03, 0x27], 0);

    harness.step(); // MOV AL, 0x03
    harness.step(); // DAA

    assert_eq!(harness.cpu.read_reg8(0), 0x03);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::PF), true); // Even parity
}

#[test]
fn test_daa_edge_case_0x99() {
    let mut harness = CpuHarness::new();
    // Edge case: 0x99 (max valid BCD)
    // MOV AL, 0x99; DAA
    harness.load_program(&[0xB0, 0x99, 0x27], 0);

    harness.step(); // MOV AL, 0x99
    harness.step(); // DAA

    // 0x99: low nibble = 9 (OK), high nibble = 9 (OK), no adjustment
    assert_eq!(harness.cpu.read_reg8(0), 0x99);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF), false);
}

#[test]
fn test_daa_edge_case_0xa0() {
    let mut harness = CpuHarness::new();
    // Edge case: 0xA0 (high nibble > 9, low nibble = 0)
    // MOV AL, 0xA0; DAA
    harness.load_program(&[0xB0, 0xA0, 0x27], 0);

    harness.step(); // MOV AL, 0xA0
    harness.step(); // DAA

    // 0xA0 + 0x60 = 0x00, CF = 1
    assert_eq!(harness.cpu.read_reg8(0), 0x00);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), true);
}

#[test]
fn test_daa_low_nibble_equals_10() {
    let mut harness = CpuHarness::new();
    // Low nibble = 0xA (10), should trigger adjustment
    // MOV AL, 0x0A; DAA
    harness.load_program(&[0xB0, 0x0A, 0x27], 0);

    harness.step(); // MOV AL, 0x0A
    harness.step(); // DAA

    assert_eq!(harness.cpu.read_reg8(0), 0x10); // 0x0A + 6 = 0x10
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF), true);
}

#[test]
fn test_daa_multi_digit_bcd_addition() {
    let mut harness = CpuHarness::new();
    // Multi-digit BCD: 58 + 46 = 104 in decimal
    // In BCD: 0x58 + 0x46 = 0x9E, DAA -> 0x04, CF = 1 (representing 04 with carry)
    // MOV AL, 0x58; ADD AL, 0x46; DAA
    harness.load_program(&[0xB0, 0x58, 0x04, 0x46, 0x27], 0);

    harness.step(); // MOV AL, 0x58
    harness.step(); // ADD AL, 0x46 (AL = 0x9E)

    assert_eq!(harness.cpu.read_reg8(0), 0x9E);

    harness.step(); // DAA

    // 0x9E: low nibble E > 9, add 6 -> 0xA4
    // High nibble of original (9) < 9, but wait - we use old_AL
    // old_AL = 0x9E, which is > 0x99, so add 0x60
    // Final: 0x9E + 6 + 0x60 = 0x04, CF = 1
    assert_eq!(harness.cpu.read_reg8(0), 0x04);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);
}

// DAS (Decimal Adjust After Subtraction) tests

#[test]
fn test_das_no_adjustment() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x45; DAS (both nibbles are valid BCD, no adjustment needed)
    harness.load_program(&[0xB0, 0x45, 0x2F], 0);

    harness.step(); // MOV AL, 0x45
    harness.step(); // DAS

    assert_eq!(harness.cpu.read_reg8(0), 0x45); // AL unchanged
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF), false);
}

#[test]
fn test_das_low_nibble_adjustment() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x1F; DAS (low nibble > 9, subtract 6)
    // Expected: 0x1F - 0x06 = 0x19
    harness.load_program(&[0xB0, 0x1F, 0x2F], 0);

    harness.step(); // MOV AL, 0x1F
    harness.step(); // DAS

    assert_eq!(harness.cpu.read_reg8(0), 0x19); // AL = 0x1F - 6 = 0x19
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF), true);
}

#[test]
fn test_das_high_nibble_adjustment() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xA5; DAS (high nibble > 9, subtract 0x60)
    // Expected: 0xA5 - 0x60 = 0x45, CF = 1
    harness.load_program(&[0xB0, 0xA5, 0x2F], 0);

    harness.step(); // MOV AL, 0xA5
    harness.step(); // DAS

    assert_eq!(harness.cpu.read_reg8(0), 0x45); // AL = 0xA5 - 0x60 = 0x45
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF), false);
}

#[test]
fn test_das_both_nibbles_adjustment() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xAF; DAS (both nibbles > 9)
    // Expected: 0xAF - 0x06 = 0xA9, then 0xA9 - 0x60 = 0x49, CF = 1
    harness.load_program(&[0xB0, 0xAF, 0x2F], 0);

    harness.step(); // MOV AL, 0xAF
    harness.step(); // DAS

    assert_eq!(harness.cpu.read_reg8(0), 0x49); // AL = 0xAF - 6 - 0x60 = 0x49
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF), true);
}

#[test]
fn test_das_with_af_set() {
    let mut harness = CpuHarness::new();
    // Set up a scenario where AF is already set
    // MOV AL, 0x10; SUB AL, 0x01 (sets AF); DAS
    // After SUB: AL = 0x0F, AF = 1
    // DAS should subtract 6 because AF is set
    harness.load_program(&[0xB0, 0x10, 0x2C, 0x01, 0x2F], 0);

    harness.step(); // MOV AL, 0x10
    harness.step(); // SUB AL, 0x01 (AL = 0x0F, AF = 1 because 0 - 1 caused borrow from bit 4)

    // Verify AF is set after SUB
    assert_eq!(harness.cpu.read_reg8(0), 0x0F);
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF));

    harness.step(); // DAS

    assert_eq!(harness.cpu.read_reg8(0), 0x09); // AL = 0x0F - 6 = 0x09
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF), true);
}

#[test]
fn test_das_with_cf_set() {
    let mut harness = CpuHarness::new();
    // Set up a scenario where CF is already set
    // MOV AL, 0x00; SUB AL, 0x01 (sets CF and AF); DAS
    // After SUB: AL = 0xFF, CF = 1, AF = 1
    // DAS subtracts 6 (due to AF) + 0x60 (due to CF) = 0x66
    harness.load_program(&[0xB0, 0x00, 0x2C, 0x01, 0x2F], 0);

    harness.step(); // MOV AL, 0x00
    harness.step(); // SUB AL, 0x01 (AL = 0xFF, CF = 1, AF = 1)

    // Verify CF and AF are set after SUB
    assert_eq!(harness.cpu.read_reg8(0), 0xFF);
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF));
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF));

    harness.step(); // DAS

    assert_eq!(harness.cpu.read_reg8(0), 0x99); // AL = 0xFF - 6 - 0x60 = 0x99
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);
}

#[test]
fn test_das_bcd_subtraction_example() {
    let mut harness = CpuHarness::new();
    // Classic BCD example: 0x25 - 0x08 = 0x1D, DAS -> 0x17 (representing 17 in BCD)
    // MOV AL, 0x25; SUB AL, 0x08; DAS
    harness.load_program(&[0xB0, 0x25, 0x2C, 0x08, 0x2F], 0);

    harness.step(); // MOV AL, 0x25
    harness.step(); // SUB AL, 0x08 (AL = 0x1D, AF = 1)

    assert_eq!(harness.cpu.read_reg8(0), 0x1D);

    harness.step(); // DAS

    assert_eq!(harness.cpu.read_reg8(0), 0x17); // Correct BCD result
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_das_bcd_subtraction_with_borrow() {
    let mut harness = CpuHarness::new();
    // BCD example with borrow: 0x02 - 0x05 = 0xFD, DAS -> 0x97, CF = 1
    // MOV AL, 0x02; SUB AL, 0x05; DAS
    harness.load_program(&[0xB0, 0x02, 0x2C, 0x05, 0x2F], 0);

    harness.step(); // MOV AL, 0x02
    harness.step(); // SUB AL, 0x05 (AL = 0xFD, CF = 1, AF = 1)

    assert_eq!(harness.cpu.read_reg8(0), 0xFD);

    harness.step(); // DAS

    // 0xFD: low nibble D > 9, subtract 6 -> 0xF7
    // old_AL = 0xFD > 0x99, subtract 0x60 -> 0x97, CF = 1
    assert_eq!(harness.cpu.read_reg8(0), 0x97); // BCD borrow
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);
}

#[test]
fn test_das_flags_zero() {
    let mut harness = CpuHarness::new();
    // Test that ZF is set when result is zero
    // MOV AL, 0x66; DAS (should produce 0x00 with ZF = 1)
    harness.load_program(&[0xB0, 0x66, 0x2F], 0);

    harness.step(); // MOV AL, 0x66
    harness.step(); // DAS (0x66 - 0x66 = 0x00)

    // 0x66: low nibble = 6, no adjustment
    // high nibble = 6, no adjustment
    // Actually, this won't produce zero. Let me reconsider.
    // For zero, we need AL = 0x00 after DAS with no adjustments
    assert_eq!(harness.cpu.read_reg8(0), 0x66); // No adjustment
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), false);
}

#[test]
fn test_das_flags_zero_actual() {
    let mut harness = CpuHarness::new();
    // Test that ZF is set when result is zero
    // MOV AL, 0x00; DAS
    harness.load_program(&[0xB0, 0x00, 0x2F], 0);

    harness.step(); // MOV AL, 0x00
    harness.step(); // DAS

    assert_eq!(harness.cpu.read_reg8(0), 0x00);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF), true);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

#[test]
fn test_das_flags_sign() {
    let mut harness = CpuHarness::new();
    // Test that SF is set when result has bit 7 set
    // MOV AL, 0x8A; DAS (should produce 0x84, SF = 1)
    harness.load_program(&[0xB0, 0x8A, 0x2F], 0);

    harness.step(); // MOV AL, 0x8A
    harness.step(); // DAS (0x8A: low nibble A > 9, subtract 6 -> 0x84)

    assert_eq!(harness.cpu.read_reg8(0), 0x84);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF), true); // Sign flag set
}

#[test]
fn test_das_flags_parity() {
    let mut harness = CpuHarness::new();
    // Test parity flag - result with even number of 1 bits sets PF
    // MOV AL, 0x03; DAS (result = 0x03, binary 0000_0011, 2 ones = even parity)
    harness.load_program(&[0xB0, 0x03, 0x2F], 0);

    harness.step(); // MOV AL, 0x03
    harness.step(); // DAS

    assert_eq!(harness.cpu.read_reg8(0), 0x03);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::PF), true); // Even parity
}

#[test]
fn test_das_edge_case_0x99() {
    let mut harness = CpuHarness::new();
    // Edge case: 0x99 (max valid BCD)
    // MOV AL, 0x99; DAS
    harness.load_program(&[0xB0, 0x99, 0x2F], 0);

    harness.step(); // MOV AL, 0x99
    harness.step(); // DAS

    // 0x99: low nibble = 9 (OK), high nibble = 9 (OK), no adjustment
    assert_eq!(harness.cpu.read_reg8(0), 0x99);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF), false);
}

#[test]
fn test_das_edge_case_0xa0() {
    let mut harness = CpuHarness::new();
    // Edge case: 0xA0 (high nibble > 9, low nibble = 0)
    // MOV AL, 0xA0; DAS
    harness.load_program(&[0xB0, 0xA0, 0x2F], 0);

    harness.step(); // MOV AL, 0xA0
    harness.step(); // DAS

    // 0xA0 - 0x60 = 0x40, CF = 1
    assert_eq!(harness.cpu.read_reg8(0), 0x40);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true);
}

#[test]
fn test_das_low_nibble_equals_10() {
    let mut harness = CpuHarness::new();
    // Low nibble = 0xA (10), should trigger adjustment
    // MOV AL, 0x0A; DAS
    harness.load_program(&[0xB0, 0x0A, 0x2F], 0);

    harness.step(); // MOV AL, 0x0A
    harness.step(); // DAS

    assert_eq!(harness.cpu.read_reg8(0), 0x04); // 0x0A - 6 = 0x04
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::AF), true);
}

#[test]
fn test_das_multi_digit_bcd_subtraction() {
    let mut harness = CpuHarness::new();
    // Multi-digit BCD: 64 - 27 = 37 in decimal
    // In BCD: 0x64 - 0x27 = 0x3D, DAS -> 0x37
    // MOV AL, 0x64; SUB AL, 0x27; DAS
    harness.load_program(&[0xB0, 0x64, 0x2C, 0x27, 0x2F], 0);

    harness.step(); // MOV AL, 0x64
    harness.step(); // SUB AL, 0x27 (AL = 0x3D, AF = 1)

    assert_eq!(harness.cpu.read_reg8(0), 0x3D);

    harness.step(); // DAS

    // 0x3D: low nibble D > 9, subtract 6 -> 0x37
    assert_eq!(harness.cpu.read_reg8(0), 0x37);
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false);
}

// === Opcode 0xFE group tests (INC/DEC r/m8) ===

#[test]
fn test_inc_r8() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x42; INC AL
    harness.load_program(&[0xB0, 0x42, 0xFE, 0xC0], 0);

    harness.step(); // MOV AL, 0x42
    harness.step(); // INC AL

    assert_eq!(harness.cpu.read_reg8(0), 0x43); // AL
}

#[test]
fn test_inc_r8_overflow() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x7F; INC AL (overflow: 127 -> -128 in signed)
    harness.load_program(&[0xB0, 0x7F, 0xFE, 0xC0], 0);

    harness.step(); // MOV AL, 0x7F
    harness.step(); // INC AL

    assert_eq!(harness.cpu.read_reg8(0), 0x80); // AL
                                                // Check overflow flag is set (positive to negative)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF));
}

#[test]
fn test_inc_mem8() {
    let mut harness = CpuHarness::new();
    // MOV BX, 0x1000; MOV byte [BX], 0x10; INC byte [BX]
    harness.load_program(&[0xBB, 0x00, 0x10, 0xC6, 0x07, 0x10, 0xFE, 0x07], 0);

    harness.step(); // MOV BX, 0x1000
    harness.step(); // MOV byte [BX], 0x10
    harness.step(); // INC byte [BX]

    // Read from memory at DS:BX
    let ds = harness.cpu.read_seg(3); // DS
    let bx = harness.cpu.regs[3];
    let value = harness.mem.read_u8((ds as u32) * 16 + (bx as u32));
    assert_eq!(value, 0x11);
}

#[test]
fn test_dec_r8() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x42; DEC AL
    harness.load_program(&[0xB0, 0x42, 0xFE, 0xC8], 0);

    harness.step(); // MOV AL, 0x42
    harness.step(); // DEC AL

    assert_eq!(harness.cpu.read_reg8(0), 0x41); // AL
}

#[test]
fn test_dec_r8_underflow() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x80; DEC AL (overflow: -128 -> 127 in signed)
    harness.load_program(&[0xB0, 0x80, 0xFE, 0xC8], 0);

    harness.step(); // MOV AL, 0x80
    harness.step(); // DEC AL

    assert_eq!(harness.cpu.read_reg8(0), 0x7F); // AL
                                                // Check overflow flag is set (negative to positive)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF));
}

#[test]
fn test_dec_mem8() {
    let mut harness = CpuHarness::new();
    // MOV BX, 0x1000; MOV byte [BX], 0x20; DEC byte [BX]
    harness.load_program(&[0xBB, 0x00, 0x10, 0xC6, 0x07, 0x20, 0xFE, 0x0F], 0);

    harness.step(); // MOV BX, 0x1000
    harness.step(); // MOV byte [BX], 0x20
    harness.step(); // DEC byte [BX]

    // Read from memory at DS:BX
    let ds = harness.cpu.read_seg(3); // DS
    let bx = harness.cpu.regs[3];
    let value = harness.mem.read_u8((ds as u32) * 16 + (bx as u32));
    assert_eq!(value, 0x1F);
}

#[test]
fn test_inc_mem8_direct_address() {
    let mut harness = CpuHarness::new();
    // Write 0x42 to memory at [0x1234], then INC byte [0x1234]
    // This tests the 0xFF sentinel bug fix for group instructions with direct addressing
    harness.mem.write_u8(0x1234, 0x42);

    // INC byte [0x1234]: 0xFE 0x06 0x34 0x12
    // ModR/M: mod=00, reg=000 (INC), r/m=110 (direct addressing)
    harness.load_program(&[0xFE, 0x06, 0x34, 0x12], 0);

    harness.step(); // INC byte [0x1234]

    let value = harness.mem.read_u8(0x1234);
    assert_eq!(value, 0x43);
}

#[test]
fn test_dec_mem16_direct_address() {
    let mut harness = CpuHarness::new();
    // Write 0x1000 to memory at [0x5678], then DEC word [0x5678]
    // This tests the 0xFF sentinel bug fix for group instructions with direct addressing
    harness.mem.write_u16(0x5678, 0x1000);

    // DEC word [0x5678]: 0xFF 0x0E 0x78 0x56
    // ModR/M: mod=00, reg=001 (DEC), r/m=110 (direct addressing)
    harness.load_program(&[0xFF, 0x0E, 0x78, 0x56], 0);

    harness.step(); // DEC word [0x5678]

    let value = harness.mem.read_u16(0x5678);
    assert_eq!(value, 0x0FFF);
}

#[test]
fn test_inc_bl() {
    let mut harness = CpuHarness::new();
    // MOV BL, 0x99; INC BL
    harness.load_program(&[0xB3, 0x99, 0xFE, 0xC3], 0);

    harness.step(); // MOV BL, 0x99
    harness.step(); // INC BL

    assert_eq!(harness.cpu.read_reg8(3), 0x9A); // BL
}

#[test]
fn test_dec_dh() {
    let mut harness = CpuHarness::new();
    // MOV DH, 0x50; DEC DH
    harness.load_program(&[0xB6, 0x50, 0xFE, 0xCE], 0);

    harness.step(); // MOV DH, 0x50
    harness.step(); // DEC DH

    assert_eq!(harness.cpu.read_reg8(6), 0x4F); // DH
}

// === MUL instruction tests ===

#[test]
fn test_mul_r8_no_overflow() {
    let mut harness = CpuHarness::new();
    // MOV AL, 5; MOV BL, 3; MUL BL (AL * BL -> AX)
    // Expected: AL = 5 * 3 = 15, AX = 0x000F, CF = 0, OF = 0
    harness.load_program(&[0xB0, 0x05, 0xB3, 0x03, 0xF6, 0xE3], 0);

    harness.step(); // MOV AL, 5
    harness.step(); // MOV BL, 3
    harness.step(); // MUL BL

    assert_eq!(harness.cpu.regs[0], 0x000F); // AX = 15
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false); // CF clear (no overflow)
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF), false); // OF clear (no overflow)
}

#[test]
fn test_mul_r8_with_overflow() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x10 (16); MOV BL, 0x10 (16); MUL BL
    // Expected: AL * BL = 16 * 16 = 256 = 0x0100
    // AX = 0x0100, AH = 1 (non-zero), CF = 1, OF = 1
    harness.load_program(&[0xB0, 0x10, 0xB3, 0x10, 0xF6, 0xE3], 0);

    harness.step(); // MOV AL, 0x10
    harness.step(); // MOV BL, 0x10
    harness.step(); // MUL BL

    assert_eq!(harness.cpu.regs[0], 0x0100); // AX = 256
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true); // CF set (overflow)
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF), true); // OF set (overflow)
}

#[test]
fn test_mul_r8_max_value() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF (255); MOV BL, 0xFF (255); MUL BL
    // Expected: 255 * 255 = 65025 = 0xFE01
    // AX = 0xFE01, CF = 1, OF = 1
    harness.load_program(&[0xB0, 0xFF, 0xB3, 0xFF, 0xF6, 0xE3], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // MOV BL, 0xFF
    harness.step(); // MUL BL

    assert_eq!(harness.cpu.regs[0], 0xFE01); // AX = 65025
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true); // CF set
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF), true); // OF set
}

#[test]
fn test_mul_r8_by_zero() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x42; MOV BL, 0x00; MUL BL
    // Expected: AL * 0 = 0, AX = 0, CF = 0, OF = 0
    harness.load_program(&[0xB0, 0x42, 0xB3, 0x00, 0xF6, 0xE3], 0);

    harness.step(); // MOV AL, 0x42
    harness.step(); // MOV BL, 0x00
    harness.step(); // MUL BL

    assert_eq!(harness.cpu.regs[0], 0x0000); // AX = 0
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false); // CF clear
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF), false); // OF clear
}

#[test]
fn test_mul_r16_no_overflow() {
    let mut harness = CpuHarness::new();
    // MOV AX, 100; MOV BX, 200; MUL BX (AX * BX -> DX:AX)
    // Expected: 100 * 200 = 20000 = 0x4E20
    // DX:AX = 0x0000:4E20, CF = 0, OF = 0
    harness.load_program(&[0xB8, 0x64, 0x00, 0xBB, 0xC8, 0x00, 0xF7, 0xE3], 0);

    harness.step(); // MOV AX, 100
    harness.step(); // MOV BX, 200
    harness.step(); // MUL BX

    assert_eq!(harness.cpu.regs[0], 0x4E20); // AX = 20000 (low word)
    assert_eq!(harness.cpu.regs[2], 0x0000); // DX = 0 (high word)
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false); // CF clear
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF), false); // OF clear
}

#[test]
fn test_mul_r16_with_overflow() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1000 (4096); MOV BX, 0x1000 (4096); MUL BX
    // Expected: 4096 * 4096 = 16777216 = 0x01000000
    // DX:AX = 0x0100:0000, CF = 1, OF = 1
    harness.load_program(&[0xB8, 0x00, 0x10, 0xBB, 0x00, 0x10, 0xF7, 0xE3], 0);

    harness.step(); // MOV AX, 0x1000
    harness.step(); // MOV BX, 0x1000
    harness.step(); // MUL BX

    assert_eq!(harness.cpu.regs[0], 0x0000); // AX = 0 (low word)
    assert_eq!(harness.cpu.regs[2], 0x0100); // DX = 0x0100 (high word)
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true); // CF set
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF), true); // OF set
}

#[test]
fn test_mul_r16_max_value() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF (65535); MOV BX, 0xFFFF (65535); MUL BX
    // Expected: 65535 * 65535 = 4294836225 = 0xFFFE0001
    // DX:AX = 0xFFFE:0001
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0xBB, 0xFF, 0xFF, 0xF7, 0xE3], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // MOV BX, 0xFFFF
    harness.step(); // MUL BX

    assert_eq!(harness.cpu.regs[0], 0x0001); // AX (low word)
    assert_eq!(harness.cpu.regs[2], 0xFFFE); // DX (high word)
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true); // CF set
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF), true); // OF set
}

#[test]
fn test_mul_r16_by_zero() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1234; MOV BX, 0x0000; MUL BX
    // Expected: AX * 0 = 0, DX:AX = 0:0, CF = 0, OF = 0
    harness.load_program(&[0xB8, 0x34, 0x12, 0xBB, 0x00, 0x00, 0xF7, 0xE3], 0);

    harness.step(); // MOV AX, 0x1234
    harness.step(); // MOV BX, 0x0000
    harness.step(); // MUL BX

    assert_eq!(harness.cpu.regs[0], 0x0000); // AX = 0
    assert_eq!(harness.cpu.regs[2], 0x0000); // DX = 0
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), false); // CF clear
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF), false); // OF clear
}

#[test]
fn test_mul_r8_edge_case_128() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x80 (128); MOV BL, 0x02; MUL BL
    // Expected: 128 * 2 = 256 = 0x0100
    // AX = 0x0100, CF = 1, OF = 1
    harness.load_program(&[0xB0, 0x80, 0xB3, 0x02, 0xF6, 0xE3], 0);

    harness.step(); // MOV AL, 0x80
    harness.step(); // MOV BL, 0x02
    harness.step(); // MUL BL

    assert_eq!(harness.cpu.regs[0], 0x0100); // AX = 256
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF), true); // CF set
    assert_eq!(harness.cpu.get_flag(ezpc::cpu::Cpu::OF), true); // OF set
}

#[test]
fn test_not_r8() {
    let mut harness = CpuHarness::new();
    // NOT AL: 0x55 -> 0xAA
    harness.load_program(
        &[
            0xB0, 0x55, // MOV AL, 0x55
            0xF6, 0xD0, // NOT AL (ModR/M=D0: reg=010, rm=000 AL, mod=11)
        ],
        0,
    );

    harness.step(); // MOV AL, 0x55
    assert_eq!(harness.cpu.read_reg8(0), 0x55); // AL = 0x55

    harness.step(); // NOT AL
    assert_eq!(harness.cpu.read_reg8(0), 0xAA); // AL = 0xAA (inverted)
}

#[test]
fn test_not_r8_zero() {
    let mut harness = CpuHarness::new();
    // NOT AL: 0x00 -> 0xFF
    harness.load_program(
        &[
            0xB0, 0x00, // MOV AL, 0x00
            0xF6, 0xD0, // NOT AL
        ],
        0,
    );

    harness.step(); // MOV AL, 0x00
    harness.step(); // NOT AL
    assert_eq!(harness.cpu.read_reg8(0), 0xFF); // AL = 0xFF
}

#[test]
fn test_not_r8_all_ones() {
    let mut harness = CpuHarness::new();
    // NOT AL: 0xFF -> 0x00
    harness.load_program(
        &[
            0xB0, 0xFF, // MOV AL, 0xFF
            0xF6, 0xD0, // NOT AL
        ],
        0,
    );

    harness.step(); // MOV AL, 0xFF
    harness.step(); // NOT AL
    assert_eq!(harness.cpu.read_reg8(0), 0x00); // AL = 0x00
}

#[test]
fn test_not_r16() {
    let mut harness = CpuHarness::new();
    // NOT AX: 0x5555 -> 0xAAAA
    harness.load_program(
        &[
            0xB8, 0x55, 0x55, // MOV AX, 0x5555
            0xF7, 0xD0, // NOT AX (ModR/M=D0: reg=010, rm=000 AX, mod=11)
        ],
        0,
    );

    harness.step(); // MOV AX, 0x5555
    assert_eq!(harness.cpu.regs[0], 0x5555); // AX = 0x5555

    harness.step(); // NOT AX
    assert_eq!(harness.cpu.regs[0], 0xAAAA); // AX = 0xAAAA (inverted)
}

#[test]
fn test_not_r16_zero() {
    let mut harness = CpuHarness::new();
    // NOT BX: 0x0000 -> 0xFFFF
    harness.load_program(
        &[
            0xBB, 0x00, 0x00, // MOV BX, 0x0000
            0xF7, 0xD3, // NOT BX (ModR/M=D3: reg=010, rm=011 BX, mod=11)
        ],
        0,
    );

    harness.step(); // MOV BX, 0x0000
    harness.step(); // NOT BX
    assert_eq!(harness.cpu.regs[3], 0xFFFF); // BX = 0xFFFF
}

#[test]
fn test_not_r16_all_ones() {
    let mut harness = CpuHarness::new();
    // NOT CX: 0xFFFF -> 0x0000
    harness.load_program(
        &[
            0xB9, 0xFF, 0xFF, // MOV CX, 0xFFFF
            0xF7, 0xD1, // NOT CX (ModR/M=D1: reg=010, rm=001 CX, mod=11)
        ],
        0,
    );

    harness.step(); // MOV CX, 0xFFFF
    harness.step(); // NOT CX
    assert_eq!(harness.cpu.regs[1], 0x0000); // CX = 0x0000
}

#[test]
fn test_not_r16_pattern() {
    let mut harness = CpuHarness::new();
    // NOT DX: 0x1234 -> 0xEDCB
    harness.load_program(
        &[
            0xBA, 0x34, 0x12, // MOV DX, 0x1234
            0xF7, 0xD2, // NOT DX (ModR/M=D2: reg=010, rm=010 DX, mod=11)
        ],
        0,
    );

    harness.step(); // MOV DX, 0x1234
    harness.step(); // NOT DX
    assert_eq!(harness.cpu.regs[2], 0xEDCB); // DX = 0xEDCB
}

#[test]
fn test_not_m8() {
    let mut harness = CpuHarness::new();
    // Set BX to point to memory location
    harness.cpu.regs[3] = 0x0100; // BX = 0x0100

    // Write test value to memory
    harness.mem.write_u8(0x0100, 0xA5);

    // NOT byte [BX]: 0xA5 -> 0x5A
    harness.load_program(
        &[
            0xF6, 0x17, // NOT byte [BX] (ModR/M=17: reg=010, rm=111 [BX], mod=00)
        ],
        0,
    );

    harness.step(); // NOT byte [BX]
    assert_eq!(harness.mem.read_u8(0x0100), 0x5A); // Memory inverted: 0xA5 -> 0x5A
}

#[test]
fn test_not_m16() {
    let mut harness = CpuHarness::new();
    // Set BX to point to memory location
    harness.cpu.regs[3] = 0x0200; // BX = 0x0200

    // Write test value to memory
    harness.mem.write_u16(0x0200, 0x1234);

    // NOT word [BX]: 0x1234 -> 0xEDCB
    harness.load_program(
        &[
            0xF7, 0x17, // NOT word [BX] (ModR/M=17: reg=010, rm=111 [BX], mod=00)
        ],
        0,
    );

    harness.step(); // NOT word [BX]
    assert_eq!(harness.mem.read_u16(0x0200), 0xEDCB); // Memory inverted: 0x1234 -> 0xEDCB
}

#[test]
fn test_not_does_not_affect_flags() {
    let mut harness = CpuHarness::new();
    // Set all flags to known state
    harness.cpu.set_flags(0xFFFF); // Set all flags

    // NOT AL
    harness.load_program(
        &[
            0xB0, 0x55, // MOV AL, 0x55
            0xF6, 0xD0, // NOT AL
        ],
        0,
    );

    harness.step(); // MOV AL, 0x55
    let flags_before = harness.cpu.get_flags();

    harness.step(); // NOT AL
    let flags_after = harness.cpu.get_flags();

    // Flags should be unchanged
    assert_eq!(flags_after, flags_before);
}

#[test]
fn test_not_m8_with_displacement() {
    let mut harness = CpuHarness::new();
    // Set BX to base address
    harness.cpu.regs[3] = 0x0100; // BX = 0x0100

    // Write test value to memory at BX+0x10
    harness.mem.write_u8(0x0110, 0x3C);

    // NOT byte [BX+0x10]: 0x3C -> 0xC3
    harness.load_program(
        &[
            0xF6, 0x57,
            0x10, // NOT byte [BX+0x10] (ModR/M=57: reg=010, rm=111 [BX+disp8], mod=01)
        ],
        0,
    );

    harness.step(); // NOT byte [BX+0x10]
    assert_eq!(harness.mem.read_u8(0x0110), 0xC3); // Memory inverted: 0x3C -> 0xC3
}

#[test]
fn test_not_double_inversion() {
    let mut harness = CpuHarness::new();
    // NOT AL twice should return to original value
    harness.load_program(
        &[
            0xB0, 0x42, // MOV AL, 0x42
            0xF6, 0xD0, // NOT AL (0x42 -> 0xBD)
            0xF6, 0xD0, // NOT AL (0xBD -> 0x42)
        ],
        0,
    );

    harness.step(); // MOV AL, 0x42
    assert_eq!(harness.cpu.read_reg8(0), 0x42);

    harness.step(); // NOT AL
    assert_eq!(harness.cpu.read_reg8(0), 0xBD);

    harness.step(); // NOT AL again
    assert_eq!(harness.cpu.read_reg8(0), 0x42); // Back to original value
}

#[test]
fn test_aam_basic() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x23; AAM 0x0A
    // 0x23 = 35 decimal, so 35 / 10 = 3 (AH), 35 % 10 = 5 (AL)
    harness.load_program(
        &[
            0xB0, 0x23, // MOV AL, 0x23 (35 decimal)
            0xD4, 0x0A, // AAM 0x0A (divide by 10)
        ],
        0,
    );

    harness.step(); // MOV AL, 0x23
    assert_eq!(harness.cpu.read_reg8(0), 0x23); // AL = 0x23

    harness.step(); // AAM 0x0A
    assert_eq!(harness.cpu.read_reg8(0), 5); // AL = 5 (35 % 10)
    assert_eq!(harness.cpu.read_reg8(4), 3); // AH = 3 (35 / 10)
}

#[test]
fn test_aam_zero_result() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x00; AAM 0x0A
    harness.load_program(
        &[
            0xB0, 0x00, // MOV AL, 0x00
            0xD4, 0x0A, // AAM 0x0A
        ],
        0,
    );

    harness.step(); // MOV AL, 0x00
    harness.step(); // AAM 0x0A

    assert_eq!(harness.cpu.read_reg8(0), 0); // AL = 0
    assert_eq!(harness.cpu.read_reg8(4), 0); // AH = 0
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set
}

#[test]
fn test_aam_max_value() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; AAM 0x0A
    // 0xFF = 255 decimal, so 255 / 10 = 25 (AH), 255 % 10 = 5 (AL)
    harness.load_program(
        &[
            0xB0, 0xFF, // MOV AL, 0xFF (255 decimal)
            0xD4, 0x0A, // AAM 0x0A
        ],
        0,
    );

    harness.step(); // MOV AL, 0xFF
    harness.step(); // AAM 0x0A

    assert_eq!(harness.cpu.read_reg8(0), 5); // AL = 5 (255 % 10)
    assert_eq!(harness.cpu.read_reg8(4), 25); // AH = 25 (255 / 10)
}

#[test]
fn test_aam_different_base() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x20; AAM 0x10 (divide by 16)
    // 0x20 = 32 decimal, so 32 / 16 = 2 (AH), 32 % 16 = 0 (AL)
    harness.load_program(
        &[
            0xB0, 0x20, // MOV AL, 0x20 (32 decimal)
            0xD4, 0x10, // AAM 0x10 (divide by 16)
        ],
        0,
    );

    harness.step(); // MOV AL, 0x20
    harness.step(); // AAM 0x10

    assert_eq!(harness.cpu.read_reg8(0), 0); // AL = 0 (32 % 16)
    assert_eq!(harness.cpu.read_reg8(4), 2); // AH = 2 (32 / 16)
}

#[test]
fn test_aad_basic() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x0305; AAD 0x0A
    // AH=3, AL=5, so result = 3 * 10 + 5 = 35 (0x23) in AL, AH=0
    harness.load_program(
        &[
            0xB8, 0x05, 0x03, // MOV AX, 0x0305 (AH=3, AL=5)
            0xD5, 0x0A, // AAD 0x0A (multiply by 10)
        ],
        0,
    );

    harness.step(); // MOV AX, 0x0305
    assert_eq!(harness.cpu.read_reg8(0), 0x05); // AL = 5
    assert_eq!(harness.cpu.read_reg8(4), 0x03); // AH = 3

    harness.step(); // AAD 0x0A
    assert_eq!(harness.cpu.read_reg8(0), 35); // AL = 35 (0x23)
    assert_eq!(harness.cpu.read_reg8(4), 0); // AH = 0
}

#[test]
fn test_aad_zero_ah() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x0009; AAD 0x0A
    // AH=0, AL=9, so result = 0 * 10 + 9 = 9 in AL, AH=0
    harness.load_program(
        &[
            0xB8, 0x09, 0x00, // MOV AX, 0x0009 (AH=0, AL=9)
            0xD5, 0x0A, // AAD 0x0A
        ],
        0,
    );

    harness.step(); // MOV AX, 0x0009
    harness.step(); // AAD 0x0A

    assert_eq!(harness.cpu.read_reg8(0), 9); // AL = 9
    assert_eq!(harness.cpu.read_reg8(4), 0); // AH = 0
}

#[test]
fn test_aad_max_value() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x0909; AAD 0x0A
    // AH=9, AL=9, so result = 9 * 10 + 9 = 99 (0x63) in AL, AH=0
    harness.load_program(
        &[
            0xB8, 0x09, 0x09, // MOV AX, 0x0909 (AH=9, AL=9)
            0xD5, 0x0A, // AAD 0x0A
        ],
        0,
    );

    harness.step(); // MOV AX, 0x0909
    harness.step(); // AAD 0x0A

    assert_eq!(harness.cpu.read_reg8(0), 99); // AL = 99 (0x63)
    assert_eq!(harness.cpu.read_reg8(4), 0); // AH = 0
}

#[test]
fn test_aad_different_base() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x0204; AAD 0x10 (base 16)
    // AH=2, AL=4, so result = 2 * 16 + 4 = 36 (0x24) in AL, AH=0
    harness.load_program(
        &[
            0xB8, 0x04, 0x02, // MOV AX, 0x0204 (AH=2, AL=4)
            0xD5, 0x10, // AAD 0x10 (multiply by 16)
        ],
        0,
    );

    harness.step(); // MOV AX, 0x0204
    harness.step(); // AAD 0x10

    assert_eq!(harness.cpu.read_reg8(0), 36); // AL = 36 (0x24)
    assert_eq!(harness.cpu.read_reg8(4), 0); // AH = 0
}

#[test]
fn test_aad_overflow_wrapping() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1A14; AAD 0x0A
    // AH=0x1A (26), AL=0x14 (20), so result = 26 * 10 + 20 = 280
    // 280 wraps to 24 in u8 (280 % 256 = 24)
    harness.load_program(
        &[
            0xB8, 0x14, 0x1A, // MOV AX, 0x1A14 (AH=26, AL=20)
            0xD5, 0x0A, // AAD 0x0A
        ],
        0,
    );

    harness.step(); // MOV AX, 0x1A14
    harness.step(); // AAD 0x0A

    assert_eq!(harness.cpu.read_reg8(0), 24); // AL = 24 (280 % 256)
    assert_eq!(harness.cpu.read_reg8(4), 0); // AH = 0
}

#[test]
fn test_aam_aad_roundtrip() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x3B; AAM 0x0A; AAD 0x0A
    // This should convert to BCD and back
    // 0x3B = 59 decimal
    // After AAM: AH=5, AL=9
    // After AAD: AL=59, AH=0
    harness.load_program(
        &[
            0xB0, 0x3B, // MOV AL, 0x3B (59 decimal)
            0xD4, 0x0A, // AAM 0x0A
            0xD5, 0x0A, // AAD 0x0A
        ],
        0,
    );

    harness.step(); // MOV AL, 0x3B
    assert_eq!(harness.cpu.read_reg8(0), 0x3B);

    harness.step(); // AAM 0x0A
    assert_eq!(harness.cpu.read_reg8(4), 5); // AH = 5
    assert_eq!(harness.cpu.read_reg8(0), 9); // AL = 9

    harness.step(); // AAD 0x0A
    assert_eq!(harness.cpu.read_reg8(0), 59); // AL = 59
    assert_eq!(harness.cpu.read_reg8(4), 0); // AH = 0
}
