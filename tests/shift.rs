//! Tests for shift and rotate instructions (SHL, SHR, SAR, ROL, ROR, RCL, RCR)

use ezpc::cpu::{Cpu, CpuHarness};

// ===== SHL (Shift Left) Tests =====

#[test]
fn test_shl_r8_1() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x55; SHL AL, 1
    harness.load_program(&[0xB0, 0x55, 0xD0, 0xE0], 0);

    harness.step(); // MOV AL, 0x55
    assert_eq!(harness.cpu.read_reg8(0), 0x55); // AL = 0x55 (01010101)

    harness.step(); // SHL AL, 1
    assert_eq!(harness.cpu.read_reg8(0), 0xAA); // AL = 0xAA (10101010)
    assert_eq!(harness.cpu.get_flag(Cpu::CF), false); // CF = 0
}

#[test]
fn test_shl_r8_1_with_carry() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x81; SHL AL, 1
    harness.load_program(&[0xB0, 0x81, 0xD0, 0xE0], 0);

    harness.step(); // MOV AL, 0x81
    harness.step(); // SHL AL, 1
    assert_eq!(harness.cpu.read_reg8(0), 0x02); // AL = 0x02
    assert_eq!(harness.cpu.get_flag(Cpu::CF), true); // CF = 1 (bit 7 shifted out)
}

#[test]
fn test_shl_r16_1() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1234; SHL AX, 1
    harness.load_program(&[0xB8, 0x34, 0x12, 0xD1, 0xE0], 0);

    harness.step(); // MOV AX, 0x1234
    harness.step(); // SHL AX, 1
    assert_eq!(harness.cpu.regs[0], 0x2468); // AX = 0x2468
    assert_eq!(harness.cpu.get_flag(Cpu::CF), false);
}

#[test]
fn test_shl_r8_cl() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x01; MOV CL, 4; SHL AL, CL
    harness.load_program(&[0xB0, 0x01, 0xB1, 0x04, 0xD2, 0xE0], 0);

    harness.step(); // MOV AL, 0x01
    harness.step(); // MOV CL, 4
    harness.step(); // SHL AL, CL
    assert_eq!(harness.cpu.read_reg8(0), 0x10); // AL = 0x10 (shifted left by 4)
}

#[test]
fn test_shl_r16_cl() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x0001; MOV CL, 8; SHL AX, CL
    harness.load_program(&[0xB8, 0x01, 0x00, 0xB1, 0x08, 0xD3, 0xE0], 0);

    harness.step(); // MOV AX, 0x0001
    harness.step(); // MOV CL, 8
    harness.step(); // SHL AX, CL
    assert_eq!(harness.cpu.regs[0], 0x0100); // AX = 0x0100 (shifted left by 8)
}

// ===== SHR (Shift Right Logical) Tests =====

#[test]
fn test_shr_r8_1() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xAA; SHR AL, 1
    harness.load_program(&[0xB0, 0xAA, 0xD0, 0xE8], 0);

    harness.step(); // MOV AL, 0xAA
    harness.step(); // SHR AL, 1
    assert_eq!(harness.cpu.read_reg8(0), 0x55); // AL = 0x55 (10101010 >> 1)
    assert_eq!(harness.cpu.get_flag(Cpu::CF), false);
}

#[test]
fn test_shr_r8_1_with_carry() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x03; SHR AL, 1
    harness.load_program(&[0xB0, 0x03, 0xD0, 0xE8], 0);

    harness.step(); // MOV AL, 0x03
    harness.step(); // SHR AL, 1
    assert_eq!(harness.cpu.read_reg8(0), 0x01); // AL = 0x01
    assert_eq!(harness.cpu.get_flag(Cpu::CF), true); // CF = 1 (bit 0 shifted out)
}

#[test]
fn test_shr_r16_1() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x8000; SHR AX, 1
    harness.load_program(&[0xB8, 0x00, 0x80, 0xD1, 0xE8], 0);

    harness.step(); // MOV AX, 0x8000
    harness.step(); // SHR AX, 1
    assert_eq!(harness.cpu.regs[0], 0x4000); // AX = 0x4000 (logical shift)
    assert_eq!(harness.cpu.get_flag(Cpu::CF), false);
}

