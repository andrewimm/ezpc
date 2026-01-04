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

#[test]
fn test_push_pop_es() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x1000; PUSH ES; POP ES
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000
            0x06, // PUSH ES
            0x07, // POP ES
        ],
        0,
    );

    // Set ES to a known value
    harness.cpu.write_seg(0, 0x5678); // ES

    harness.step(); // MOV SP, 0x1000
    assert_eq!(harness.cpu.regs[4], 0x1000); // SP

    harness.step(); // PUSH ES
    assert_eq!(harness.cpu.regs[4], 0x0FFE); // SP should decrement by 2
                                             // Verify ES was pushed to stack
    let pushed_value = harness.mem.read_u16(0x0FFE);
    assert_eq!(pushed_value, 0x5678);

    // Change ES to verify POP works
    harness.cpu.write_seg(0, 0x0000); // ES
    assert_eq!(harness.cpu.read_seg(0), 0x0000);

    harness.step(); // POP ES
    assert_eq!(harness.cpu.read_seg(0), 0x5678); // ES restored
    assert_eq!(harness.cpu.regs[4], 0x1000); // SP back to original
}

#[test]
fn test_push_cs() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x1000; PUSH CS
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000
            0x0E, // PUSH CS
        ],
        0,
    );

    // CS will be 0 after load_program
    harness.step(); // MOV SP, 0x1000
    assert_eq!(harness.cpu.regs[4], 0x1000); // SP

    harness.step(); // PUSH CS
    assert_eq!(harness.cpu.regs[4], 0x0FFE); // SP should decrement by 2
                                             // Verify CS was pushed to stack (CS should be 0)
    let pushed_value = harness.mem.read_u16(0x0FFE);
    assert_eq!(pushed_value, 0x0000);
}

#[test]
fn test_push_pop_ss() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x1000; MOV AX, 0x5678; PUSH AX; PUSH SS; POP AX; POP SS
    // We push AX first with a known value, then push SS (which is 0).
    // Then we pop into AX (getting 0), and pop into SS (getting 0x5678).
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000
            0xB8, 0x78, 0x56, // MOV AX, 0x5678
            0x50, // PUSH AX
            0x16, // PUSH SS
            0x58, // POP AX (should get 0 from SS)
            0x17, // POP SS (should get 0x5678 from original AX push)
        ],
        0,
    );

    harness.step(); // MOV SP, 0x1000
    assert_eq!(harness.cpu.regs[4], 0x1000);

    harness.step(); // MOV AX, 0x5678
    assert_eq!(harness.cpu.regs[0], 0x5678);

    harness.step(); // PUSH AX
    assert_eq!(harness.cpu.regs[4], 0x0FFE);
    assert_eq!(harness.mem.read_u16(0x0FFE), 0x5678);

    harness.step(); // PUSH SS
    assert_eq!(harness.cpu.regs[4], 0x0FFC);
    assert_eq!(harness.mem.read_u16(0x0FFC), 0x0000); // SS is 0

    harness.step(); // POP AX
    assert_eq!(harness.cpu.regs[0], 0x0000); // AX gets SS value (0)
    assert_eq!(harness.cpu.regs[4], 0x0FFE);

    harness.step(); // POP SS
    assert_eq!(harness.cpu.read_seg(2), 0x5678); // SS gets original AX value
    assert_eq!(harness.cpu.regs[4], 0x1000);
}

#[test]
fn test_push_pop_ds() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x1000; PUSH DS; POP DS
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000
            0x1E, // PUSH DS
            0x1F, // POP DS
        ],
        0,
    );

    // Set DS to a known value
    harness.cpu.write_seg(3, 0xDEF0); // DS

    harness.step(); // MOV SP, 0x1000
    assert_eq!(harness.cpu.regs[4], 0x1000); // SP

    harness.step(); // PUSH DS
    assert_eq!(harness.cpu.regs[4], 0x0FFE); // SP should decrement by 2
                                             // Verify DS was pushed to stack
    let pushed_value = harness.mem.read_u16(0x0FFE);
    assert_eq!(pushed_value, 0xDEF0);

    // Change DS to verify POP works
    harness.cpu.write_seg(3, 0x0000); // DS
    assert_eq!(harness.cpu.read_seg(3), 0x0000);

    harness.step(); // POP DS
    assert_eq!(harness.cpu.read_seg(3), 0xDEF0); // DS restored
    assert_eq!(harness.cpu.regs[4], 0x1000); // SP back to original
}

#[test]
fn test_push_all_segments() {
    let mut harness = CpuHarness::new();
    // MOV SP, 0x1000; PUSH ES; PUSH CS; PUSH SS; PUSH DS
    harness.load_program(
        &[
            0xBC, 0x00, 0x10, // MOV SP, 0x1000
            0x06, // PUSH ES
            0x0E, // PUSH CS
            0x16, // PUSH SS
            0x1E, // PUSH DS
        ],
        0,
    );

    // Set segment registers to known values (but keep CS and SS at 0 to avoid issues)
    harness.cpu.write_seg(0, 0x1111); // ES (safe to change, not used for fetching)
    harness.cpu.write_seg(3, 0x4444); // DS (safe to change, not used for fetching)
                                      // CS and SS remain 0

    harness.step(); // MOV SP, 0x1000
    assert_eq!(harness.cpu.regs[4], 0x1000);

    harness.step(); // PUSH ES
    assert_eq!(harness.cpu.regs[4], 0x0FFE);
    assert_eq!(harness.mem.read_u16(0x0FFE), 0x1111);

    harness.step(); // PUSH CS
    assert_eq!(harness.cpu.regs[4], 0x0FFC);
    assert_eq!(harness.mem.read_u16(0x0FFC), 0x0000); // CS is 0

    harness.step(); // PUSH SS
    assert_eq!(harness.cpu.regs[4], 0x0FFA);
    assert_eq!(harness.mem.read_u16(0x0FFA), 0x0000); // SS is 0

    harness.step(); // PUSH DS
    assert_eq!(harness.cpu.regs[4], 0x0FF8);
    assert_eq!(harness.mem.read_u16(0x0FF8), 0x4444);
}
