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
