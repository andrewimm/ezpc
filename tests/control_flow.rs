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

#[test]
fn test_loop_taken() {
    let mut harness = CpuHarness::new();
    // MOV CX, 3; <loop_start>: NOP; LOOP loop_start
    harness.load_program(
        &[
            0xB9, 0x03, 0x00, // MOV CX, 3 (offset 0-2)
            0x90, // NOP (offset 3)
            0xE2, 0xFD, // LOOP -3 (offset 4-5, jumps back to offset 3)
        ],
        0,
    );

    harness.step(); // MOV CX, 3
    assert_eq!(harness.cpu.regs[1], 3); // CX = 3

    // First iteration
    harness.step(); // NOP
    assert_eq!(harness.cpu.ip, 4); // At LOOP instruction

    harness.step(); // LOOP (CX becomes 2, jump taken)
    assert_eq!(harness.cpu.regs[1], 2); // CX decremented to 2
    assert_eq!(harness.cpu.ip, 3); // Jumped back to NOP

    // Second iteration
    harness.step(); // NOP
    harness.step(); // LOOP (CX becomes 1, jump taken)
    assert_eq!(harness.cpu.regs[1], 1); // CX decremented to 1
    assert_eq!(harness.cpu.ip, 3); // Jumped back to NOP

    // Third iteration
    harness.step(); // NOP
    harness.step(); // LOOP (CX becomes 0, jump NOT taken)
    assert_eq!(harness.cpu.regs[1], 0); // CX decremented to 0
    assert_eq!(harness.cpu.ip, 6); // Fell through, not jumped
}

