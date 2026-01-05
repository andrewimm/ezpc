//! Logical operation tests (AND, OR, XOR, NOT, etc.)

use ezpc::cpu::CpuHarness;

#[test]
fn test_and_r8_imm() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; AND AL, 0x0F
    harness.load_program(&[0xB0, 0xFF, 0x24, 0x0F], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // AND AL, 0x0F

    assert_eq!(harness.cpu.read_reg8(0), 0x0F); // AL = 0x0F
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_and_r16_imm() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; AND AX, 0x00FF
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0x25, 0xFF, 0x00], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // AND AX, 0x00FF

    assert_eq!(harness.cpu.regs[0], 0x00FF); // AX = 0x00FF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_and_zero_result() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x0F; AND AL, 0xF0
    harness.load_program(&[0xB0, 0x0F, 0x24, 0xF0], 0);

    harness.step(); // MOV AL, 0x0F
    harness.step(); // AND AL, 0xF0

    assert_eq!(harness.cpu.read_reg8(0), 0x00); // AL = 0
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_and_sign_flag() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; AND AL, 0x80
    harness.load_program(&[0xB0, 0xFF, 0x24, 0x80], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // AND AL, 0x80

    assert_eq!(harness.cpu.read_reg8(0), 0x80); // AL = 0x80
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set (bit 7 is 1)
}

#[test]
fn test_and_parity_flag() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; AND AL, 0x03
    harness.load_program(&[0xB0, 0xFF, 0x24, 0x03], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // AND AL, 0x03

    assert_eq!(harness.cpu.read_reg8(0), 0x03); // AL = 0x03 (two bits set, even parity)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::PF)); // PF should be set (even number of bits)
}

#[test]
fn test_and_r_rm_byte() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; MOV CL, 0x0F; AND AL, CL
    harness.load_program(&[0xB0, 0xFF, 0xB1, 0x0F, 0x22, 0xC1], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // MOV CL, 0x0F
    harness.step(); // AND AL, CL (opcode 0x22, ModR/M 0xC1)

    assert_eq!(harness.cpu.read_reg8(0), 0x0F); // AL = 0x0F
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
}

#[test]
fn test_and_r_rm_word() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; MOV CX, 0x00FF; AND AX, CX
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0xB9, 0xFF, 0x00, 0x23, 0xC1], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // MOV CX, 0x00FF
    harness.step(); // AND AX, CX (opcode 0x23, ModR/M 0xC1)

    assert_eq!(harness.cpu.regs[0], 0x00FF); // AX = 0x00FF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
}

#[test]
fn test_and_rm_r_byte() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; MOV CL, 0x0F; AND CL, AL
    harness.load_program(&[0xB0, 0xFF, 0xB1, 0x0F, 0x20, 0xC1], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // MOV CL, 0x0F
    harness.step(); // AND CL, AL (opcode 0x20, ModR/M 0xC1)

    assert_eq!(harness.cpu.read_reg8(1), 0x0F); // CL = 0x0F
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
}

#[test]
fn test_and_rm_r_word() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; MOV CX, 0x00FF; AND CX, AX
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0xB9, 0xFF, 0x00, 0x21, 0xC1], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // MOV CX, 0x00FF
    harness.step(); // AND CX, AX (opcode 0x21, ModR/M 0xC1)

    assert_eq!(harness.cpu.regs[1], 0x00FF); // CX = 0x00FF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
}