#[test]
fn test_shr_r8_cl() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x80; MOV CL, 4; SHR AL, CL
    harness.load_program(&[0xB0, 0x80, 0xB1, 0x04, 0xD2, 0xE8], 0);

    harness.step(); // MOV AL, 0x80
    harness.step(); // MOV CL, 4
    harness.step(); // SHR AL, CL
    assert_eq!(harness.cpu.read_reg8(0), 0x08); // AL = 0x08 (shifted right by 4)
}

// ===== SAR (Shift Arithmetic Right) Tests =====

#[test]
fn test_sar_r8_1_positive() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x40; SAR AL, 1
    harness.load_program(&[0xB0, 0x40, 0xD0, 0xF8], 0);

    harness.step(); // MOV AL, 0x40
    harness.step(); // SAR AL, 1
    assert_eq!(harness.cpu.read_reg8(0), 0x20); // AL = 0x20 (positive number)
}

#[test]
fn test_sar_r8_1_negative() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x80; SAR AL, 1
    harness.load_program(&[0xB0, 0x80, 0xD0, 0xF8], 0);

    harness.step(); // MOV AL, 0x80 (negative: -128)
    harness.step(); // SAR AL, 1
    assert_eq!(harness.cpu.read_reg8(0), 0xC0); // AL = 0xC0 (sign extended: -64)
}

#[test]
fn test_sar_r16_1_negative() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x8000; SAR AX, 1
    harness.load_program(&[0xB8, 0x00, 0x80, 0xD1, 0xF8], 0);

    harness.step(); // MOV AX, 0x8000 (negative)
    harness.step(); // SAR AX, 1
    assert_eq!(harness.cpu.regs[0], 0xC000); // AX = 0xC000 (sign extended)
}

#[test]
fn test_sar_r8_cl() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; MOV CL, 3; SAR AL, CL
    harness.load_program(&[0xB0, 0xFF, 0xB1, 0x03, 0xD2, 0xF8], 0);

    harness.step(); // MOV AL, 0xFF (-1)
    harness.step(); // MOV CL, 3
    harness.step(); // SAR AL, CL
    assert_eq!(harness.cpu.read_reg8(0), 0xFF); // AL = 0xFF (still -1)
}

// ===== ROL (Rotate Left) Tests =====

#[test]
fn test_rol_r8_1() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x81; ROL AL, 1
    harness.load_program(&[0xB0, 0x81, 0xD0, 0xC0], 0);

    harness.step(); // MOV AL, 0x81 (10000001)
    harness.step(); // ROL AL, 1
    assert_eq!(harness.cpu.read_reg8(0), 0x03); // AL = 0x03 (00000011)
    assert_eq!(harness.cpu.get_flag(Cpu::CF), true); // CF = 1
}

#[test]
fn test_rol_r16_1() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x8001; ROL AX, 1
    harness.load_program(&[0xB8, 0x01, 0x80, 0xD1, 0xC0], 0);

    harness.step(); // MOV AX, 0x8001
    harness.step(); // ROL AX, 1
    assert_eq!(harness.cpu.regs[0], 0x0003); // AX = 0x0003
    assert_eq!(harness.cpu.get_flag(Cpu::CF), true);
}

#[test]
fn test_rol_r8_cl() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x80; MOV CL, 2; ROL AL, CL
    harness.load_program(&[0xB0, 0x80, 0xB1, 0x02, 0xD2, 0xC0], 0);

    harness.step(); // MOV AL, 0x80
    harness.step(); // MOV CL, 2
    harness.step(); // ROL AL, CL
    assert_eq!(harness.cpu.read_reg8(0), 0x02); // AL = 0x02
}

// ===== ROR (Rotate Right) Tests =====

#[test]
fn test_ror_r8_1() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x81; ROR AL, 1
    harness.load_program(&[0xB0, 0x81, 0xD0, 0xC8], 0);

    harness.step(); // MOV AL, 0x81 (10000001)
    harness.step(); // ROR AL, 1
    assert_eq!(harness.cpu.read_reg8(0), 0xC0); // AL = 0xC0 (11000000)
    assert_eq!(harness.cpu.get_flag(Cpu::CF), true); // CF = 1
}