#[test]
fn test_loop_not_taken() {
    let mut harness = CpuHarness::new();
    // MOV CX, 1; NOP; LOOP -3; MOV AX, 0x1234
    harness.load_program(
        &[
            0xB9, 0x01, 0x00, // MOV CX, 1 (offset 0-2)
            0x90, // NOP (offset 3)
            0xE2, 0xFD, // LOOP -3 (offset 4-5)
            0xB8, 0x34, 0x12, // MOV AX, 0x1234 (offset 6-8)
        ],
        0,
    );

    harness.step(); // MOV CX, 1
    assert_eq!(harness.cpu.regs[1], 1); // CX = 1

    harness.step(); // NOP
    harness.step(); // LOOP (CX becomes 0, jump NOT taken)
    assert_eq!(harness.cpu.regs[1], 0); // CX decremented to 0
    assert_eq!(harness.cpu.ip, 6); // Not jumped

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_loope_taken() {
    let mut harness = CpuHarness::new();
    // MOV CX, 2; MOV AL, 0; ADD AL, 0 (set ZF); LOOPE -3
    harness.load_program(
        &[
            0xB9, 0x02, 0x00, // MOV CX, 2 (offset 0-2)
            0xB0, 0x00, // MOV AL, 0 (offset 3-4)
            0x04, 0x00, // ADD AL, 0 (sets ZF) (offset 5-6)
            0xE1, 0xFC, // LOOPE -4 (offset 7-8, jumps back to offset 5)
        ],
        0,
    );

    harness.step(); // MOV CX, 2
    assert_eq!(harness.cpu.regs[1], 2); // CX = 2

    harness.step(); // MOV AL, 0

    // First iteration
    harness.step(); // ADD AL, 0 (sets ZF)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF));

    harness.step(); // LOOPE (CX becomes 1, ZF=1, jump taken)
    assert_eq!(harness.cpu.regs[1], 1); // CX decremented to 1
    assert_eq!(harness.cpu.ip, 5); // Jumped back

    // Second iteration
    harness.step(); // ADD AL, 0 (ZF still 1)
    harness.step(); // LOOPE (CX becomes 0, jump NOT taken because CX=0)
    assert_eq!(harness.cpu.regs[1], 0);
    assert_eq!(harness.cpu.ip, 9); // Not jumped
}

#[test]
fn test_loope_not_taken_zf_clear() {
    let mut harness = CpuHarness::new();
    // MOV CX, 2; MOV AL, 1; ADD AL, 0 (clears ZF); LOOPE -3
    harness.load_program(
        &[
            0xB9, 0x02, 0x00, // MOV CX, 2
            0xB0, 0x01, // MOV AL, 1
            0x04, 0x00, // ADD AL, 0 (clears ZF, result is 1)
            0xE1, 0xFC, // LOOPE -4
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // MOV CX, 2
    harness.step(); // MOV AL, 1
    harness.step(); // ADD AL, 0
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be clear

    harness.step(); // LOOPE (CX becomes 1, but ZF=0, jump NOT taken)
    assert_eq!(harness.cpu.regs[1], 1); // CX decremented to 1
    assert_eq!(harness.cpu.ip, 9); // Not jumped

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_loopne_taken() {
    let mut harness = CpuHarness::new();
    // MOV CX, 2; MOV AL, 1; ADD AL, 0 (clears ZF); LOOPNE -3
    harness.load_program(
        &[
            0xB9, 0x02, 0x00, // MOV CX, 2 (offset 0-2)
            0xB0, 0x01, // MOV AL, 1 (offset 3-4)
            0x04, 0x00, // ADD AL, 0 (clears ZF) (offset 5-6)
            0xE0, 0xFC, // LOOPNE -4 (offset 7-8)
        ],
        0,
    );

    harness.step(); // MOV CX, 2
    assert_eq!(harness.cpu.regs[1], 2); // CX = 2

    harness.step(); // MOV AL, 1

    // First iteration
    harness.step(); // ADD AL, 0 (clears ZF)
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF));

    harness.step(); // LOOPNE (CX becomes 1, ZF=0, jump taken)
    assert_eq!(harness.cpu.regs[1], 1); // CX decremented to 1
    assert_eq!(harness.cpu.ip, 5); // Jumped back

    // Second iteration
    harness.step(); // ADD AL, 0 (ZF still 0)
    harness.step(); // LOOPNE (CX becomes 0, jump NOT taken because CX=0)
    assert_eq!(harness.cpu.regs[1], 0);
    assert_eq!(harness.cpu.ip, 9); // Not jumped
}

#[test]
fn test_loopne_not_taken_zf_set() {
    let mut harness = CpuHarness::new();
    // MOV CX, 2; MOV AL, 0; ADD AL, 0 (sets ZF); LOOPNE -3
    harness.load_program(
        &[
            0xB9, 0x02, 0x00, // MOV CX, 2
            0xB0, 0x00, // MOV AL, 0
            0x04, 0x00, // ADD AL, 0 (sets ZF)
            0xE0, 0xFC, // LOOPNE -4
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0,
    );

    harness.step(); // MOV CX, 2
    harness.step(); // MOV AL, 0
    harness.step(); // ADD AL, 0
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // ZF should be set

    harness.step(); // LOOPNE (CX becomes 1, but ZF=1, jump NOT taken)
    assert_eq!(harness.cpu.regs[1], 1); // CX decremented to 1
    assert_eq!(harness.cpu.ip, 9); // Not jumped

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jcxz_taken() {
    let mut harness = CpuHarness::new();
    // MOV CX, 0; JCXZ +2; NOP; NOP; MOV AX, 0x1234
    harness.load_program(
        &[
            0xB9, 0x00, 0x00, // MOV CX, 0 (offset 0-2)
            0xE3, 0x02, // JCXZ +2 (offset 3-4)
            0x90, 0x90, // 2 NOPs to skip (offset 5-6)
            0xB8, 0x34, 0x12, // MOV AX, 0x1234 (offset 7-9)
        ],
        0,
    );

    harness.step(); // MOV CX, 0
    assert_eq!(harness.cpu.regs[1], 0); // CX = 0

    harness.step(); // JCXZ +2 (should jump)
    assert_eq!(harness.cpu.regs[1], 0); // CX unchanged
    assert_eq!(harness.cpu.ip, 7); // Jumped over NOPs

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_jcxz_not_taken() {
    let mut harness = CpuHarness::new();
    // MOV CX, 1; JCXZ +2; MOV AX, 0x1234
    harness.load_program(
        &[
            0xB9, 0x01, 0x00, // MOV CX, 1 (offset 0-2)
            0xE3, 0x02, // JCXZ +2 (offset 3-4)
            0xB8, 0x34, 0x12, // MOV AX, 0x1234 (offset 5-7)
        ],
        0,
    );

    harness.step(); // MOV CX, 1
    assert_eq!(harness.cpu.regs[1], 1); // CX = 1

    harness.step(); // JCXZ +2 (should NOT jump)
    assert_eq!(harness.cpu.regs[1], 1); // CX unchanged
    assert_eq!(harness.cpu.ip, 5); // Not jumped

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234);
}

#[test]
fn test_int_n() {
    let mut harness = CpuHarness::new();

    // Set up interrupt vector table (IVT) entry for interrupt 0x10
    // IVT entry 0x10 is at address 0x10 * 4 = 0x40
    // Format: [offset_low, offset_high, segment_low, segment_high]
    // Let's set it to 0x0500:0x0100
    harness.mem.write_u16(0x40, 0x0100); // Offset = 0x0100
    harness.mem.write_u16(0x42, 0x0500); // Segment = 0x0500

    // Load program at CS=0x0100
    harness.load_program(&[0xCD, 0x10], 0x0100);

    // Set up remaining state
    harness.cpu.regs[4] = 0x2000; // SP = 0x2000 (avoid overlap with program)
    harness.cpu.write_seg(2, 0x0200); // SS = 0x0200

    // Set some flags to verify they are pushed
    harness.cpu.set_flag(ezpc::cpu::Cpu::ZF, true);
    harness.cpu.set_flag(ezpc::cpu::Cpu::CF, true);
    harness.cpu.set_flag(ezpc::cpu::Cpu::TF, true); // This should be cleared
    harness.cpu.set_flag(ezpc::cpu::Cpu::IF, true); // This should be cleared
    let flags_before = harness.cpu.get_flags();

    harness.step(); // INT 0x10

    // Verify CS:IP was loaded from IVT
    assert_eq!(harness.cpu.read_seg(1), 0x0500); // CS
    assert_eq!(harness.cpu.ip, 0x0100); // IP

    // Verify stack has FLAGS, CS, IP (in that order, pushed last to first)
    // Stack grows downward: SP was 0x2000, now should be 0x1FFA (0x2000 - 6)
    assert_eq!(harness.cpu.regs[4], 0x1FFA); // SP

    // Verify values on stack (remember: pushed FLAGS, CS, IP)
    // Address SS:0x1FFA has IP (pushed last)
    // Address SS:0x1FFC has CS (pushed second)
    // Address SS:0x1FFE has FLAGS (pushed first)
    let stacked_ip = harness.cpu.read_mem16(&harness.mem, 0x0200, 0x1FFA);
    let stacked_cs = harness.cpu.read_mem16(&harness.mem, 0x0200, 0x1FFC);
    let stacked_flags = harness.cpu.read_mem16(&harness.mem, 0x0200, 0x1FFE);

    assert_eq!(stacked_ip, 2); // Return address (IP after INT instruction)
    assert_eq!(stacked_cs, 0x0100); // Old CS
    assert_eq!(stacked_flags, flags_before); // Flags before INT

    // Verify TF and IF were cleared
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::TF));
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::IF));

    // Verify other flags are preserved
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF));
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::CF));
}