#[test]
fn test_and_rm8_imm8() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; AND AL, 0x0F (using Group 0x80, reg=4)
    harness.load_program(&[0xB0, 0xFF, 0x80, 0xE0, 0x0F], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // AND AL, 0x0F (opcode 0x80, ModR/M 0xE0, imm8 0x0F)

    assert_eq!(harness.cpu.read_reg8(0), 0x0F); // AL = 0x0F
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_and_rm16_imm16() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; AND AX, 0x00FF (using Group 0x81, reg=4)
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0x81, 0xE0, 0xFF, 0x00], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // AND AX, 0x00FF (opcode 0x81, ModR/M 0xE0, imm16 0x00FF)

    assert_eq!(harness.cpu.regs[0], 0x00FF); // AX = 0x00FF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_and_rm16_imm8() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; AND AX, 0x0F (using Group 0x83, reg=4, sign-extended)
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0x83, 0xE0, 0x0F], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // AND AX, 0x0F (opcode 0x83, ModR/M 0xE0, imm8 0x0F sign-extended to 0x000F)

    assert_eq!(harness.cpu.regs[0], 0x000F); // AX = 0x000F
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_and_rm16_imm8_sign_extended() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; AND AX, 0xFF (using Group 0x83, reg=4, sign-extended to 0xFFFF)
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0x83, 0xE0, 0xFF], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // AND AX, 0xFF (opcode 0x83, ModR/M 0xE0, imm8 0xFF sign-extended to 0xFFFF)

    assert_eq!(harness.cpu.regs[0], 0xFFFF); // AX = 0xFFFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set (bit 15 is 1)
}

#[test]
fn test_and_rm8_imm8_zero_result() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x0F; AND AL, 0xF0 (using Group 0x80, reg=4)
    harness.load_program(&[0xB0, 0x0F, 0x80, 0xE0, 0xF0], 0);

    harness.step(); // MOV AL, 0x0F
    harness.step(); // AND AL, 0xF0 (opcode 0x80, ModR/M 0xE0, imm8 0xF0)

    assert_eq!(harness.cpu.read_reg8(0), 0x00); // AL = 0
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_and_rm16_imm8_memory() {
    let mut harness = CpuHarness::new();
    // Write test value to memory
    harness.mem.write_u16(0x1000, 0xFFFF);
    // MOV BX, 0x1000; AND word [BX], 0x0F (using Group 0x83, reg=4)
    harness.load_program(&[0xBB, 0x00, 0x10, 0x83, 0x27, 0x0F], 0);

    harness.step(); // MOV BX, 0x1000
    harness.step(); // AND word [BX], 0x0F (opcode 0x83, ModR/M 0x27, imm8 0x0F)

    assert_eq!(harness.mem.read_u16(0x1000), 0x000F); // Memory = 0x000F
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_or_rm8_imm8() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x0F; OR AL, 0xF0 (using Group 0x80, reg=1)
    harness.load_program(&[0xB0, 0x0F, 0x80, 0xC8, 0xF0], 0);

    harness.step(); // MOV AL, 0x0F
    harness.step(); // OR AL, 0xF0 (opcode 0x80, ModR/M 0xC8, imm8 0xF0)

    assert_eq!(harness.cpu.read_reg8(0), 0xFF); // AL = 0xFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set
}

#[test]
fn test_or_rm16_imm16() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x00FF; OR AX, 0xFF00 (using Group 0x81, reg=1)
    harness.load_program(&[0xB8, 0xFF, 0x00, 0x81, 0xC8, 0x00, 0xFF], 0);

    harness.step(); // MOV AX, 0x00FF
    harness.step(); // OR AX, 0xFF00 (opcode 0x81, ModR/M 0xC8, imm16 0xFF00)

    assert_eq!(harness.cpu.regs[0], 0xFFFF); // AX = 0xFFFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set
}

#[test]
fn test_or_rm16_imm8() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x0F00; OR AX, 0x0F (using Group 0x83, reg=1, sign-extended)
    harness.load_program(&[0xB8, 0x00, 0x0F, 0x83, 0xC8, 0x0F], 0);

    harness.step(); // MOV AX, 0x0F00
    harness.step(); // OR AX, 0x0F (opcode 0x83, ModR/M 0xC8, imm8 0x0F sign-extended to 0x000F)

    assert_eq!(harness.cpu.regs[0], 0x0F0F); // AX = 0x0F0F
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_or_rm16_imm8_sign_extended() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x0F00; OR AX, 0xFF (using Group 0x83, reg=1, sign-extended to 0xFFFF)
    harness.load_program(&[0xB8, 0x00, 0x0F, 0x83, 0xC8, 0xFF], 0);

    harness.step(); // MOV AX, 0x0F00
    harness.step(); // OR AX, 0xFF (opcode 0x83, ModR/M 0xC8, imm8 0xFF sign-extended to 0xFFFF)

    assert_eq!(harness.cpu.regs[0], 0xFFFF); // AX = 0xFFFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set
}

