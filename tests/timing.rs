//! Cycle timing tests
//!
//! Verify that instruction timing matches expected 8088 values.

use ezpc::cpu::CpuHarness;

/// Test NOP timing (3 cycles)
#[test]
fn test_nop_cycles() {
    let mut harness = CpuHarness::new();
    // NOP = 0x90
    harness.load_program(&[0x90], 0);

    let initial_cycles = harness.cpu.total_cycles;
    let step_cycles = harness.step();

    assert_eq!(step_cycles, 3, "NOP should take 3 cycles");
    assert_eq!(
        harness.cpu.total_cycles - initial_cycles,
        3,
        "total_cycles should increase by 3"
    );
}

/// Test MOV reg,reg timing (2 cycles)
#[test]
fn test_mov_reg_reg_cycles() {
    let mut harness = CpuHarness::new();
    // MOV CX, AX = 89 C1
    harness.load_program(&[0x89, 0xC1], 0);
    harness.cpu.write_reg16(0, 0x1234); // AX

    let initial_cycles = harness.cpu.total_cycles;
    let step_cycles = harness.step();

    // MOV r16, r16 base is 2 cycles
    assert_eq!(step_cycles, 2, "MOV reg,reg should take 2 cycles");
    assert_eq!(harness.cpu.total_cycles - initial_cycles, 2);
}

/// Test MOV reg, imm timing (4 cycles)
#[test]
fn test_mov_reg_imm_cycles() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1234 = B8 34 12
    harness.load_program(&[0xB8, 0x34, 0x12], 0);

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 4, "MOV r16, imm16 should take 4 cycles");
}

/// Test conditional jump not taken (4 cycles)
#[test]
fn test_jz_not_taken_cycles() {
    let mut harness = CpuHarness::new();
    // JZ +5 = 74 05
    // Set up so ZF=0 (not taken)
    harness.load_program(&[0x74, 0x05], 0);
    harness.cpu.set_flag(ezpc::cpu::Cpu::ZF, false);

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 4, "JZ not taken should take 4 cycles");
}

/// Test conditional jump taken (16 cycles)
#[test]
fn test_jz_taken_cycles() {
    let mut harness = CpuHarness::new();
    // JZ +5 = 74 05
    // Set up so ZF=1 (taken)
    harness.load_program(&[0x74, 0x05], 0);
    harness.cpu.set_flag(ezpc::cpu::Cpu::ZF, true);

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 16, "JZ taken should take 16 cycles");
}

/// Test JMP short timing (15 cycles)
#[test]
fn test_jmp_short_cycles() {
    let mut harness = CpuHarness::new();
    // JMP short +5 = EB 05
    harness.load_program(&[0xEB, 0x05], 0);

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 15, "JMP short should take 15 cycles");
}

/// Test CALL near timing (23 cycles)
#[test]
fn test_call_near_cycles() {
    let mut harness = CpuHarness::new();
    // CALL near +0x1234 = E8 34 12
    harness.load_program(&[0xE8, 0x34, 0x12], 0);
    harness.cpu.write_reg16(4, 0xFFFE); // SP

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 23, "CALL near should take 23 cycles");
}

/// Test RET near timing (20 cycles)
#[test]
fn test_ret_near_cycles() {
    let mut harness = CpuHarness::new();
    // RET near = C3
    harness.load_program(&[0xC3], 0);
    // Set up stack with return address
    harness.cpu.write_reg16(4, 0xFFFC); // SP
    harness.cpu.write_seg(2, 0); // SS
    // Write return address to stack
    harness.mem.write_u16(0xFFFC, 0x1000);

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 20, "RET near should take 20 cycles");
}

/// Test INC r16 timing (2 cycles)
#[test]
fn test_inc_r16_cycles() {
    let mut harness = CpuHarness::new();
    // INC AX = 40
    harness.load_program(&[0x40], 0);

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 2, "INC r16 should take 2 cycles");
}

/// Test PUSH r16 timing (15 cycles)
#[test]
fn test_push_r16_cycles() {
    let mut harness = CpuHarness::new();
    // PUSH AX = 50
    harness.load_program(&[0x50], 0);
    harness.cpu.write_reg16(4, 0xFFFE); // SP

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 15, "PUSH r16 should take 15 cycles");
}

/// Test POP r16 timing (12 cycles)
#[test]
fn test_pop_r16_cycles() {
    let mut harness = CpuHarness::new();
    // POP AX = 58
    harness.load_program(&[0x58], 0);
    harness.cpu.write_reg16(4, 0xFFFC); // SP
    harness.cpu.write_seg(2, 0); // SS

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 12, "POP r16 should take 12 cycles");
}

