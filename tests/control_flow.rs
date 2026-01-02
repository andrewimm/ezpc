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

#[test]
fn test_ret_near() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x1000; CALL +3; MOV AX, 0x1234; RET
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000 (offset 0-2)
            0xE8, 0x03, 0x00, // CALL +3 (offset 3-5, after reading IP=6, jump to 6+3=9)
            0xB8, 0x34, 0x12, // MOV AX, 0x1234 (offset 6-8)
            0xC3, // RET (offset 9)
        ],
        0,
    );

    harness.step(); // MOV SP, 0x1000
    harness.step(); // CALL +3, pushes return address 6, jumps to 9

    // Verify we're at offset 9 (RET instruction)
    assert_eq!(harness.cpu.ip, 9);
    assert_eq!(harness.cpu.regs[4], 0x0FFE); // SP after push

    harness.step(); // RET at offset 9, pops return address and jumps back

    // Should be back at offset 6 (after the CALL)
    assert_eq!(harness.cpu.ip, 6);
    // SP should be restored to 0x1000
    assert_eq!(harness.cpu.regs[4], 0x1000);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_ret_near_imm() {
    let mut harness = CpuHarness::new();
    // Manually set up stack with a dummy value to clean up
    // MOV SP, 0x0FFE (simulate one value already pushed); CALL +3; MOV AX, 0x1234; RET 2
    harness.load_program(
        &[
            0xBC, 0xFE, 0x0F, // MOV SP, 0x0FFE (offset 0-2)
            0xE8, 0x03, 0x00, // CALL +3 (offset 3-5, after reading IP=6, jump to 6+3=9)
            0xB8, 0x34, 0x12, // MOV AX, 0x1234 (offset 6-8)
            0xC2, 0x02, 0x00, // RET 2 (offset 9-11)
        ],
        0,
    );

    harness.step(); // MOV SP, 0x0FFE
    assert_eq!(harness.cpu.regs[4], 0x0FFE);

    harness.step(); // CALL +3, pushes return address 6, jumps to 9
    assert_eq!(harness.cpu.ip, 9); // At procedure (RET 2)
    assert_eq!(harness.cpu.regs[4], 0x0FFC); // SP after push

    harness.step(); // RET 2, pops return address and adds 2 to SP

    // Should be back at offset 6
    assert_eq!(harness.cpu.ip, 6);
    // SP should be 0x1000 (0x0FFC + 2 for pop + 2 for cleanup)
    assert_eq!(harness.cpu.regs[4], 0x1000);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_ret_far() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x1000; CALL far 0x0000:0x000B; MOV AX, 0x1234; RETF
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000 (offset 0-2)
            0x9A, 0x0B, 0x00, 0x00, 0x00, // CALL far 0x0000:0x000B (offset 3-7)
            0xB8, 0x34, 0x12, // MOV AX, 0x1234 (offset 8-10)
            0xCB, // RETF (offset 11)
        ],
        0,
    );

    harness.step(); // MOV SP, 0x1000
    harness.step(); // CALL far, pushes CS=0, IP=8, jumps to 0x0000:0x000B

    // Verify we're at the far address (which is actually in same segment)
    assert_eq!(harness.cpu.segments[1], 0x0000); // CS (unchanged)
    assert_eq!(harness.cpu.ip, 0x000B); // IP at offset 11
    assert_eq!(harness.cpu.regs[4], 0x0FFC); // SP after pushing CS and IP

    harness.step(); // RETF at offset 11, pops IP and CS

    // Should be back at CS=0, IP=8
    assert_eq!(harness.cpu.segments[1], 0); // CS restored
    assert_eq!(harness.cpu.ip, 8); // IP restored
    assert_eq!(harness.cpu.regs[4], 0x1000); // SP restored

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_ret_far_imm() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x0FFE (simulate one value already on stack); CALL far 0x0000:0x000B; MOV AX, 0x1234; RETF 2
    harness.load_program(
        &[
            0xBC, 0xFE, 0x0F, // MOV SP, 0x0FFE (offset 0-2)
            0x9A, 0x0B, 0x00, 0x00, 0x00, // CALL far 0x0000:0x000B (offset 3-7)
            0xB8, 0x34, 0x12, // MOV AX, 0x1234 (offset 8-10)
            0xCA, 0x02, 0x00, // RETF 2 (offset 11-13)
        ],
        0,
    );

    harness.step(); // MOV SP, 0x0FFE
    assert_eq!(harness.cpu.regs[4], 0x0FFE);

    harness.step(); // CALL far, pushes CS=0, IP=8
    assert_eq!(harness.cpu.segments[1], 0x0000); // CS (unchanged)
    assert_eq!(harness.cpu.ip, 0x000B); // IP at offset 11
    assert_eq!(harness.cpu.regs[4], 0x0FFA); // SP after pushing CS and IP

    harness.step(); // RETF 2, pops IP, CS, and adds 2 to SP

    // Should be back at CS=0, IP=8
    assert_eq!(harness.cpu.segments[1], 0); // CS restored
    assert_eq!(harness.cpu.ip, 8); // IP restored
    // SP should be 0x1000 (0x0FFA + 4 for pops + 2 for cleanup)
    assert_eq!(harness.cpu.regs[4], 0x1000);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jo_taken() {
    let mut harness = CpuHarness::new();
    // Manually set OF and test JO
    harness.cpu.set_flag(ezpc::cpu::Cpu::OF, true);

    harness.load_program(
        &[
            0x70, 0x02, // JO +2
            0x90, 0x90, // NOPs
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // JO +2
    assert_eq!(harness.cpu.ip, 4);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jno_not_taken() {
    let mut harness = CpuHarness::new();
    // Manually set OF and test JNO (should not jump)
    harness.cpu.set_flag(ezpc::cpu::Cpu::OF, true);

    harness.load_program(
        &[
            0x71, 0x02, // JNO +2
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // JNO +2
    assert_eq!(harness.cpu.ip, 2);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jno_taken() {
    let mut harness = CpuHarness::new();
    // OF not set, JNO should jump
    harness.load_program(
        &[
            0x71, 0x02, // JNO +2
            0x90, 0x90, // NOPs
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // JNO +2
    assert_eq!(harness.cpu.ip, 4);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jbe_taken_carry() {
    let mut harness = CpuHarness::new();
    // MOV AL, 255; ADD AL, 1 (sets CF); JBE +2 (should be taken)
    harness.load_program(
        &[
            0xB0, 0xFF, // MOV AL, 255
            0x04, 0x01, // ADD AL, 1 (sets CF)
            0x76, 0x02, // JBE +2
            0x90, 0x90, // 2 NOPs to skip
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // MOV AL, 255
    harness.step(); // ADD AL, 1
    harness.step(); // JBE +2 (should be taken)

    // IP should be 6 + 2 = 8
    assert_eq!(harness.cpu.ip, 8);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jbe_taken_zero() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0; ADD AL, 0 (sets ZF); JBE +2 (should be taken)
    harness.load_program(
        &[
            0xB0, 0x00, // MOV AL, 0
            0x04, 0x00, // ADD AL, 0 (sets ZF)
            0x76, 0x02, // JBE +2
            0x90, 0x90, // 2 NOPs to skip
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // MOV AL, 0
    harness.step(); // ADD AL, 0
    harness.step(); // JBE +2 (should be taken)

    // IP should be 6 + 2 = 8
    assert_eq!(harness.cpu.ip, 8);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_ja_not_taken() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0; ADD AL, 0 (sets ZF); JA +2 (should NOT be taken)
    harness.load_program(
        &[
            0xB0, 0x00, // MOV AL, 0
            0x04, 0x00, // ADD AL, 0 (sets ZF)
            0x77, 0x02, // JA +2
            0xB8, 0x34, 0x12, // MOV AX, 0x1234 (should execute)
        ],
        0,
    );

    harness.step(); // MOV AL, 0
    harness.step(); // ADD AL, 0
    harness.step(); // JA +2 (not taken)

    // IP should be 6 (not jumped)
    assert_eq!(harness.cpu.ip, 6);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_ja_taken() {
    let mut harness = CpuHarness::new();
    // MOV AL, 1; ADD AL, 1 (no carry, not zero); JA +2 (should be taken)
    harness.load_program(
        &[
            0xB0, 0x01, // MOV AL, 1
            0x04, 0x01, // ADD AL, 1
            0x77, 0x02, // JA +2
            0x90, 0x90, // 2 NOPs to skip
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // MOV AL, 1
    harness.step(); // ADD AL, 1
    harness.step(); // JA +2 (should be taken)

    // IP should be 6 + 2 = 8
    assert_eq!(harness.cpu.ip, 8);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jp_taken() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0; ADD AL, 0 (even parity, PF=1); JP +2 (should be taken)
    harness.load_program(
        &[
            0xB0, 0x00, // MOV AL, 0
            0x04, 0x00, // ADD AL, 0 (sets PF for even parity)
            0x7A, 0x02, // JP +2
            0x90, 0x90, // 2 NOPs to skip
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // MOV AL, 0
    harness.step(); // ADD AL, 0
    harness.step(); // JP +2 (should be taken)

    // IP should be 6 + 2 = 8
    assert_eq!(harness.cpu.ip, 8);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jnp_taken() {
    let mut harness = CpuHarness::new();
    // MOV AL, 1; ADD AL, 0 (odd parity, PF=0); JNP +2 (should be taken)
    harness.load_program(
        &[
            0xB0, 0x01, // MOV AL, 1
            0x04, 0x00, // ADD AL, 0 (odd parity)
            0x7B, 0x02, // JNP +2
            0x90, 0x90, // 2 NOPs to skip
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // MOV AL, 1
    harness.step(); // ADD AL, 0
    harness.step(); // JNP +2 (should be taken)

    // IP should be 6 + 2 = 8
    assert_eq!(harness.cpu.ip, 8);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jl_taken() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0; ADD AL, 0x80; JL +2
    harness.load_program(
        &[
            0xB0, 0x00, // MOV AL, 0
            0x04, 0x80, // ADD AL, 0x80
            0x7C, 0x02, // JL +2
            0x90, 0x90, // NOPs
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // MOV AL, 0
    harness.step(); // ADD AL, 0x80
    harness.step(); // JL +2
    assert_eq!(harness.cpu.ip, 8);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jge_taken() {
    let mut harness = CpuHarness::new();
    // MOV AL, 1; ADD AL, 1 (no overflow, SF=0, OF=0, SF==OF); JGE +2 (should be taken)
    harness.load_program(
        &[
            0xB0, 0x01, // MOV AL, 1
            0x04, 0x01, // ADD AL, 1 (SF=0, OF=0)
            0x7D, 0x02, // JGE +2 (SF == OF, should be taken)
            0x90, 0x90, // 2 NOPs to skip
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // MOV AL, 1
    harness.step(); // ADD AL, 1
    harness.step(); // JGE +2 (should be taken)

    // IP should be 6 + 2 = 8
    assert_eq!(harness.cpu.ip, 8);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jle_taken_zero() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0; ADD AL, 0 (ZF=1); JLE +2 (should be taken)
    harness.load_program(
        &[
            0xB0, 0x00, // MOV AL, 0
            0x04, 0x00, // ADD AL, 0 (ZF=1)
            0x7E, 0x02, // JLE +2 (ZF=1 or SF!=OF, should be taken)
            0x90, 0x90, // 2 NOPs to skip
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // MOV AL, 0
    harness.step(); // ADD AL, 0
    harness.step(); // JLE +2 (should be taken)

    // IP should be 6 + 2 = 8
    assert_eq!(harness.cpu.ip, 8);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jle_taken_less() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0; ADD AL, 0x80; JLE +2
    harness.load_program(
        &[
            0xB0, 0x00, // MOV AL, 0
            0x04, 0x80, // ADD AL, 0x80
            0x7E, 0x02, // JLE +2
            0x90, 0x90, // NOPs
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // MOV AL, 0
    harness.step(); // ADD AL, 0x80
    harness.step(); // JLE +2
    assert_eq!(harness.cpu.ip, 8);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jg_taken() {
    let mut harness = CpuHarness::new();
    // MOV AL, 1; ADD AL, 1 (ZF=0, SF=0, OF=0, SF==OF); JG +2 (should be taken)
    harness.load_program(
        &[
            0xB0, 0x01, // MOV AL, 1
            0x04, 0x01, // ADD AL, 1 (ZF=0, SF=0, OF=0)
            0x7F, 0x02, // JG +2 (ZF=0 and SF==OF, should be taken)
            0x90, 0x90, // 2 NOPs to skip
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // MOV AL, 1
    harness.step(); // ADD AL, 1
    harness.step(); // JG +2 (should be taken)

    // IP should be 6 + 2 = 8
    assert_eq!(harness.cpu.ip, 8);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jg_not_taken_zero() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0; ADD AL, 0 (ZF=1); JG +2 (should NOT be taken)
    harness.load_program(
        &[
            0xB0, 0x00, // MOV AL, 0
            0x04, 0x00, // ADD AL, 0 (ZF=1)
            0x7F, 0x02, // JG +2 (not taken because ZF=1)
            0xB8, 0x34, 0x12, // MOV AX, 0x1234 (should execute)
        ],
        0,
    );

    harness.step(); // MOV AL, 0
    harness.step(); // ADD AL, 0
    harness.step(); // JG +2 (not taken)

    // IP should be 6 (not jumped)
    assert_eq!(harness.cpu.ip, 6);

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}