#[test]
fn test_or_rm8_imm8_zero_result() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x00; OR AL, 0x00 (using Group 0x80, reg=1)
    harness.load_program(&[0xB0, 0x00, 0x80, 0xC8, 0x00], 0);

    harness.step(); // MOV AL, 0x00
    harness.step(); // OR AL, 0x00 (opcode 0x80, ModR/M 0xC8, imm8 0x00)

    assert_eq!(harness.cpu.read_reg8(0), 0x00); // AL = 0
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_or_rm16_imm8_memory() {
    let mut harness = CpuHarness::new();
    // Write test value to memory
    harness.mem.write_u16(0x1000, 0x0F00);
    // MOV BX, 0x1000; OR word [BX], 0x0F (using Group 0x83, reg=1)
    harness.load_program(&[0xBB, 0x00, 0x10, 0x83, 0x0F, 0x0F], 0);

    harness.step(); // MOV BX, 0x1000
    harness.step(); // OR word [BX], 0x0F (opcode 0x83, ModR/M 0x0F, imm8 0x0F)

    assert_eq!(harness.mem.read_u16(0x1000), 0x0F0F); // Memory = 0x0F0F
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_xor_rm8_imm8() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; XOR AL, 0x0F (using Group 0x80, reg=6)
    harness.load_program(&[0xB0, 0xFF, 0x80, 0xF0, 0x0F], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // XOR AL, 0x0F (opcode 0x80, ModR/M 0xF0, imm8 0x0F)

    assert_eq!(harness.cpu.read_reg8(0), 0xF0); // AL = 0xF0
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set
}

#[test]
fn test_xor_rm16_imm16() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; XOR AX, 0x00FF (using Group 0x81, reg=6)
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0x81, 0xF0, 0xFF, 0x00], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // XOR AX, 0x00FF (opcode 0x81, ModR/M 0xF0, imm16 0x00FF)

    assert_eq!(harness.cpu.regs[0], 0xFF00); // AX = 0xFF00
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set
}

#[test]
fn test_xor_rm16_imm8() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xF0F0; XOR AX, 0x0F (using Group 0x83, reg=6, sign-extended)
    harness.load_program(&[0xB8, 0xF0, 0xF0, 0x83, 0xF0, 0x0F], 0);

    harness.step(); // MOV AX, 0xF0F0
    harness.step(); // XOR AX, 0x0F (opcode 0x83, ModR/M 0xF0, imm8 0x0F sign-extended to 0x000F)

    assert_eq!(harness.cpu.regs[0], 0xF0FF); // AX = 0xF0FF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set
}

#[test]
fn test_xor_rm16_imm8_sign_extended() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xAAAA; XOR AX, 0xFF (using Group 0x83, reg=6, sign-extended to 0xFFFF)
    harness.load_program(&[0xB8, 0xAA, 0xAA, 0x83, 0xF0, 0xFF], 0);

    harness.step(); // MOV AX, 0xAAAA
    harness.step(); // XOR AX, 0xFF (opcode 0x83, ModR/M 0xF0, imm8 0xFF sign-extended to 0xFFFF)

    assert_eq!(harness.cpu.regs[0], 0x5555); // AX = 0x5555 (0xAAAA ^ 0xFFFF)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_xor_rm8_imm8_zero_result() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; XOR AL, 0xFF (using Group 0x80, reg=6)
    harness.load_program(&[0xB0, 0xFF, 0x80, 0xF0, 0xFF], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // XOR AL, 0xFF (opcode 0x80, ModR/M 0xF0, imm8 0xFF)

    assert_eq!(harness.cpu.read_reg8(0), 0x00); // AL = 0 (self XOR)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_xor_rm16_imm8_memory() {
    let mut harness = CpuHarness::new();
    // Write test value to memory
    harness.mem.write_u16(0x1000, 0xF0F0);
    // MOV BX, 0x1000; XOR word [BX], 0x0F (using Group 0x83, reg=6)
    harness.load_program(&[0xBB, 0x00, 0x10, 0x83, 0x37, 0x0F], 0);

    harness.step(); // MOV BX, 0x1000
    harness.step(); // XOR word [BX], 0x0F (opcode 0x83, ModR/M 0x37, imm8 0x0F)

    assert_eq!(harness.mem.read_u16(0x1000), 0xF0FF); // Memory = 0xF0FF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set
}