#[test]
fn test_int3() {
    let mut harness = CpuHarness::new();

    // Set up interrupt vector table (IVT) entry for interrupt 3 (breakpoint)
    // IVT entry 3 is at address 3 * 4 = 0x0C
    // Format: [offset_low, offset_high, segment_low, segment_high]
    // Let's set it to 0x0600:0x0200
    harness.mem.write_u16(0x0C, 0x0200); // Offset = 0x0200
    harness.mem.write_u16(0x0E, 0x0600); // Segment = 0x0600

    // Load program at CS=0x0100: INT3 (0xCC - only 1 byte!)
    harness.load_program(&[0xCC], 0x0100);

    // Set up remaining state
    harness.cpu.regs[4] = 0x2000; // SP = 0x2000 (avoid overlap with program)
    harness.cpu.write_seg(2, 0x0200); // SS = 0x0200

    // Set some flags
    harness.cpu.set_flag(ezpc::cpu::Cpu::SF, true);
    harness.cpu.set_flag(ezpc::cpu::Cpu::TF, true); // This should be cleared
    harness.cpu.set_flag(ezpc::cpu::Cpu::IF, true); // This should be cleared
    let flags_before = harness.cpu.get_flags();

    harness.step(); // INT3

    // Verify CS:IP was loaded from IVT entry 3
    assert_eq!(harness.cpu.read_seg(1), 0x0600); // CS
    assert_eq!(harness.cpu.ip, 0x0200); // IP

    // Verify stack has FLAGS, CS, IP
    assert_eq!(harness.cpu.regs[4], 0x1FFA); // SP

    // Verify values on stack
    let stacked_ip = harness.cpu.read_mem16(&harness.mem, 0x0200, 0x1FFA);
    let stacked_cs = harness.cpu.read_mem16(&harness.mem, 0x0200, 0x1FFC);
    let stacked_flags = harness.cpu.read_mem16(&harness.mem, 0x0200, 0x1FFE);

    assert_eq!(stacked_ip, 1); // Return address (IP after INT3 - only 1 byte!)
    assert_eq!(stacked_cs, 0x0100); // Old CS
    assert_eq!(stacked_flags, flags_before); // Flags before INT3

    // Verify TF and IF were cleared
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::TF));
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::IF));

    // Verify other flags are preserved
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::SF));
}

