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

#[test]
fn test_call_near() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x1000; CALL +2; NOP; NOP; MOV AX, 0x1234
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000 (offset 0-2)
            0xE8, 0x02, 0x00, // CALL +2 (offset 3-5)
            0x90, 0x90, // 2 NOPs to skip (offset 6-7)
            0xB8, 0x34, 0x12, // MOV AX, 0x1234 (offset 8-10)
        ],
        0,
    );

    harness.step(); // MOV SP, 0x1000
    harness.step(); // CALL +2

    // After CALL: IP at 6, pushed 6, then IP += 2, so IP = 8
    assert_eq!(harness.cpu.ip, 8);
    assert_eq!(harness.cpu.regs[4], 0x0FFE); // SP -= 2

    let return_addr = harness.mem.read_u16(0x0FFE);
    assert_eq!(return_addr, 6);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_call_far() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x1000; CALL 0x2000:0x0100
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000
            0x9A, 0x00, 0x01, 0x00, 0x20, // CALL far 0x2000:0x0100
        ],
        0,
    );

    // Place a NOP at the target address 0x2000:0x0100
    harness.mem.write_u8(0x20000 + 0x0100, 0x90); // NOP

    harness.step(); // MOV SP, 0x1000
    harness.step(); // CALL far

    // Check CS:IP changed to 0x2000:0x0100
    assert_eq!(harness.cpu.segments[1], 0x2000); // CS
    assert_eq!(harness.cpu.ip, 0x0100);

    // Check SP decremented by 4 (2 words pushed)
    assert_eq!(harness.cpu.regs[4], 0x0FFC);

    // Stack layout after push CS then push IP:
    // [0x0FFC] = IP (pushed second, SP points here)
    // [0x0FFE] = CS (pushed first)

    // Check return IP was pushed at 0x0FFC
    let return_ip = harness.mem.read_u16(0x0FFC);
    assert_eq!(return_ip, 8);

    // Check return CS was pushed at 0x0FFE
    let return_cs = harness.mem.read_u16(0x0FFE);
    assert_eq!(return_cs, 0);
}

#[test]
fn test_call_rm16_register() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x1000; MOV BX, 0x0100; CALL BX
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000 (offset 0-2)
            0xBB, 0x00, 0x01, // MOV BX, 0x0100 (offset 3-5)
            0xFF,
            0xD3, // CALL BX (0xFF with ModR/M=D3: reg=010, mod=11, rm=011=BX) (offset 6-7)
            0x90, // NOP (offset 8)
        ],
        0,
    );

    // Place target code at offset 0x0100
    harness.mem.write_u8(0x0100, 0xB8); // MOV AX,
    harness.mem.write_u8(0x0101, 0x34); // 0x1234
    harness.mem.write_u8(0x0102, 0x12);

    harness.step(); // MOV SP, 0x1000
    harness.step(); // MOV BX, 0x0100
    harness.step(); // CALL BX

    // IP should be at 0x0100
    assert_eq!(harness.cpu.ip, 0x0100);

    // SP should decrement by 2
    assert_eq!(harness.cpu.regs[4], 0x0FFE);

    // Return address should be 8
    let return_addr = harness.mem.read_u16(0x0FFE);
    assert_eq!(return_addr, 8);
}

#[test]
fn test_call_backward() {
    let mut harness = CpuHarness::new();
    // Create a simple backward CALL
    // MOV SP, 0x1000; JMP +2; <skip>; <target at 7>: NOP; CALL -8
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000 (offset 0-2)
            0xEB, 0x02, // JMP +2 (offset 3-4, after reading IP=5, +2 = 7)
            0x90, 0x90, // 2 NOPs to skip (offset 5-6)
            0x90, // NOP at offset 7 (target)
            0xE8, 0xF8, 0xFF, // CALL -8 (offset 8-10, calls back to offset 3)
        ],
        0,
    );

    harness.step(); // MOV SP, 0x1000
    harness.step(); // JMP +2, IP should be at 7
    assert_eq!(harness.cpu.ip, 7);

    harness.step(); // NOP at 7
    assert_eq!(harness.cpu.ip, 8);

    harness.step(); // CALL -8
    // IP after CALL instruction read is 11, CALL -8 means IP = 11 + (-8) = 3
    assert_eq!(harness.cpu.ip, 3);

    // Return address 11 should be on stack
    let return_addr = harness.mem.read_u16(0x0FFE);
    assert_eq!(return_addr, 11);
}