// OR instruction tests

#[test]
fn test_or_r8_imm() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x0F; OR AL, 0xF0
    harness.load_program(&[0xB0, 0x0F, 0x0C, 0xF0], 0);

    harness.step(); // MOV AL, 0x0F
    harness.step(); // OR AL, 0xF0

    assert_eq!(harness.cpu.read_reg8(0), 0xFF); // AL = 0xFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set (bit 7 is 1)
}

#[test]
fn test_or_r16_imm() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x00FF; OR AX, 0xFF00
    harness.load_program(&[0xB8, 0xFF, 0x00, 0x0D, 0x00, 0xFF], 0);

    harness.step(); // MOV AX, 0x00FF
    harness.step(); // OR AX, 0xFF00

    assert_eq!(harness.cpu.regs[0], 0xFFFF); // AX = 0xFFFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set
}

#[test]
fn test_or_zero_result() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x00; OR AL, 0x00
    harness.load_program(&[0xB0, 0x00, 0x0C, 0x00], 0);

    harness.step(); // MOV AL, 0x00
    harness.step(); // OR AL, 0x00

    assert_eq!(harness.cpu.read_reg8(0), 0x00); // AL = 0
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_or_sign_flag() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x00; OR AL, 0x80
    harness.load_program(&[0xB0, 0x00, 0x0C, 0x80], 0);

    harness.step(); // MOV AL, 0x00
    harness.step(); // OR AL, 0x80

    assert_eq!(harness.cpu.read_reg8(0), 0x80); // AL = 0x80
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set (bit 7 is 1)
}

#[test]
fn test_or_r_rm_byte() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x0F; MOV CL, 0xF0; OR AL, CL
    harness.load_program(&[0xB0, 0x0F, 0xB1, 0xF0, 0x0A, 0xC1], 0);

    harness.step(); // MOV AL, 0x0F
    harness.step(); // MOV CL, 0xF0
    harness.step(); // OR AL, CL (opcode 0x0A, ModR/M 0xC1)

    assert_eq!(harness.cpu.read_reg8(0), 0xFF); // AL = 0xFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
}

#[test]
fn test_or_r_rm_word() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x00FF; MOV CX, 0xFF00; OR AX, CX
    harness.load_program(&[0xB8, 0xFF, 0x00, 0xB9, 0x00, 0xFF, 0x0B, 0xC1], 0);

    harness.step(); // MOV AX, 0x00FF
    harness.step(); // MOV CX, 0xFF00
    harness.step(); // OR AX, CX (opcode 0x0B, ModR/M 0xC1)

    assert_eq!(harness.cpu.regs[0], 0xFFFF); // AX = 0xFFFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
}

#[test]
fn test_or_rm_r_byte() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x0F; MOV CL, 0xF0; OR CL, AL
    harness.load_program(&[0xB0, 0x0F, 0xB1, 0xF0, 0x08, 0xC1], 0);

    harness.step(); // MOV AL, 0x0F
    harness.step(); // MOV CL, 0xF0
    harness.step(); // OR CL, AL (opcode 0x08, ModR/M 0xC1)

    assert_eq!(harness.cpu.read_reg8(1), 0xFF); // CL = 0xFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
}

#[test]
fn test_or_rm_r_word() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x00FF; MOV CX, 0xFF00; OR CX, AX
    harness.load_program(&[0xB8, 0xFF, 0x00, 0xB9, 0x00, 0xFF, 0x09, 0xC1], 0);

    harness.step(); // MOV AX, 0x00FF
    harness.step(); // MOV CX, 0xFF00
    harness.step(); // OR CX, AX (opcode 0x09, ModR/M 0xC1)

    assert_eq!(harness.cpu.regs[1], 0xFFFF); // CX = 0xFFFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
}

