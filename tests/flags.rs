//! Flag manipulation instruction tests

use ezpc::cpu::Cpu;
use ezpc::cpu::CpuHarness;

#[test]
fn test_clc() {
    let mut harness = CpuHarness::new();
    // Set carry flag first
    harness.cpu.set_flag(Cpu::CF, true);
    assert!(harness.cpu.get_flag(Cpu::CF));

    // CLC
    harness.load_program(&[0xF8], 0);
    harness.step();

    // Carry flag should be cleared
    assert!(!harness.cpu.get_flag(Cpu::CF));
    assert_eq!(harness.cpu.ip, 1);
}

#[test]
fn test_stc() {
    let mut harness = CpuHarness::new();
    // Ensure carry flag is clear
    harness.cpu.set_flag(Cpu::CF, false);
    assert!(!harness.cpu.get_flag(Cpu::CF));

    // STC
    harness.load_program(&[0xF9], 0);
    harness.step();

    // Carry flag should be set
    assert!(harness.cpu.get_flag(Cpu::CF));
    assert_eq!(harness.cpu.ip, 1);
}

#[test]
fn test_cmc() {
    let mut harness = CpuHarness::new();
    // CMC, CMC
    harness.load_program(&[0xF5, 0xF5], 0);

    // Start with CF=0
    harness.cpu.set_flag(Cpu::CF, false);

    // First CMC: 0 -> 1
    harness.step();
    assert!(harness.cpu.get_flag(Cpu::CF));

    // Second CMC: 1 -> 0
    harness.step();
    assert!(!harness.cpu.get_flag(Cpu::CF));
    assert_eq!(harness.cpu.ip, 2);
}

#[test]
fn test_cli() {
    let mut harness = CpuHarness::new();
    // Set interrupt flag first
    harness.cpu.set_flag(Cpu::IF, true);
    assert!(harness.cpu.get_flag(Cpu::IF));

    // CLI
    harness.load_program(&[0xFA], 0);
    harness.step();

    // Interrupt flag should be cleared
    assert!(!harness.cpu.get_flag(Cpu::IF));
    assert_eq!(harness.cpu.ip, 1);
}

#[test]
fn test_sti() {
    let mut harness = CpuHarness::new();
    // Ensure interrupt flag is clear
    harness.cpu.set_flag(Cpu::IF, false);
    assert!(!harness.cpu.get_flag(Cpu::IF));

    // STI
    harness.load_program(&[0xFB], 0);
    harness.step();

    // Interrupt flag should be set
    assert!(harness.cpu.get_flag(Cpu::IF));
    assert_eq!(harness.cpu.ip, 1);
}

#[test]
fn test_cld() {
    let mut harness = CpuHarness::new();
    // Set direction flag first
    harness.cpu.set_flag(Cpu::DF, true);
    assert!(harness.cpu.get_flag(Cpu::DF));

    // CLD
    harness.load_program(&[0xFC], 0);
    harness.step();

    // Direction flag should be cleared
    assert!(!harness.cpu.get_flag(Cpu::DF));
    assert_eq!(harness.cpu.ip, 1);
}

#[test]
fn test_std() {
    let mut harness = CpuHarness::new();
    // Ensure direction flag is clear
    harness.cpu.set_flag(Cpu::DF, false);
    assert!(!harness.cpu.get_flag(Cpu::DF));

    // STD
    harness.load_program(&[0xFD], 0);
    harness.step();

    // Direction flag should be set
    assert!(harness.cpu.get_flag(Cpu::DF));
    assert_eq!(harness.cpu.ip, 1);
}