/// Test ADD r8,r8 timing (3 cycles)
#[test]
fn test_add_r8_r8_cycles() {
    let mut harness = CpuHarness::new();
    // ADD AL, BL = 00 D8
    harness.load_program(&[0x00, 0xD8], 0);

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 3, "ADD r8,r8 should take 3 cycles");
}

/// Test ADD AL, imm8 timing (4 cycles)
#[test]
fn test_add_al_imm8_cycles() {
    let mut harness = CpuHarness::new();
    // ADD AL, 0x55 = 04 55
    harness.load_program(&[0x04, 0x55], 0);

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 4, "ADD AL, imm8 should take 4 cycles");
}

/// Test LOOP taken timing
#[test]
fn test_loop_taken_cycles() {
    let mut harness = CpuHarness::new();
    // LOOP -2 = E2 FE
    harness.load_program(&[0xE2, 0xFE], 0);
    harness.cpu.write_reg16(1, 5); // CX = 5 (will loop)

    let step_cycles = harness.step();

    // LOOP taken = 17 cycles (5 base + 12 extra)
    assert_eq!(step_cycles, 17, "LOOP taken should take 17 cycles");
}

/// Test LOOP not taken timing
#[test]
fn test_loop_not_taken_cycles() {
    let mut harness = CpuHarness::new();
    // LOOP -2 = E2 FE
    harness.load_program(&[0xE2, 0xFE], 0);
    harness.cpu.write_reg16(1, 1); // CX = 1 (will become 0, not loop)

    let step_cycles = harness.step();

    // LOOP not taken = 5 cycles
    assert_eq!(step_cycles, 5, "LOOP not taken should take 5 cycles");
}

/// Test segment override adds 2 cycles
#[test]
fn test_segment_override_adds_cycles() {
    let mut harness = CpuHarness::new();
    // ES: MOV AX, [BX] = 26 8B 07
    // Without segment override: MOV AX, [BX] = 8B 07
    harness.load_program(&[0x26, 0x8B, 0x07], 0);

    let cycles_with_override = harness.step();

    // Reset for comparison
    let mut harness2 = CpuHarness::new();
    harness2.load_program(&[0x8B, 0x07], 0);
    let cycles_without_override = harness2.step();

    // Segment override adds 2 cycles
    assert_eq!(
        cycles_with_override,
        cycles_without_override + 2,
        "Segment override should add 2 cycles"
    );
}

/// Test that total_cycles accumulates correctly over multiple instructions
#[test]
fn test_total_cycles_accumulation() {
    let mut harness = CpuHarness::new();
    // NOP; NOP; NOP = 90 90 90
    harness.load_program(&[0x90, 0x90, 0x90], 0);

    let initial_cycles = harness.cpu.total_cycles;

    harness.step(); // 3 cycles
    harness.step(); // 3 cycles
    harness.step(); // 3 cycles

    assert_eq!(
        harness.cpu.total_cycles - initial_cycles,
        9,
        "3 NOPs should accumulate to 9 cycles"
    );
}

/// Test IN AL, imm8 timing (10 cycles)
#[test]
fn test_in_al_imm8_cycles() {
    let mut harness = CpuHarness::new();
    // IN AL, 0x60 = E4 60
    harness.load_program(&[0xE4, 0x60], 0);

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 10, "IN AL, imm8 should take 10 cycles");
}

/// Test OUT imm8, AL timing (10 cycles)
#[test]
fn test_out_imm8_al_cycles() {
    let mut harness = CpuHarness::new();
    // OUT 0x60, AL = E6 60
    harness.load_program(&[0xE6, 0x60], 0);

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 10, "OUT imm8, AL should take 10 cycles");
}

/// Test IN AL, DX timing (8 cycles)
#[test]
fn test_in_al_dx_cycles() {
    let mut harness = CpuHarness::new();
    // IN AL, DX = EC
    harness.load_program(&[0xEC], 0);
    harness.cpu.write_reg16(2, 0x60); // DX = 0x60

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 8, "IN AL, DX should take 8 cycles");
}

/// Test CLI timing (2 cycles)
#[test]
fn test_cli_cycles() {
    let mut harness = CpuHarness::new();
    // CLI = FA
    harness.load_program(&[0xFA], 0);

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 2, "CLI should take 2 cycles");
}

/// Test STI timing (2 cycles)
#[test]
fn test_sti_cycles() {
    let mut harness = CpuHarness::new();
    // STI = FB
    harness.load_program(&[0xFB], 0);

    let step_cycles = harness.step();

    assert_eq!(step_cycles, 2, "STI should take 2 cycles");
}