// XOR instruction tests

#[test]
fn test_xor_r8_imm() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; XOR AL, 0x0F
    harness.load_program(&[0xB0, 0xFF, 0x34, 0x0F], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // XOR AL, 0x0F

    assert_eq!(harness.cpu.read_reg8(0), 0xF0); // AL = 0xF0
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set (bit 7 is 1)
}

#[test]
fn test_xor_r16_imm() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; XOR AX, 0x00FF
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0x35, 0xFF, 0x00], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // XOR AX, 0x00FF

    assert_eq!(harness.cpu.regs[0], 0xFF00); // AX = 0xFF00
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set
}

#[test]
fn test_xor_zero_result() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; XOR AL, 0xFF
    harness.load_program(&[0xB0, 0xFF, 0x34, 0xFF], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // XOR AL, 0xFF

    assert_eq!(harness.cpu.read_reg8(0), 0x00); // AL = 0 (XOR with itself)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_xor_self_register() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x55; XOR AL, AL (common idiom for zeroing a register)
    harness.load_program(&[0xB0, 0x55, 0x32, 0xC0], 0);

    harness.step(); // MOV AL, 0x55
    harness.step(); // XOR AL, AL (opcode 0x32, ModR/M 0xC0)

    assert_eq!(harness.cpu.read_reg8(0), 0x00); // AL = 0
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_xor_sign_flag() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; XOR AL, 0x7F
    harness.load_program(&[0xB0, 0xFF, 0x34, 0x7F], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // XOR AL, 0x7F

    assert_eq!(harness.cpu.read_reg8(0), 0x80); // AL = 0x80
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set (bit 7 is 1)
}

#[test]
fn test_xor_r_rm_byte() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xAA; MOV CL, 0x55; XOR AL, CL
    harness.load_program(&[0xB0, 0xAA, 0xB1, 0x55, 0x32, 0xC1], 0);

    harness.step(); // MOV AL, 0xAA
    harness.step(); // MOV CL, 0x55
    harness.step(); // XOR AL, CL (opcode 0x32, ModR/M 0xC1)

    assert_eq!(harness.cpu.read_reg8(0), 0xFF); // AL = 0xFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
}

#[test]
fn test_xor_r_rm_word() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xAAAA; MOV CX, 0x5555; XOR AX, CX
    harness.load_program(&[0xB8, 0xAA, 0xAA, 0xB9, 0x55, 0x55, 0x33, 0xC1], 0);

    harness.step(); // MOV AX, 0xAAAA
    harness.step(); // MOV CX, 0x5555
    harness.step(); // XOR AX, CX (opcode 0x33, ModR/M 0xC1)

    assert_eq!(harness.cpu.regs[0], 0xFFFF); // AX = 0xFFFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
}

#[test]
fn test_xor_rm_r_byte() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xAA; MOV CL, 0x55; XOR CL, AL
    harness.load_program(&[0xB0, 0xAA, 0xB1, 0x55, 0x30, 0xC1], 0);

    harness.step(); // MOV AL, 0xAA
    harness.step(); // MOV CL, 0x55
    harness.step(); // XOR CL, AL (opcode 0x30, ModR/M 0xC1)

    assert_eq!(harness.cpu.read_reg8(1), 0xFF); // CL = 0xFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
}

#[test]
fn test_xor_rm_r_word() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xAAAA; MOV CX, 0x5555; XOR CX, AX
    harness.load_program(&[0xB8, 0xAA, 0xAA, 0xB9, 0x55, 0x55, 0x31, 0xC1], 0);

    harness.step(); // MOV AX, 0xAAAA
    harness.step(); // MOV CX, 0x5555
    harness.step(); // XOR CX, AX (opcode 0x31, ModR/M 0xC1)

    assert_eq!(harness.cpu.regs[1], 0xFFFF); // CX = 0xFFFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
}

// TEST instruction tests

