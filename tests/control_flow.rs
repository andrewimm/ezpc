//! Control flow instruction tests (JMP, conditional jumps, etc.)

use ezpc::cpu::CpuHarness;

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