#[test]
fn test_iret() {
    let mut harness = CpuHarness::new();

    // Set up stack to simulate an interrupt return
    harness.cpu.write_seg(2, 0x0200); // SS = 0x0200
    harness.cpu.regs[4] = 0x1FFA; // SP = 0x1FFA (simulating 3 pushes)

    // Push FLAGS, CS, IP onto stack (in that order)
    // Simulate the state after an interrupt:
    // - Return IP = 0x0300
    // - Return CS = 0x0300
    // - FLAGS = 0x0246 (ZF=1, PF=1, IF=1)
    harness
        .cpu
        .write_mem16(&mut harness.mem, 0x0200, 0x1FFA, 0x0300); // IP
    harness
        .cpu
        .write_mem16(&mut harness.mem, 0x0200, 0x1FFC, 0x0300); // CS
    harness
        .cpu
        .write_mem16(&mut harness.mem, 0x0200, 0x1FFE, 0x0246); // FLAGS

    // Set up current state (as if we're in an interrupt handler)
    harness.cpu.write_seg(1, 0x0500); // CS = 0x0500 (interrupt handler segment)

    // Load program: IRET (0xCF)
    harness.load_program(&[0xCF], 0);

    harness.step(); // IRET

    // Verify CS:IP was restored
    assert_eq!(harness.cpu.ip, 0x0300); // Restored IP
    assert_eq!(harness.cpu.read_seg(1), 0x0300); // Restored CS

    // Verify FLAGS were restored
    let flags = harness.cpu.get_flags();
    assert_eq!(flags, 0x0246); // Exact flags restored

    // Verify individual flags from restored state
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF));
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::PF));
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::IF));

    // Verify SP was adjusted (3 words popped)
    assert_eq!(harness.cpu.regs[4], 0x2000); // SP = 0x1FFA + 6
}