#[test]
fn test_test_al_imm8_zero() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x0F; TEST AL, 0xF0
    harness.load_program(&[0xB0, 0x0F, 0xA8, 0xF0], 0);

    harness.step(); // MOV AL, 0x0F
    harness.step(); // TEST AL, 0xF0

    // AL should still be 0x0F (TEST doesn't modify the operand)
    assert_eq!(harness.cpu.read_reg8(0), 0x0F);
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set (0x0F & 0xF0 = 0)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_test_al_imm8_nonzero() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; TEST AL, 0x0F
    harness.load_program(&[0xB0, 0xFF, 0xA8, 0x0F], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // TEST AL, 0x0F

    // AL should still be 0xFF (TEST doesn't modify the operand)
    assert_eq!(harness.cpu.read_reg8(0), 0xFF);
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear (0xFF & 0x0F = 0x0F)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_test_al_imm8_sign_flag() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; TEST AL, 0x80
    harness.load_program(&[0xB0, 0xFF, 0xA8, 0x80], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // TEST AL, 0x80

    // AL should still be 0xFF (TEST doesn't modify the operand)
    assert_eq!(harness.cpu.read_reg8(0), 0xFF);
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear (0xFF & 0x80 = 0x80)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set (bit 7 is 1)
}

#[test]
fn test_test_ax_imm16_zero() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x00FF; TEST AX, 0xFF00
    harness.load_program(&[0xB8, 0xFF, 0x00, 0xA9, 0x00, 0xFF], 0);

    harness.step(); // MOV AX, 0x00FF
    harness.step(); // TEST AX, 0xFF00

    // AX should still be 0x00FF (TEST doesn't modify the operand)
    assert_eq!(harness.cpu.regs[0], 0x00FF);
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set (0x00FF & 0xFF00 = 0)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_test_ax_imm16_nonzero() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; TEST AX, 0x00FF
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0xA9, 0xFF, 0x00], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // TEST AX, 0x00FF

    // AX should still be 0xFFFF (TEST doesn't modify the operand)
    assert_eq!(harness.cpu.regs[0], 0xFFFF);
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear (0xFFFF & 0x00FF = 0x00FF)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_test_ax_imm16_sign_flag() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; TEST AX, 0x8000
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0xA9, 0x00, 0x80], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // TEST AX, 0x8000

    // AX should still be 0xFFFF (TEST doesn't modify the operand)
    assert_eq!(harness.cpu.regs[0], 0xFFFF);
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear (0xFFFF & 0x8000 = 0x8000)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set (bit 15 is 1)
}