#[test]
fn test_flag_sequence() {
    let mut harness = CpuHarness::new();
    // CLC, STC, CLI, STI, CLD, STD
    harness.load_program(&[0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD], 0);

    // Execute CLC
    harness.step();
    assert!(!harness.cpu.get_flag(Cpu::CF));
    assert_eq!(harness.cpu.ip, 1);

    // Execute STC
    harness.step();
    assert!(harness.cpu.get_flag(Cpu::CF));
    assert_eq!(harness.cpu.ip, 2);

    // Execute CLI
    harness.step();
    assert!(!harness.cpu.get_flag(Cpu::IF));
    assert_eq!(harness.cpu.ip, 3);

    // Execute STI
    harness.step();
    assert!(harness.cpu.get_flag(Cpu::IF));
    assert_eq!(harness.cpu.ip, 4);

    // Execute CLD
    harness.step();
    assert!(!harness.cpu.get_flag(Cpu::DF));
    assert_eq!(harness.cpu.ip, 5);

    // Execute STD
    harness.step();
    assert!(harness.cpu.get_flag(Cpu::DF));
    assert_eq!(harness.cpu.ip, 6);
}

#[test]
fn test_flags_dont_affect_other_flags() {
    let mut harness = CpuHarness::new();

    // Set all flags we care about
    harness.cpu.set_flag(Cpu::CF, true);
    harness.cpu.set_flag(Cpu::IF, true);
    harness.cpu.set_flag(Cpu::DF, true);
    harness.cpu.set_flag(Cpu::ZF, true);
    harness.cpu.set_flag(Cpu::SF, true);

    // CLD should only clear DF
    harness.load_program(&[0xFC], 0);
    harness.step();

    assert!(!harness.cpu.get_flag(Cpu::DF)); // Cleared
    assert!(harness.cpu.get_flag(Cpu::CF)); // Unchanged
    assert!(harness.cpu.get_flag(Cpu::IF)); // Unchanged
    assert!(harness.cpu.get_flag(Cpu::ZF)); // Unchanged
    assert!(harness.cpu.get_flag(Cpu::SF)); // Unchanged
}

#[test]
fn test_pushf() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x1000; PUSHF
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000
            0x9C, // PUSHF
        ],
        0,
    );

    // Set some flags to a known state
    harness.cpu.set_flag(Cpu::CF, true);
    harness.cpu.set_flag(Cpu::ZF, true);
    harness.cpu.set_flag(Cpu::SF, false);
    harness.cpu.set_flag(Cpu::OF, true);

    harness.step(); // MOV SP, 0x1000
    assert_eq!(harness.cpu.regs[4], 0x1000); // SP

    let flags_before = harness.cpu.get_flags();

    harness.step(); // PUSHF
    assert_eq!(harness.cpu.regs[4], 0x0FFE); // SP should decrement by 2

    // Verify FLAGS was pushed to stack
    let pushed_flags = harness.mem.read_u16(0x0FFE);
    assert_eq!(pushed_flags, flags_before);
    assert_eq!(harness.cpu.ip, 4);
}

#[test]
fn test_popf() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x1000; PUSHF; POPF
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000
            0x9C, // PUSHF
            0x9D, // POPF
        ],
        0,
    );

    // Set flags to a known state
    harness.cpu.set_flag(Cpu::CF, true);
    harness.cpu.set_flag(Cpu::ZF, false);
    harness.cpu.set_flag(Cpu::SF, true);
    harness.cpu.set_flag(Cpu::IF, true);

    harness.step(); // MOV SP, 0x1000
    let flags_before = harness.cpu.get_flags();

    harness.step(); // PUSHF
    assert_eq!(harness.cpu.regs[4], 0x0FFE); // SP

    // Clear all flags to verify POPF works
    harness.cpu.set_flag(Cpu::CF, false);
    harness.cpu.set_flag(Cpu::ZF, false);
    harness.cpu.set_flag(Cpu::SF, false);
    harness.cpu.set_flag(Cpu::IF, false);

    harness.step(); // POPF
    assert_eq!(harness.cpu.regs[4], 0x1000); // SP back to original

    // Verify flags were restored
    let flags_after = harness.cpu.get_flags();
    assert_eq!(flags_after, flags_before);
    assert_eq!(harness.cpu.ip, 5);
}