#[test]
fn test_ror_r16_1() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x8001; ROR AX, 1
    harness.load_program(&[0xB8, 0x01, 0x80, 0xD1, 0xC8], 0);

    harness.step(); // MOV AX, 0x8001
    harness.step(); // ROR AX, 1
    assert_eq!(harness.cpu.regs[0], 0xC000); // AX = 0xC000
    assert_eq!(harness.cpu.get_flag(Cpu::CF), true);
}

// ===== RCL (Rotate Through Carry Left) Tests =====

#[test]
fn test_rcl_r8_1() {
    let mut harness = CpuHarness::new();
    // STC; MOV AL, 0x80; RCL AL, 1
    harness.load_program(&[0xF9, 0xB0, 0x80, 0xD0, 0xD0], 0);

    harness.step(); // STC (set carry flag)
    harness.step(); // MOV AL, 0x80
    harness.step(); // RCL AL, 1
    assert_eq!(harness.cpu.read_reg8(0), 0x01); // AL = 0x01 (carry rotated in)
    assert_eq!(harness.cpu.get_flag(Cpu::CF), true); // CF = 1 (bit 7 rotated out)
}

#[test]
fn test_rcl_r8_1_no_carry() {
    let mut harness = CpuHarness::new();
    // CLC; MOV AL, 0x40; RCL AL, 1
    harness.load_program(&[0xF8, 0xB0, 0x40, 0xD0, 0xD0], 0);

    harness.step(); // CLC (clear carry flag)
    harness.step(); // MOV AL, 0x40
    harness.step(); // RCL AL, 1
    assert_eq!(harness.cpu.read_reg8(0), 0x80); // AL = 0x80
    assert_eq!(harness.cpu.get_flag(Cpu::CF), false); // CF = 0
}

// ===== RCR (Rotate Through Carry Right) Tests =====

#[test]
fn test_rcr_r8_1() {
    let mut harness = CpuHarness::new();
    // STC; MOV AL, 0x01; RCR AL, 1
    harness.load_program(&[0xF9, 0xB0, 0x01, 0xD0, 0xD8], 0);

    harness.step(); // STC (set carry flag)
    harness.step(); // MOV AL, 0x01
    harness.step(); // RCR AL, 1
    assert_eq!(harness.cpu.read_reg8(0), 0x80); // AL = 0x80 (carry rotated in)
    assert_eq!(harness.cpu.get_flag(Cpu::CF), true); // CF = 1 (bit 0 rotated out)
}

#[test]
fn test_rcr_r8_1_no_carry() {
    let mut harness = CpuHarness::new();
    // CLC; MOV AL, 0x02; RCR AL, 1
    harness.load_program(&[0xF8, 0xB0, 0x02, 0xD0, 0xD8], 0);

    harness.step(); // CLC (clear carry flag)
    harness.step(); // MOV AL, 0x02
    harness.step(); // RCR AL, 1
    assert_eq!(harness.cpu.read_reg8(0), 0x01); // AL = 0x01
    assert_eq!(harness.cpu.get_flag(Cpu::CF), false); // CF = 0
}

// ===== Additional Tests =====

#[test]
fn test_shl_sets_zero_flag() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x80; MOV CL, 1; SHL AL, CL
    harness.load_program(&[0xB0, 0x80, 0xB1, 0x01, 0xD2, 0xE0], 0);

    harness.step(); // MOV AL, 0x80
    harness.step(); // MOV CL, 1
    harness.step(); // SHL AL, CL
    assert_eq!(harness.cpu.read_reg8(0), 0x00); // AL = 0
    assert_eq!(harness.cpu.get_flag(Cpu::ZF), true); // ZF = 1
}

#[test]
fn test_shr_sets_zero_flag() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x01; SHR AL, 1
    harness.load_program(&[0xB0, 0x01, 0xD0, 0xE8], 0);

    harness.step(); // MOV AL, 0x01
    harness.step(); // SHR AL, 1
    assert_eq!(harness.cpu.read_reg8(0), 0x00); // AL = 0
    assert_eq!(harness.cpu.get_flag(Cpu::ZF), true); // ZF = 1
}