#[test]
fn test_test_rm8_r8_zero() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x0F; MOV CL, 0xF0; TEST AL, CL
    harness.load_program(&[0xB0, 0x0F, 0xB1, 0xF0, 0x84, 0xC1], 0);

    harness.step(); // MOV AL, 0x0F
    harness.step(); // MOV CL, 0xF0
    harness.step(); // TEST AL, CL (opcode 0x84, ModR/M 0xC1)

    // AL and CL should be unchanged (TEST doesn't modify operands)
    assert_eq!(harness.cpu.read_reg8(0), 0x0F); // AL = 0x0F
    assert_eq!(harness.cpu.read_reg8(1), 0xF0); // CL = 0xF0
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set (0x0F & 0xF0 = 0)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_test_rm8_r8_nonzero() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; MOV CL, 0x0F; TEST AL, CL
    harness.load_program(&[0xB0, 0xFF, 0xB1, 0x0F, 0x84, 0xC1], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // MOV CL, 0x0F
    harness.step(); // TEST AL, CL (opcode 0x84, ModR/M 0xC1)

    // AL and CL should be unchanged (TEST doesn't modify operands)
    assert_eq!(harness.cpu.read_reg8(0), 0xFF); // AL = 0xFF
    assert_eq!(harness.cpu.read_reg8(1), 0x0F); // CL = 0x0F
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear (0xFF & 0x0F = 0x0F)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_test_rm8_r8_sign_flag() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; MOV CL, 0x80; TEST AL, CL
    harness.load_program(&[0xB0, 0xFF, 0xB1, 0x80, 0x84, 0xC1], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // MOV CL, 0x80
    harness.step(); // TEST AL, CL (opcode 0x84, ModR/M 0xC1)

    // AL and CL should be unchanged (TEST doesn't modify operands)
    assert_eq!(harness.cpu.read_reg8(0), 0xFF); // AL = 0xFF
    assert_eq!(harness.cpu.read_reg8(1), 0x80); // CL = 0x80
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear (0xFF & 0x80 = 0x80)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set (bit 7 is 1)
}

#[test]
fn test_test_rm16_r16_zero() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x00FF; MOV CX, 0xFF00; TEST AX, CX
    harness.load_program(&[0xB8, 0xFF, 0x00, 0xB9, 0x00, 0xFF, 0x85, 0xC1], 0);

    harness.step(); // MOV AX, 0x00FF
    harness.step(); // MOV CX, 0xFF00
    harness.step(); // TEST AX, CX (opcode 0x85, ModR/M 0xC1)

    // AX and CX should be unchanged (TEST doesn't modify operands)
    assert_eq!(harness.cpu.regs[0], 0x00FF); // AX = 0x00FF
    assert_eq!(harness.cpu.regs[1], 0xFF00); // CX = 0xFF00
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set (0x00FF & 0xFF00 = 0)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_test_rm16_r16_nonzero() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; MOV CX, 0x00FF; TEST AX, CX
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0xB9, 0xFF, 0x00, 0x85, 0xC1], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // MOV CX, 0x00FF
    harness.step(); // TEST AX, CX (opcode 0x85, ModR/M 0xC1)

    // AX and CX should be unchanged (TEST doesn't modify operands)
    assert_eq!(harness.cpu.regs[0], 0xFFFF); // AX = 0xFFFF
    assert_eq!(harness.cpu.regs[1], 0x00FF); // CX = 0x00FF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear (0xFFFF & 0x00FF = 0x00FF)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_test_rm16_r16_sign_flag() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; MOV CX, 0x8000; TEST AX, CX
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0xB9, 0x00, 0x80, 0x85, 0xC1], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // MOV CX, 0x8000
    harness.step(); // TEST AX, CX (opcode 0x85, ModR/M 0xC1)

    // AX and CX should be unchanged (TEST doesn't modify operands)
    assert_eq!(harness.cpu.regs[0], 0xFFFF); // AX = 0xFFFF
    assert_eq!(harness.cpu.regs[1], 0x8000); // CX = 0x8000
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear (0xFFFF & 0x8000 = 0x8000)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set (bit 15 is 1)
}

#[test]
fn test_test_parity_flag() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; TEST AL, 0x03
    harness.load_program(&[0xB0, 0xFF, 0xA8, 0x03], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // TEST AL, 0x03

    // AL should still be 0xFF (TEST doesn't modify the operand)
    assert_eq!(harness.cpu.read_reg8(0), 0xFF);
    // Result is 0x03 (two bits set, even parity)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::PF)); // PF should be set (even number of bits)
}

#[test]
fn test_test_rm8_imm8_zero() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x55; TEST AL, 0xAA (Group 0xF6, reg=0)
    harness.load_program(&[0xB0, 0x55, 0xF6, 0xC0, 0xAA], 0);

    harness.step(); // MOV AL, 0x55
    harness.step(); // TEST AL, 0xAA (opcode 0xF6, ModR/M 0xC0, imm8 0xAA)

    // AL should be unchanged (TEST doesn't modify operands)
    assert_eq!(harness.cpu.read_reg8(0), 0x55); // AL = 0x55
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set (0x55 & 0xAA = 0)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_test_rm8_imm8_nonzero() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; TEST AL, 0x0F (Group 0xF6, reg=0)
    harness.load_program(&[0xB0, 0xFF, 0xF6, 0xC0, 0x0F], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // TEST AL, 0x0F (opcode 0xF6, ModR/M 0xC0, imm8 0x0F)

    // AL should be unchanged (TEST doesn't modify operands)
    assert_eq!(harness.cpu.read_reg8(0), 0xFF); // AL = 0xFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear (0xFF & 0x0F = 0x0F)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_test_rm8_imm8_sign_flag() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0xFF; TEST AL, 0x80 (Group 0xF6, reg=0)
    harness.load_program(&[0xB0, 0xFF, 0xF6, 0xC0, 0x80], 0);

    harness.step(); // MOV AL, 0xFF
    harness.step(); // TEST AL, 0x80 (opcode 0xF6, ModR/M 0xC0, imm8 0x80)

    // AL should be unchanged (TEST doesn't modify operands)
    assert_eq!(harness.cpu.read_reg8(0), 0xFF); // AL = 0xFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear (0xFF & 0x80 = 0x80)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set (bit 7 is 1)
}