#[test]
fn test_sahf() {
    let mut harness = CpuHarness::new();
    // MOV AH, 0xD5; SAHF
    // 0xD5 = 11010101b = SF=1, ZF=1, AF=1, PF=1, CF=1
    harness.load_program(
        &[
            0xB4, 0xD5, // MOV AH, 0xD5
            0x9E, // SAHF
        ],
        0,
    );

    // Set some high flags that should not be affected
    harness.cpu.set_flag(Cpu::OF, true);
    harness.cpu.set_flag(Cpu::IF, true);
    harness.cpu.set_flag(Cpu::DF, true);

    harness.step(); // MOV AH, 0xD5
    assert_eq!(harness.cpu.read_reg8(4), 0xD5); // AH

    harness.step(); // SAHF

    // Check that the low flags were set from AH
    assert!(harness.cpu.get_flag(Cpu::SF)); // Bit 7
    assert!(harness.cpu.get_flag(Cpu::ZF)); // Bit 6
    assert!(harness.cpu.get_flag(Cpu::AF)); // Bit 4
    assert!(harness.cpu.get_flag(Cpu::PF)); // Bit 2
    assert!(harness.cpu.get_flag(Cpu::CF)); // Bit 0

    // High flags should be unchanged
    assert!(harness.cpu.get_flag(Cpu::OF));
    assert!(harness.cpu.get_flag(Cpu::IF));
    assert!(harness.cpu.get_flag(Cpu::DF));
    assert_eq!(harness.cpu.ip, 3);
}

#[test]
fn test_lahf() {
    let mut harness = CpuHarness::new();
    // LAHF
    harness.load_program(&[0x9F], 0);

    // Set flags to a known pattern
    harness.cpu.set_flag(Cpu::SF, true); // Bit 7
    harness.cpu.set_flag(Cpu::ZF, false); // Bit 6
    harness.cpu.set_flag(Cpu::AF, true); // Bit 4
    harness.cpu.set_flag(Cpu::PF, true); // Bit 2
    harness.cpu.set_flag(Cpu::CF, true); // Bit 0
                                         // Bit 1 is always set

    // Clear AH to verify LAHF works
    harness.cpu.write_reg8(4, 0x00); // AH
    assert_eq!(harness.cpu.read_reg8(4), 0x00);

    harness.step(); // LAHF

    // AH should now contain the low byte of FLAGS
    // Expected: SF(1) ZF(0) 0 AF(1) 0 PF(1) 1 CF(1) = 10010111b = 0x97
    let ah = harness.cpu.read_reg8(4);
    assert_eq!(ah, 0x97);
    assert_eq!(harness.cpu.ip, 1);
}

#[test]
fn test_sahf_lahf_roundtrip() {
    let mut harness = CpuHarness::new();
    // MOV AH, 0xC5; SAHF; LAHF
    harness.load_program(
        &[
            0xB4, 0xC5, // MOV AH, 0xC5
            0x9E, // SAHF
            0x9F, // LAHF
        ],
        0,
    );

    harness.step(); // MOV AH, 0xC5

    harness.step(); // SAHF

    // Clear AH to test LAHF
    harness.cpu.write_reg8(4, 0x00);

    harness.step(); // LAHF

    // AH should be restored (with bit 1 always set)
    let ah_after = harness.cpu.read_reg8(4);
    // 0xC5 = 11000101, bit 1 is already 0, so with bit 1 forced to 1: 11000111 = 0xC7
    assert_eq!(ah_after, 0xC7);
}

#[test]
fn test_pushf_popf_preserves_all_flags() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x1000; PUSHF; POPF
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000
            0x9C, // PUSHF
            0x9D, // POPF
        ],
        0,
    );

    // Set all flags to a known pattern
    harness.cpu.set_flag(Cpu::CF, true);
    harness.cpu.set_flag(Cpu::PF, false);
    harness.cpu.set_flag(Cpu::AF, true);
    harness.cpu.set_flag(Cpu::ZF, false);
    harness.cpu.set_flag(Cpu::SF, true);
    harness.cpu.set_flag(Cpu::TF, false);
    harness.cpu.set_flag(Cpu::IF, true);
    harness.cpu.set_flag(Cpu::DF, false);
    harness.cpu.set_flag(Cpu::OF, true);

    harness.step(); // MOV SP, 0x1000

    let flags_before = harness.cpu.get_flags();

    harness.step(); // PUSHF
    harness.step(); // POPF

    let flags_after = harness.cpu.get_flags();
    assert_eq!(flags_after, flags_before);
}