#[test]
fn test_int_then_iret() {
    let mut harness = CpuHarness::new();

    // Set up IVT entry for interrupt 0x20
    // IVT entry 0x20 is at address 0x20 * 4 = 0x80
    // Point it to address where we'll put an IRET instruction
    harness.mem.write_u16(0x80, 0x0100); // Offset = 0x0100
    harness.mem.write_u16(0x82, 0x0300); // Segment = 0x0300

    // Put IRET instruction at the interrupt handler location (0x0300:0x0100)
    let handler_addr = (0x0300_u32 << 4) + 0x0100;
    harness.mem.write_u8(handler_addr, 0xCF); // IRET

    // Load program at CS=0x0100: INT 0x20, then MOV AX, 0x1234
    harness.load_program(
        &[
            0xCD, 0x20, // INT 0x20
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
        ],
        0x0100,
    );

    // Set up remaining state
    harness.cpu.write_seg(2, 0x0200); // SS = 0x0200
    harness.cpu.regs[4] = 0x2000; // SP = 0x2000 (avoid overlap with program)

    // Set specific flags
    harness.cpu.set_flag(ezpc::cpu::Cpu::ZF, true);
    harness.cpu.set_flag(ezpc::cpu::Cpu::CF, false);
    harness.cpu.set_flag(ezpc::cpu::Cpu::IF, true);
    let initial_flags = harness.cpu.get_flags();

    let initial_ip = harness.cpu.ip;

    // Execute INT 0x20
    harness.step();

    // We should now be at the interrupt handler
    assert_eq!(harness.cpu.read_seg(1), 0x0300); // CS
    assert_eq!(harness.cpu.ip, 0x0100); // IP
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::IF)); // IF cleared by INT

    // Execute IRET
    harness.step();

    // We should be back where we were, right after the INT instruction
    assert_eq!(harness.cpu.read_seg(1), 0x0100); // CS restored
    assert_eq!(harness.cpu.ip, initial_ip + 2); // IP = after INT instruction

    // Flags should be restored (including IF)
    let restored_flags = harness.cpu.get_flags();
    assert_eq!(restored_flags, initial_flags); // Flags fully restored
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF));
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::CF));
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::IF)); // IF restored

    // Stack should be back to original position
    assert_eq!(harness.cpu.regs[4], 0x2000); // SP

    // Continue execution - should execute MOV AX, 0x1234
    harness.step();
    assert_eq!(harness.cpu.regs[0], 0x1234); // AX
}

#[test]
fn test_hardware_interrupt_from_pic() {
    let mut harness = CpuHarness::new();

    // Set up interrupt vector for IRQ0 (INT 0x08) at address 0x08 * 4 = 0x20
    // IVT entry: offset=0x1000, segment=0x0100
    harness.mem.write_u16(0x20, 0x1000); // Offset
    harness.mem.write_u16(0x22, 0x0100); // Segment

    // Set up interrupt handler at 0x0100:0x1000
    // Handler: MOV AX, 0xABCD; IRET
    harness.mem.write_u8(0x01000 + 0x1000, 0xB8); // MOV AX, imm16
    harness.mem.write_u8(0x01000 + 0x1001, 0xCD);
    harness.mem.write_u8(0x01000 + 0x1002, 0xAB);
    harness.mem.write_u8(0x01000 + 0x1003, 0xCF); // IRET

    // Main program: STI; NOP; MOV BX, 0x1234
    harness.load_program(
        &[
            0xFB, // STI - enable interrupts
            0x90, // NOP - interrupt will be checked after this
            0xBB, 0x34, 0x12, // MOV BX, 0x1234
        ],
        0,
    );

    // Initial state
    harness.cpu.regs[4] = 0x2000; // SP
    harness.cpu.regs[0] = 0x0000; // AX (should be changed by handler)
    harness.cpu.regs[3] = 0x0000; // BX (for verification)

    // Unmask IRQ0 on the PIC
    harness.mem.pic_mut().set_imr(0x00); // Enable all interrupts

    // Trigger IRQ0 (rising edge)
    harness.mem.pic_mut().set_irq_level(0, false);
    harness.mem.pic_mut().set_irq_level(0, true);

    // Verify PIC has pending interrupt
    assert!(harness.mem.pic().intr_out());

    // Execute STI - enables interrupts
    harness.step();

    // Verify IF flag is set
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::IF));

    // Execute NOP - after this, hardware interrupt check occurs
    harness.step();

    // After NOP completes, check_interrupts() should have:
    // 1. Detected the pending interrupt
    // 2. Pushed FLAGS (with IF=1), CS, IP
    // 3. Jumped to interrupt handler at 0x0100:0x1000
    // Now we should be in the handler, with IF cleared

    // Verify we jumped to the handler
    assert_eq!(harness.cpu.read_seg(1), 0x0100); // CS = 0x0100
    assert_eq!(harness.cpu.ip, 0x1000); // IP = 0x1000

    // Verify IF was cleared by interrupt entry
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::IF));

    // Execute handler: MOV AX, 0xABCD
    harness.step();
    assert_eq!(harness.cpu.regs[0], 0xABCD); // AX set by handler

    // Execute IRET - should return to main program
    harness.step();

    // After IRET, we're back in main program
    assert_eq!(harness.cpu.ip, 2); // After STI and NOP
    assert_eq!(harness.cpu.read_seg(1), 0x0000); // CS restored

    // Verify IF flag is restored (was set by STI before interrupt)
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::IF));

    // PIC should no longer have pending interrupt
    assert!(!harness.mem.pic().intr_out());

    // Continue execution - should execute MOV BX, 0x1234
    harness.step();
    assert_eq!(harness.cpu.regs[3], 0x1234); // BX
}