#[test]
fn test_test_rm16_imm16_zero() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x00FF; TEST AX, 0xFF00 (Group 0xF7, reg=0)
    harness.load_program(&[0xB8, 0xFF, 0x00, 0xF7, 0xC0, 0x00, 0xFF], 0);

    harness.step(); // MOV AX, 0x00FF
    harness.step(); // TEST AX, 0xFF00 (opcode 0xF7, ModR/M 0xC0, imm16 0xFF00)

    // AX should be unchanged (TEST doesn't modify operands)
    assert_eq!(harness.cpu.regs[0], 0x00FF); // AX = 0x00FF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set (0x00FF & 0xFF00 = 0)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_test_rm16_imm16_nonzero() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; TEST AX, 0x00FF (Group 0xF7, reg=0)
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0xF7, 0xC0, 0xFF, 0x00], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // TEST AX, 0x00FF (opcode 0xF7, ModR/M 0xC0, imm16 0x00FF)

    // AX should be unchanged (TEST doesn't modify operands)
    assert_eq!(harness.cpu.regs[0], 0xFFFF); // AX = 0xFFFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear (0xFFFF & 0x00FF = 0x00FF)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be clear
}

#[test]
fn test_test_rm16_imm16_sign_flag() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0xFFFF; TEST AX, 0x8000 (Group 0xF7, reg=0)
    harness.load_program(&[0xB8, 0xFF, 0xFF, 0xF7, 0xC0, 0x00, 0x80], 0);

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // TEST AX, 0x8000 (opcode 0xF7, ModR/M 0xC0, imm16 0x8000)

    // AX should be unchanged (TEST doesn't modify operands)
    assert_eq!(harness.cpu.regs[0], 0xFFFF); // AX = 0xFFFF
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear (0xFFFF & 0x8000 = 0x8000)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF)); // SF should be set (bit 15 is 1)
}

#[test]
fn test_test_mem8_imm8() {
    let mut harness = CpuHarness::new();
    // Write test value to memory
    harness.mem.write_u8(0x1000, 0xAA);
    // MOV BX, 0x1000; TEST byte [BX], 0x55 (Group 0xF6, reg=0)
    harness.load_program(&[0xBB, 0x00, 0x10, 0xF6, 0x07, 0x55], 0);

    harness.step(); // MOV BX, 0x1000
    harness.step(); // TEST byte [BX], 0x55 (opcode 0xF6, ModR/M 0x07, imm8 0x55)

    // Memory should be unchanged (TEST doesn't modify operands)
    assert_eq!(harness.mem.read_u8(0x1000), 0xAA);
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set (0xAA & 0x55 = 0)
}

#[test]
fn test_test_mem16_imm16() {
    let mut harness = CpuHarness::new();
    // Write test value to memory
    harness.mem.write_u16(0x2000, 0xFF00);
    // MOV BX, 0x2000; TEST word [BX], 0x00FF (Group 0xF7, reg=0)
    harness.load_program(&[0xBB, 0x00, 0x20, 0xF7, 0x07, 0xFF, 0x00], 0);

    harness.step(); // MOV BX, 0x2000
    harness.step(); // TEST word [BX], 0x00FF (opcode 0xF7, ModR/M 0x07, imm16 0x00FF)

    // Memory should be unchanged (TEST doesn't modify operands)
    assert_eq!(harness.mem.read_u16(0x2000), 0xFF00);
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF)); // CF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::OF)); // OF should be clear
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set (0xFF00 & 0x00FF = 0)
}
