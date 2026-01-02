//! Stack operation instruction tests (PUSH, POP)

use ezpc::cpu::CpuHarness;

#[test]
fn test_push_pop() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x1000; MOV AX, 0x1234; PUSH AX; MOV AX, 0; POP AX
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
            0x50, // PUSH AX
            0xB8, 0x00, 0x00, // MOV AX, 0
            0x58, // POP AX
        ],
        0,
    );

    harness.step(); // MOV SP, 0x1000
    assert_eq!(harness.cpu.regs[4], 0x1000); // SP

    harness.step(); // MOV AX, 0x1234
    assert_eq!(harness.cpu.regs[0], 0x1234); // AX

    harness.step(); // PUSH AX
    assert_eq!(harness.cpu.regs[4], 0x0FFE); // SP should decrement by 2

    harness.step(); // MOV AX, 0
    assert_eq!(harness.cpu.regs[0], 0x0000); // AX cleared

    harness.step(); // POP AX
    assert_eq!(harness.cpu.regs[0], 0x1234); // AX restored
    assert_eq!(harness.cpu.regs[4], 0x1000); // SP back to original
}
