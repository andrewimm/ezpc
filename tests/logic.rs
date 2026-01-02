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