#[test]
fn test_hardware_interrupt_when_disabled() {
    let mut harness = CpuHarness::new();

    // Set up interrupt vector for IRQ0 (INT 0x08)
    harness.mem.write_u16(0x20, 0x1000);
    harness.mem.write_u16(0x22, 0x0100);

    // Set up interrupt handler
    harness.mem.write_u8(0x01000 + 0x1000, 0xB8); // MOV AX, imm16
    harness.mem.write_u8(0x01000 + 0x1001, 0xCD);
    harness.mem.write_u8(0x01000 + 0x1002, 0xAB);
    harness.mem.write_u8(0x01000 + 0x1003, 0xCF); // IRET

    // Main program: CLI; NOP; NOP
    harness.load_program(
        &[
            0xFA, // CLI - disable interrupts
            0x90, // NOP
            0x90, // NOP
        ],
        0,
    );

    harness.cpu.regs[0] = 0x0000; // AX

    // Unmask IRQ0 and trigger it
    harness.mem.pic_mut().set_imr(0x00);
    harness.mem.pic_mut().set_irq_level(0, false);
    harness.mem.pic_mut().set_irq_level(0, true);

    assert!(harness.mem.pic().intr_out());

    // Execute CLI - disables interrupts
    harness.step();
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::IF));

    // Execute NOP - interrupt should NOT be handled (IF=0)
    harness.step();

    // AX should still be 0 (handler didn't run)
    assert_eq!(harness.cpu.regs[0], 0x0000);

    // IP should be 2 (after CLI and NOP)
    assert_eq!(harness.cpu.ip, 2);

    // PIC should still have pending interrupt
    assert!(harness.mem.pic().intr_out());
}

#[test]
fn test_sti_no_delay_when_already_enabled() {
    let mut harness = CpuHarness::new();

    // Set up interrupt vector for IRQ0 (INT 0x08)
    harness.mem.write_u16(0x20, 0x1000);
    harness.mem.write_u16(0x22, 0x0100);

    // Set up interrupt handler
    harness.mem.write_u8(0x01000 + 0x1000, 0xB8); // MOV AX, imm16
    harness.mem.write_u8(0x01000 + 0x1001, 0xCD);
    harness.mem.write_u8(0x01000 + 0x1002, 0xAB);
    harness.mem.write_u8(0x01000 + 0x1003, 0xCF); // IRET

    // Main program: STI; STI; NOP
    // First STI enables interrupts (delay set)
    // Second STI is redundant (no delay since IF already 1)
    // After second STI, interrupt should be taken immediately
    harness.load_program(
        &[
            0xFB, // STI - enable interrupts (delay set)
            0xFB, // STI - redundant (no delay)
            0x90, // NOP - should not reach here
        ],
        0,
    );

    harness.cpu.regs[4] = 0x2000; // SP
    harness.cpu.regs[0] = 0x0000; // AX

    // Unmask and trigger IRQ0
    harness.mem.pic_mut().set_imr(0x00);
    harness.mem.pic_mut().set_irq_level(0, false);
    harness.mem.pic_mut().set_irq_level(0, true);

    assert!(harness.mem.pic().intr_out());

    // Execute first STI - enables interrupts, sets delay
    harness.step();
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::IF));

    // Execute second STI - IF already set, no delay should be added
    // After this instruction, interrupt should be recognized
    harness.step();

    // We should be in the interrupt handler now
    assert_eq!(harness.cpu.read_seg(1), 0x0100); // CS
    assert_eq!(harness.cpu.ip, 0x1000); // IP

    // Execute handler
    harness.step(); // MOV AX, 0xABCD
    assert_eq!(harness.cpu.regs[0], 0xABCD);

    harness.step(); // IRET

    // Should return to IP=2 (after both STIs, before NOP)
    assert_eq!(harness.cpu.ip, 2);
    assert_eq!(harness.cpu.read_seg(1), 0x0000);
}

