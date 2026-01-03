//! String operation instruction tests

use ezpc::cpu::CpuHarness;

#[test]
fn test_stosb_single() {
    let mut harness = CpuHarness::new();
    // CLD; MOV AL, 0x42; MOV DI, 0x1000; STOSB
    harness.load_program(
        &[
            0xFC, // CLD
            0xB0, 0x42, // MOV AL, 0x42
            0xBF, 0x00, 0x10, // MOV DI, 0x1000
            0xAA, // STOSB
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV AL, 0x42
    harness.step(); // MOV DI, 0x1000
    harness.step(); // STOSB

    // Check that 0x42 was written to ES:DI (ES:0x1000)
    assert_eq!(harness.mem.read_u8(0x1000), 0x42);
    // Check that DI was incremented
    assert_eq!(harness.cpu.read_reg16(7), 0x1001);
}

#[test]
fn test_stosb_backward() {
    let mut harness = CpuHarness::new();
    // STD; MOV AL, 0x42; MOV DI, 0x1000; STOSB
    harness.load_program(
        &[
            0xFD, // STD
            0xB0, 0x42, // MOV AL, 0x42
            0xBF, 0x00, 0x10, // MOV DI, 0x1000
            0xAA, // STOSB
        ],
        0,
    );

    harness.step(); // STD
    harness.step(); // MOV AL, 0x42
    harness.step(); // MOV DI, 0x1000
    harness.step(); // STOSB

    // Check that 0x42 was written to ES:DI
    assert_eq!(harness.mem.read_u8(0x1000), 0x42);
    // Check that DI was decremented (DF=1)
    assert_eq!(harness.cpu.read_reg16(7), 0x0FFF);
}

#[test]
fn test_rep_stosb() {
    let mut harness = CpuHarness::new();
    // CLD; MOV AL, 0x55; MOV CX, 5; MOV DI, 0x2000; REP STOSB
    harness.load_program(
        &[
            0xFC, // CLD
            0xB0, 0x55, // MOV AL, 0x55
            0xB9, 0x05, 0x00, // MOV CX, 5
            0xBF, 0x00, 0x20, // MOV DI, 0x2000
            0xF3, 0xAA, // REP STOSB
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV AL, 0x55
    harness.step(); // MOV CX, 5
    harness.step(); // MOV DI, 0x2000

    // Execute REP STOSB - should repeat 5 times
    harness.step(); // First iteration
    harness.step(); // Second iteration
    harness.step(); // Third iteration
    harness.step(); // Fourth iteration
    harness.step(); // Fifth iteration

    // Check that 5 bytes were written
    for i in 0..5 {
        assert_eq!(harness.mem.read_u8(0x2000 + i), 0x55);
    }

    // Check that CX is now 0
    assert_eq!(harness.cpu.read_reg16(1), 0);
    // Check that DI advanced by 5
    assert_eq!(harness.cpu.read_reg16(7), 0x2005);
}

#[test]
fn test_es_rep_stosb() {
    // This test verifies that ES: REP STOSB works correctly
    // (multiple prefixes chaining)
    let mut harness = CpuHarness::new();

    // Setup: CLD; MOV AL, 0xAA; MOV CX, 3; MOV DI, 0x3000; ES: REP STOSB
    harness.load_program(
        &[
            0xFC, // CLD
            0xB0, 0xAA, // MOV AL, 0xAA
            0xB9, 0x03, 0x00, // MOV CX, 3
            0xBF, 0x00, 0x30, // MOV DI, 0x3000
            0x26, 0xF3, 0xAA, // ES: REP STOSB
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV AL, 0xAA
    harness.step(); // MOV CX, 3
    harness.step(); // MOV DI, 0x3000

    // Execute ES: REP STOSB - should repeat 3 times
    harness.step(); // First iteration
    harness.step(); // Second iteration
    harness.step(); // Third iteration

    // Check that 3 bytes were written to ES segment
    for i in 0..3 {
        assert_eq!(harness.mem.read_u8(0x3000 + i), 0xAA);
    }

    // Check that CX is now 0
    assert_eq!(harness.cpu.read_reg16(1), 0);
    // Check that DI advanced by 3
    assert_eq!(harness.cpu.read_reg16(7), 0x3003);
}

#[test]
fn test_stosw() {
    let mut harness = CpuHarness::new();
    // CLD; MOV AX, 0x1234; MOV DI, 0x4000; STOSW
    harness.load_program(
        &[
            0xFC, // CLD
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
            0xBF, 0x00, 0x40, // MOV DI, 0x4000
            0xAB, // STOSW
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV AX, 0x1234
    harness.step(); // MOV DI, 0x4000
    harness.step(); // STOSW

    // Check that 0x1234 was written to ES:DI
    assert_eq!(harness.mem.read_u16(0x4000), 0x1234);
    // Check that DI was incremented by 2
    assert_eq!(harness.cpu.read_reg16(7), 0x4002);
}

#[test]
fn test_rep_stosw() {
    let mut harness = CpuHarness::new();
    // CLD; MOV AX, 0xBEEF; MOV CX, 4; MOV DI, 0x5000; REP STOSW
    harness.load_program(
        &[
            0xFC, // CLD
            0xB8, 0xEF, 0xBE, // MOV AX, 0xBEEF
            0xB9, 0x04, 0x00, // MOV CX, 4
            0xBF, 0x00, 0x50, // MOV DI, 0x5000
            0xF3, 0xAB, // REP STOSW
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV AX, 0xBEEF
    harness.step(); // MOV CX, 4
    harness.step(); // MOV DI, 0x5000

    // Execute REP STOSW - should repeat 4 times
    for _ in 0..4 {
        harness.step();
    }

    // Check that 4 words were written
    for i in 0..4 {
        assert_eq!(harness.mem.read_u16(0x5000 + i * 2), 0xBEEF);
    }

    // Check that CX is now 0
    assert_eq!(harness.cpu.read_reg16(1), 0);
    // Check that DI advanced by 8 (4 words * 2 bytes)
    assert_eq!(harness.cpu.read_reg16(7), 0x5008);
}

#[test]
fn test_lodsb() {
    let mut harness = CpuHarness::new();

    // Write test data to memory
    harness.mem.write_u8(0x1000, 0x77);

    // CLD; MOV SI, 0x1000; LODSB
    harness.load_program(
        &[
            0xFC, // CLD
            0xBE, 0x00, 0x10, // MOV SI, 0x1000
            0xAC, // LODSB
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV SI, 0x1000
    harness.step(); // LODSB

    // Check that AL contains the loaded byte
    assert_eq!(harness.cpu.read_reg8(0), 0x77);
    // Check that SI was incremented
    assert_eq!(harness.cpu.read_reg16(6), 0x1001);
}

#[test]
fn test_lodsw() {
    let mut harness = CpuHarness::new();

    // Write test data to memory
    harness.mem.write_u16(0x2000, 0xABCD);

    // CLD; MOV SI, 0x2000; LODSW
    harness.load_program(
        &[
            0xFC, // CLD
            0xBE, 0x00, 0x20, // MOV SI, 0x2000
            0xAD, // LODSW
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV SI, 0x2000
    harness.step(); // LODSW

    // Check that AX contains the loaded word
    assert_eq!(harness.cpu.read_reg16(0), 0xABCD);
    // Check that SI was incremented by 2
    assert_eq!(harness.cpu.read_reg16(6), 0x2002);
}

#[test]
fn test_movsb() {
    let mut harness = CpuHarness::new();

    // Write test data to source
    harness.mem.write_u8(0x1000, 0x88);

    // CLD; MOV SI, 0x1000; MOV DI, 0x2000; MOVSB
    harness.load_program(
        &[
            0xFC, // CLD
            0xBE, 0x00, 0x10, // MOV SI, 0x1000
            0xBF, 0x00, 0x20, // MOV DI, 0x2000
            0xA4, // MOVSB
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV SI, 0x1000
    harness.step(); // MOV DI, 0x2000
    harness.step(); // MOVSB

    // Check that byte was copied
    assert_eq!(harness.mem.read_u8(0x2000), 0x88);
    // Check that SI and DI were incremented
    assert_eq!(harness.cpu.read_reg16(6), 0x1001);
    assert_eq!(harness.cpu.read_reg16(7), 0x2001);
}

#[test]
fn test_rep_movsb() {
    let mut harness = CpuHarness::new();

    // Write test data to source
    for i in 0..10 {
        harness.mem.write_u8(0x1000 + i, i as u8 + 0x30);
    }

    // CLD; MOV SI, 0x1000; MOV DI, 0x3000; MOV CX, 10; REP MOVSB
    harness.load_program(
        &[
            0xFC, // CLD
            0xBE, 0x00, 0x10, // MOV SI, 0x1000
            0xBF, 0x00, 0x30, // MOV DI, 0x3000
            0xB9, 0x0A, 0x00, // MOV CX, 10
            0xF3, 0xA4, // REP MOVSB
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV SI, 0x1000
    harness.step(); // MOV DI, 0x3000
    harness.step(); // MOV CX, 10

    // Execute REP MOVSB - should repeat 10 times
    for _ in 0..10 {
        harness.step();
    }

    // Check that all bytes were copied
    for i in 0..10 {
        assert_eq!(harness.mem.read_u8(0x3000 + i), i as u8 + 0x30);
    }

    // Check that CX is now 0
    assert_eq!(harness.cpu.read_reg16(1), 0);
    // Check that SI and DI advanced by 10
    assert_eq!(harness.cpu.read_reg16(6), 0x100A);
    assert_eq!(harness.cpu.read_reg16(7), 0x300A);
}

#[test]
fn test_movsw() {
    let mut harness = CpuHarness::new();

    // Write test data to source
    harness.mem.write_u16(0x1000, 0x5678);

    // CLD; MOV SI, 0x1000; MOV DI, 0x2000; MOVSW
    harness.load_program(
        &[
            0xFC, // CLD
            0xBE, 0x00, 0x10, // MOV SI, 0x1000
            0xBF, 0x00, 0x20, // MOV DI, 0x2000
            0xA5, // MOVSW
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV SI, 0x1000
    harness.step(); // MOV DI, 0x2000
    harness.step(); // MOVSW

    // Check that word was copied
    assert_eq!(harness.mem.read_u16(0x2000), 0x5678);
    // Check that SI and DI were incremented by 2
    assert_eq!(harness.cpu.read_reg16(6), 0x1002);
    assert_eq!(harness.cpu.read_reg16(7), 0x2002);
}

//
// CMPSB/CMPSW Tests
//

#[test]
fn test_cmpsb_equal() {
    let mut harness = CpuHarness::new();

    // Setup two equal bytes
    harness.mem.write_u8(0x1000, 0x42);
    harness.mem.write_u8(0x2000, 0x42);

    // CLD; MOV SI, 0x1000; MOV DI, 0x2000; CMPSB
    harness.load_program(
        &[
            0xFC, // CLD
            0xBE, 0x00, 0x10, // MOV SI, 0x1000
            0xBF, 0x00, 0x20, // MOV DI, 0x2000
            0xA6, // CMPSB
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV SI, 0x1000
    harness.step(); // MOV DI, 0x2000
    harness.step(); // CMPSB

    // Bytes are equal, so ZF should be set
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF));
    // SI and DI should have advanced
    assert_eq!(harness.cpu.read_reg16(6), 0x1001);
    assert_eq!(harness.cpu.read_reg16(7), 0x2001);
}

#[test]
fn test_cmpsb_not_equal() {
    let mut harness = CpuHarness::new();

    // Setup two different bytes
    harness.mem.write_u8(0x1000, 0x50);
    harness.mem.write_u8(0x2000, 0x30);

    // CLD; MOV SI, 0x1000; MOV DI, 0x2000; CMPSB
    harness.load_program(
        &[
            0xFC, // CLD
            0xBE, 0x00, 0x10, // MOV SI, 0x1000
            0xBF, 0x00, 0x20, // MOV DI, 0x2000
            0xA6, // CMPSB
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV SI, 0x1000
    harness.step(); // MOV DI, 0x2000
    harness.step(); // CMPSB

    // Bytes are not equal, so ZF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF));
    // SI and DI should have advanced
    assert_eq!(harness.cpu.read_reg16(6), 0x1001);
    assert_eq!(harness.cpu.read_reg16(7), 0x2001);
}

#[test]
fn test_cmpsw_equal() {
    let mut harness = CpuHarness::new();

    // Setup two equal words
    harness.mem.write_u16(0x1000, 0x1234);
    harness.mem.write_u16(0x2000, 0x1234);

    // CLD; MOV SI, 0x1000; MOV DI, 0x2000; CMPSW
    harness.load_program(
        &[
            0xFC, // CLD
            0xBE, 0x00, 0x10, // MOV SI, 0x1000
            0xBF, 0x00, 0x20, // MOV DI, 0x2000
            0xA7, // CMPSW
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV SI, 0x1000
    harness.step(); // MOV DI, 0x2000
    harness.step(); // CMPSW

    // Words are equal, so ZF should be set
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF));
    // SI and DI should have advanced by 2
    assert_eq!(harness.cpu.read_reg16(6), 0x1002);
    assert_eq!(harness.cpu.read_reg16(7), 0x2002);
}

#[test]
fn test_repe_cmpsb_all_equal() {
    let mut harness = CpuHarness::new();

    // Setup matching strings
    for i in 0..5 {
        harness.mem.write_u8(0x1000 + i, 0x41 + i as u8); // "ABCDE"
        harness.mem.write_u8(0x2000 + i, 0x41 + i as u8); // "ABCDE"
    }

    // CLD; MOV SI, 0x1000; MOV DI, 0x2000; MOV CX, 5; REPE CMPSB
    harness.load_program(
        &[
            0xFC, // CLD
            0xBE, 0x00, 0x10, // MOV SI, 0x1000
            0xBF, 0x00, 0x20, // MOV DI, 0x2000
            0xB9, 0x05, 0x00, // MOV CX, 5
            0xF3, 0xA6, // REPE CMPSB
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV SI, 0x1000
    harness.step(); // MOV DI, 0x2000
    harness.step(); // MOV CX, 5

    // Execute REPE CMPSB - should repeat 5 times (all equal)
    for _ in 0..5 {
        harness.step();
    }

    // All bytes were equal, CX should be 0, ZF should be set
    assert_eq!(harness.cpu.read_reg16(1), 0); // CX
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF));
    assert_eq!(harness.cpu.read_reg16(6), 0x1005);
    assert_eq!(harness.cpu.read_reg16(7), 0x2005);
}

#[test]
fn test_repe_cmpsb_early_mismatch() {
    let mut harness = CpuHarness::new();

    // Setup strings that differ at position 2
    harness.mem.write_u8(0x1000, 0x41); // A
    harness.mem.write_u8(0x1001, 0x42); // B
    harness.mem.write_u8(0x1002, 0x43); // C
    harness.mem.write_u8(0x2000, 0x41); // A
    harness.mem.write_u8(0x2001, 0x42); // B
    harness.mem.write_u8(0x2002, 0x58); // X (different!)

    // CLD; MOV SI, 0x1000; MOV DI, 0x2000; MOV CX, 5; REPE CMPSB
    harness.load_program(
        &[
            0xFC, // CLD
            0xBE, 0x00, 0x10, // MOV SI, 0x1000
            0xBF, 0x00, 0x20, // MOV DI, 0x2000
            0xB9, 0x05, 0x00, // MOV CX, 5
            0xF3, 0xA6, // REPE CMPSB
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV SI, 0x1000
    harness.step(); // MOV DI, 0x2000
    harness.step(); // MOV CX, 5

    // Execute REPE CMPSB - should stop at position 2 (3rd comparison)
    harness.step(); // Compare 0: A==A, continue
    harness.step(); // Compare 1: B==B, continue
    harness.step(); // Compare 2: C!=X, stop

    // Should have stopped early, CX should be 2 (5 - 3 comparisons)
    assert_eq!(harness.cpu.read_reg16(1), 2); // CX
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // Not equal
    assert_eq!(harness.cpu.read_reg16(6), 0x1003);
    assert_eq!(harness.cpu.read_reg16(7), 0x2003);
}

#[test]
fn test_repne_cmpsb_find_match() {
    let mut harness = CpuHarness::new();

    // Setup strings with first match at position 3
    harness.mem.write_u8(0x1000, 0x41); // A
    harness.mem.write_u8(0x1001, 0x42); // B
    harness.mem.write_u8(0x1002, 0x43); // C
    harness.mem.write_u8(0x1003, 0x44); // D
    harness.mem.write_u8(0x2000, 0x58); // X (different)
    harness.mem.write_u8(0x2001, 0x59); // Y (different)
    harness.mem.write_u8(0x2002, 0x5A); // Z (different)
    harness.mem.write_u8(0x2003, 0x44); // D (same!)

    // CLD; MOV SI, 0x1000; MOV DI, 0x2000; MOV CX, 5; REPNE CMPSB
    harness.load_program(
        &[
            0xFC, // CLD
            0xBE, 0x00, 0x10, // MOV SI, 0x1000
            0xBF, 0x00, 0x20, // MOV DI, 0x2000
            0xB9, 0x05, 0x00, // MOV CX, 5
            0xF2, 0xA6, // REPNE CMPSB
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV SI, 0x1000
    harness.step(); // MOV DI, 0x2000
    harness.step(); // MOV CX, 5

    // Execute REPNE CMPSB - should stop when it finds a match at position 3
    harness.step(); // Compare 0: A!=X, continue
    harness.step(); // Compare 1: B!=Y, continue
    harness.step(); // Compare 2: C!=Z, continue
    harness.step(); // Compare 3: D==D, stop

    // Should have stopped when equal found, CX should be 1 (5 - 4 comparisons)
    assert_eq!(harness.cpu.read_reg16(1), 1); // CX
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // Equal
    assert_eq!(harness.cpu.read_reg16(6), 0x1004);
    assert_eq!(harness.cpu.read_reg16(7), 0x2004);
}

//
// SCASB/SCASW Tests
//

#[test]
fn test_scasb_found() {
    let mut harness = CpuHarness::new();

    // Setup memory with search value
    harness.mem.write_u8(0x1000, 0x55);

    // CLD; MOV AL, 0x55; MOV DI, 0x1000; SCASB
    harness.load_program(
        &[
            0xFC, // CLD
            0xB0, 0x55, // MOV AL, 0x55
            0xBF, 0x00, 0x10, // MOV DI, 0x1000
            0xAE, // SCASB
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV AL, 0x55
    harness.step(); // MOV DI, 0x1000
    harness.step(); // SCASB

    // AL == [ES:DI], so ZF should be set
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF));
    // DI should have advanced
    assert_eq!(harness.cpu.read_reg16(7), 0x1001);
}

#[test]
fn test_scasb_not_found() {
    let mut harness = CpuHarness::new();

    // Setup memory with different value
    harness.mem.write_u8(0x1000, 0x99);

    // CLD; MOV AL, 0x55; MOV DI, 0x1000; SCASB
    harness.load_program(
        &[
            0xFC, // CLD
            0xB0, 0x55, // MOV AL, 0x55
            0xBF, 0x00, 0x10, // MOV DI, 0x1000
            0xAE, // SCASB
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV AL, 0x55
    harness.step(); // MOV DI, 0x1000
    harness.step(); // SCASB

    // AL != [ES:DI], so ZF should be clear
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF));
    // DI should have advanced
    assert_eq!(harness.cpu.read_reg16(7), 0x1001);
}

#[test]
fn test_scasw_found() {
    let mut harness = CpuHarness::new();

    // Setup memory with search value
    harness.mem.write_u16(0x1000, 0xBEEF);

    // CLD; MOV AX, 0xBEEF; MOV DI, 0x1000; SCASW
    harness.load_program(
        &[
            0xFC, // CLD
            0xB8, 0xEF, 0xBE, // MOV AX, 0xBEEF
            0xBF, 0x00, 0x10, // MOV DI, 0x1000
            0xAF, // SCASW
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV AX, 0xBEEF
    harness.step(); // MOV DI, 0x1000
    harness.step(); // SCASW

    // AX == [ES:DI], so ZF should be set
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF));
    // DI should have advanced by 2
    assert_eq!(harness.cpu.read_reg16(7), 0x1002);
}

#[test]
fn test_repne_scasb_find_char() {
    let mut harness = CpuHarness::new();

    // Setup string with search char at position 4
    harness.mem.write_u8(0x1000, 0x48); // H
    harness.mem.write_u8(0x1001, 0x45); // E
    harness.mem.write_u8(0x1002, 0x4C); // L
    harness.mem.write_u8(0x1003, 0x4C); // L
    harness.mem.write_u8(0x1004, 0x4F); // O

    // CLD; MOV AL, 0x4F; MOV DI, 0x1000; MOV CX, 10; REPNE SCASB
    harness.load_program(
        &[
            0xFC, // CLD
            0xB0, 0x4F, // MOV AL, 'O' (0x4F)
            0xBF, 0x00, 0x10, // MOV DI, 0x1000
            0xB9, 0x0A, 0x00, // MOV CX, 10
            0xF2, 0xAE, // REPNE SCASB
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV AL, 0x4F
    harness.step(); // MOV DI, 0x1000
    harness.step(); // MOV CX, 10

    // Execute REPNE SCASB - should find 'O' at position 4
    harness.step(); // Compare 0: AL != H, continue
    harness.step(); // Compare 1: AL != E, continue
    harness.step(); // Compare 2: AL != L, continue
    harness.step(); // Compare 3: AL != L, continue
    harness.step(); // Compare 4: AL == O, stop

    // Should have found the character, CX should be 5 (10 - 5 scans)
    assert_eq!(harness.cpu.read_reg16(1), 5); // CX
    assert!(harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // Found (equal)
    assert_eq!(harness.cpu.read_reg16(7), 0x1005);
}

#[test]
fn test_repe_scasb_count_repeated_char() {
    let mut harness = CpuHarness::new();

    // Setup string with 4 'A's followed by 'B'
    for i in 0..4 {
        harness.mem.write_u8(0x1000 + i, 0x41); // A
    }
    harness.mem.write_u8(0x1004, 0x42); // B

    // CLD; MOV AL, 0x41; MOV DI, 0x1000; MOV CX, 10; REPE SCASB
    harness.load_program(
        &[
            0xFC, // CLD
            0xB0, 0x41, // MOV AL, 'A' (0x41)
            0xBF, 0x00, 0x10, // MOV DI, 0x1000
            0xB9, 0x0A, 0x00, // MOV CX, 10
            0xF3, 0xAE, // REPE SCASB
        ],
        0,
    );

    harness.step(); // CLD
    harness.step(); // MOV AL, 0x41
    harness.step(); // MOV DI, 0x1000
    harness.step(); // MOV CX, 10

    // Execute REPE SCASB - should stop when it hits 'B'
    harness.step(); // Compare 0: AL == A, continue
    harness.step(); // Compare 1: AL == A, continue
    harness.step(); // Compare 2: AL == A, continue
    harness.step(); // Compare 3: AL == A, continue
    harness.step(); // Compare 4: AL != B, stop

    // Should have stopped at 'B', CX should be 5 (10 - 5 scans)
    assert_eq!(harness.cpu.read_reg16(1), 5); // CX
    assert!(!harness.cpu.get_flag(ezpc::cpu::Cpu::ZF)); // Not equal
    assert_eq!(harness.cpu.read_reg16(7), 0x1005);
}
