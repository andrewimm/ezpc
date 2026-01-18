//! Tests for tier 2 decode cache functionality

use ezpc::cpu::CpuHarness;

/// Test that cache hits work correctly for a simple loop
#[test]
fn test_cache_hits_in_loop() {
    let mut harness = CpuHarness::new();

    // Simple loop: DEC CX; JNZ -2 (loop back)
    // 0x49 = DEC CX (1 byte)
    // 0x75 0xFD = JNZ -3 (2 bytes, jumps back 3 bytes to DEC CX)
    harness.load_program(&[0x49, 0x75, 0xFD], 0);
    harness.cpu.regs[1] = 5; // CX = 5, loop 5 times

    // Execute the loop 5 times (each iteration = 2 instructions)
    for _ in 0..10 {
        harness.step();
    }

    // CX should be 0 after 5 decrements
    assert_eq!(harness.cpu.regs[1], 0, "CX should be 0 after loop");

    // Cache should have hits from repeated executions
    assert!(
        harness.cpu.decode_cache.total_hits() > 0,
        "Cache should have hits from loop execution"
    );
}

/// Test that cache statistics are tracked correctly
#[test]
fn test_cache_statistics() {
    let mut harness = CpuHarness::new();

    // Simple program: NOP; NOP; NOP
    harness.load_program(&[0x90, 0x90, 0x90], 0);

    // First execution - all misses
    harness.step(); // NOP at offset 0 - miss
    harness.step(); // NOP at offset 1 - miss
    harness.step(); // NOP at offset 2 - miss

    let misses_after_first_run = harness.cpu.decode_cache.total_misses();
    assert_eq!(
        misses_after_first_run, 3,
        "Should have 3 misses on first run"
    );

    // Reset IP to start of program
    harness.cpu.ip = 0;

    // Second execution - all hits
    harness.step(); // NOP at offset 0 - hit
    harness.step(); // NOP at offset 1 - hit
    harness.step(); // NOP at offset 2 - hit

    let hits_after_second_run = harness.cpu.decode_cache.total_hits();
    assert_eq!(hits_after_second_run, 3, "Should have 3 hits on second run");
}

/// Test that cache is invalidated when memory is written via CPU
#[test]
fn test_cache_invalidation_on_write() {
    let mut harness = CpuHarness::new();

    // Program: MOV AX, 0x1234
    // B8 34 12 = MOV AX, 0x1234
    harness.load_program(&[0xB8, 0x34, 0x12], 0);

    // Execute once to cache the instruction
    harness.step();
    assert_eq!(harness.cpu.regs[0], 0x1234, "AX should be 0x1234");

    // Verify instruction is cached (reset IP and run again)
    harness.cpu.ip = 0;
    harness.step();
    assert!(
        harness.cpu.decode_cache.total_hits() >= 1,
        "Should have cache hit"
    );

    // Now modify the instruction in memory using CPU write method
    // Change immediate value from 0x1234 to 0x5678
    // Physical address 0 (CS=0, offset=0) + 1 = address of low byte
    let segment = harness.cpu.segments[1]; // CS
    harness
        .cpu
        .write_mem16(&mut harness.mem, segment, 1, 0x5678);

    // Reset IP and execute - should get new value (cache was invalidated)
    harness.cpu.ip = 0;
    harness.step();
    assert_eq!(
        harness.cpu.regs[0], 0x5678,
        "AX should be 0x5678 after self-modifying code"
    );
}

/// Test that hit count tracking per instruction works
#[test]
fn test_hit_count_tracking() {
    let mut harness = CpuHarness::new();

    // Simple program: NOP
    harness.load_program(&[0x90], 0);

    // Execute NOP 10 times
    for _ in 0..10 {
        harness.cpu.ip = 0;
        harness.step();
    }

    // Check that the cache entry has recorded hits
    let addr = 0u32; // Physical address of NOP (CS=0, IP=0)
    if let Some(entry) = harness.cpu.decode_cache.get(addr) {
        // Note: get() increments hit count, so we see one more
        assert!(
            entry.hit_count >= 9,
            "Hit count should be at least 9 (10 executions - 1 miss)"
        );
    } else {
        panic!("Cache entry should exist");
    }
}

/// Test that cache clear works
#[test]
fn test_cache_clear() {
    let mut harness = CpuHarness::new();

    // Load and execute a program to populate cache
    harness.load_program(&[0x90, 0x90, 0x90], 0);
    harness.step();
    harness.step();
    harness.step();

    assert_eq!(
        harness.cpu.decode_cache.len(),
        3,
        "Cache should have 3 entries"
    );

    // Clear the cache
    harness.cpu.decode_cache.clear();

    assert!(
        harness.cpu.decode_cache.is_empty(),
        "Cache should be empty after clear"
    );
}

/// Test that segment override instructions bypass cache correctly
#[test]
fn test_segment_override_bypasses_cache() {
    let mut harness = CpuHarness::new();

    // Set up different segments
    harness.cpu.segments[0] = 0x0100; // ES = 0x0100
    harness.cpu.segments[3] = 0x0200; // DS = 0x0200
    harness.cpu.regs[3] = 0x0000; // BX = 0

    // Write different values at ES:0 and DS:0
    harness.mem.write_u8(0x01000, 0xAA); // ES:0
    harness.mem.write_u8(0x02000, 0x55); // DS:0

    // First: MOV AL, [BX] without override (uses DS)
    harness.load_program(&[0x8A, 0x07], 0);
    harness.step();
    assert_eq!(harness.cpu.read_reg8(0), 0x55, "Should read from DS");

    // Second: ES: MOV AL, [BX] with override (uses ES)
    harness.load_program(&[0x26, 0x8A, 0x07], 0);
    harness.step();
    assert_eq!(
        harness.cpu.read_reg8(0),
        0xAA,
        "Should read from ES with override"
    );

    // Third: Back to MOV AL, [BX] without override (should still use DS)
    harness.load_program(&[0x8A, 0x07], 0);
    harness.step();
    assert_eq!(harness.cpu.read_reg8(0), 0x55, "Should still read from DS");
}

/// Test cache hit rate calculation
#[test]
fn test_cache_hit_rate() {
    let mut harness = CpuHarness::new();

    // Execute a loop to get some hits and misses
    // NOP; JMP -1 (loop forever)
    harness.load_program(&[0x90, 0xEB, 0xFD], 0);

    // Execute 20 instructions (first 3 are misses, rest are hits)
    for _ in 0..20 {
        harness.step();
    }

    let hit_rate = harness.cpu.decode_cache.hit_rate();
    assert!(hit_rate > 0.5, "Hit rate should be > 50% for a tight loop");
}