#[test]
fn test_jmp_far() {
    let mut harness = CpuHarness::new();
    // JMP far 0x0100:0x0200
    harness.load_program(
        &[
            0xEA, 0x00, 0x02, 0x00, 0x01, // JMP far 0x0100:0x0200
        ],
        0,
    );

    // Place a NOP at the target address 0x0100:0x0200
    // Physical address = (0x0100 << 4) + 0x0200 = 0x1000 + 0x0200 = 0x1200
    harness.mem.write_u8(0x1200, 0x90); // NOP

    harness.step(); // JMP far

    // Check CS:IP changed to 0x0100:0x0200
    assert_eq!(harness.cpu.segments[1], 0x0100); // CS
    assert_eq!(harness.cpu.ip, 0x0200);

    // Execute NOP to verify we're at the correct location
    harness.step(); // NOP
    assert_eq!(harness.cpu.ip, 0x0201);
}

#[test]
fn test_call_far_indirect_direct_address() {
    let mut harness = CpuHarness::new();
    // Test CALL FAR [disp16] - opcode 0xFF /3 with direct addressing
    // This tests the 0xFF sentinel bug fix for group instructions

    // Set up stack
    harness.cpu.regs[4] = 0x1000; // SP

    // Store far pointer (IP=0x0100, CS=0x2000) at memory address 0x5000
    harness.mem.write_u16(0x5000, 0x0100); // IP
    harness.mem.write_u16(0x5002, 0x2000); // CS

    // Place NOP at target address 0x2000:0x0100
    harness.mem.write_u8(0x20000 + 0x0100, 0x90); // NOP

    // CALL FAR [0x5000]: 0xFF 0x1E 0x00 0x50
    // ModR/M: mod=00, reg=011 (CALL FAR), r/m=110 (direct addressing)
    harness.load_program(&[0xFF, 0x1E, 0x00, 0x50], 0);

    harness.step(); // CALL FAR [0x5000]

    // Check CS:IP changed to 0x2000:0x0100
    assert_eq!(harness.cpu.segments[1], 0x2000); // CS
    assert_eq!(harness.cpu.ip, 0x0100);

    // Check stack: CS and IP should be pushed
    assert_eq!(harness.cpu.regs[4], 0x0FFC); // SP decremented by 4

    // Check return IP (pushed second, at lower address)
    let return_ip = harness.mem.read_u16(0x0FFC);
    assert_eq!(return_ip, 4); // After the 4-byte instruction

    // Check return CS (pushed first, at higher address)
    let return_cs = harness.mem.read_u16(0x0FFE);
    assert_eq!(return_cs, 0); // Original CS
}

#[test]
fn test_jmp_far_indirect_direct_address() {
    let mut harness = CpuHarness::new();
    // Test JMP FAR [disp16] - opcode 0xFF /5 with direct addressing
    // This tests the 0xFF sentinel bug fix for group instructions

    // Store far pointer (IP=0x0200, CS=0x3000) at memory address 0x6000
    harness.mem.write_u16(0x6000, 0x0200); // IP
    harness.mem.write_u16(0x6002, 0x3000); // CS

    // Fill target area with NOPs to avoid executing garbage
    for i in 0..10 {
        harness.mem.write_u8(0x30000 + 0x0200 + i, 0x90); // NOPs
    }

    // JMP FAR [0x6000]: 0xFF 0x2E 0x00 0x60
    // ModR/M: mod=00, reg=101 (5=JMP FAR), r/m=110 (6=direct addressing)
    harness.load_program(&[0xFF, 0x2E, 0x00, 0x60], 0);

    harness.step(); // JMP FAR [0x6000]

    // Check CS:IP changed to 0x3000:0x0200
    assert_eq!(harness.cpu.segments[1], 0x3000); // CS
    assert_eq!(harness.cpu.ip, 0x0200);
}
