//! Basic data transfer instruction tests (MOV, XCHG, NOP)

use ezpc::cpu::CpuHarness;

#[test]
fn test_nop() {
    let mut harness = CpuHarness::new();
    harness.load_program(&[0x90], 0); // NOP

    // Execute NOP
    harness.step();

    // IP should have advanced by 1
    assert_eq!(harness.cpu.ip, 1);
}

#[test]
fn test_mov_r16_imm() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1234; MOV CX, 0x5678
    harness.load_program(&[0xB8, 0x34, 0x12, 0xB9, 0x78, 0x56], 0);

    // Execute MOV AX, 0x1234
    harness.step();
    assert_eq!(harness.cpu.regs[0], 0x1234); // AX
    assert_eq!(harness.cpu.ip, 3);

    // Execute MOV CX, 0x5678
    harness.step();
    assert_eq!(harness.cpu.regs[1], 0x5678); // CX
    assert_eq!(harness.cpu.ip, 6);
}

#[test]
fn test_mov_r16_imm_all_registers() {
    let mut harness = CpuHarness::new();
    // Test all 16-bit registers: AX, CX, DX, BX, SP, BP, SI, DI
    // MOV AX, 0x1111; MOV CX, 0x2222; MOV DX, 0x3333; MOV BX, 0x4444
    // MOV SP, 0x5555; MOV BP, 0x6666; MOV SI, 0x7777; MOV DI, 0x8888
    harness.load_program(
        &[
            0xB8, 0x11, 0x11, // MOV AX, 0x1111
            0xB9, 0x22, 0x22, // MOV CX, 0x2222
            0xBA, 0x33, 0x33, // MOV DX, 0x3333
            0xBB, 0x44, 0x44, // MOV BX, 0x4444
            0xBC, 0x55, 0x55, // MOV SP, 0x5555
            0xBD, 0x66, 0x66, // MOV BP, 0x6666
            0xBE, 0x77, 0x77, // MOV SI, 0x7777
            0xBF, 0x88, 0x88, // MOV DI, 0x8888
        ],
        0,
    );

    // Execute and verify each MOV instruction
    harness.step();
    assert_eq!(harness.cpu.regs[0], 0x1111); // AX

    harness.step();
    assert_eq!(harness.cpu.regs[1], 0x2222); // CX

    harness.step();
    assert_eq!(harness.cpu.regs[2], 0x3333); // DX

    harness.step();
    assert_eq!(harness.cpu.regs[3], 0x4444); // BX

    harness.step();
    assert_eq!(harness.cpu.regs[4], 0x5555); // SP

    harness.step();
    assert_eq!(harness.cpu.regs[5], 0x6666); // BP

    harness.step();
    assert_eq!(harness.cpu.regs[6], 0x7777); // SI

    harness.step();
    assert_eq!(harness.cpu.regs[7], 0x8888); // DI
}

#[test]
fn test_mov_bp_specific() {
    let mut harness = CpuHarness::new();
    // MOV BP, 0x0472 (the exact instruction from the BIOS)
    harness.load_program(&[0xBD, 0x72, 0x04], 0);

    harness.step();
    assert_eq!(harness.cpu.regs[5], 0x0472); // BP
    assert_eq!(harness.cpu.ip, 3);
}

#[test]
fn test_mov_r16_direct_address() {
    let mut harness = CpuHarness::new();
    // Set up memory at address 0x0472 with a test value
    harness.mem.write_u16(0x0472, 0xABCD);

    // MOV BP, [0x0472] - the actual problematic instruction: 0x8B 0x2E 0x72 0x04
    harness.load_program(&[0x8B, 0x2E, 0x72, 0x04], 0);

    harness.step();
    assert_eq!(harness.cpu.regs[5], 0xABCD); // BP should contain value from [0x0472]
    assert_eq!(harness.cpu.ip, 4);
}

#[test]
fn test_mov_direct_address_all_regs() {
    let mut harness = CpuHarness::new();
    // Test direct addressing with different registers
    harness.mem.write_u16(0x1000, 0x1111);
    harness.mem.write_u16(0x2000, 0x2222);

    // MOV AX, [0x1000] - 0x8B 0x06 0x00 0x10
    harness.load_program(&[0x8B, 0x06, 0x00, 0x10], 0);
    harness.step();
    assert_eq!(harness.cpu.regs[0], 0x1111); // AX

    // MOV CX, [0x2000] - 0x8B 0x0E 0x00 0x20
    harness.load_program(&[0x8B, 0x0E, 0x00, 0x20], 0);
    harness.cpu.ip = 0;
    harness.step();
    assert_eq!(harness.cpu.regs[1], 0x2222); // CX
}

#[test]
fn test_mov_r8_imm() {
    let mut harness = CpuHarness::new();
    // MOV AL, 0x12; MOV AH, 0x34
    harness.load_program(&[0xB0, 0x12, 0xB4, 0x34], 0);

    // Execute MOV AL, 0x12
    harness.step();
    assert_eq!(harness.cpu.read_reg8(0), 0x12); // AL
    assert_eq!(harness.cpu.ip, 2);

    // Execute MOV AH, 0x34
    harness.step();
    assert_eq!(harness.cpu.read_reg8(4), 0x34); // AH
    assert_eq!(harness.cpu.regs[0], 0x3412); // AX should be 0x3412
    assert_eq!(harness.cpu.ip, 4);
}

#[test]
fn test_mov_r16_r16() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1234; MOV BX, AX
    harness.load_program(&[0xB8, 0x34, 0x12, 0x8B, 0xD8], 0);

    // Execute MOV AX, 0x1234
    harness.step();
    assert_eq!(harness.cpu.regs[0], 0x1234); // AX

    // Execute MOV BX, AX (8B D8: MOV r16, r/m16 with ModR/M=D8)
    harness.step();
    assert_eq!(harness.cpu.regs[3], 0x1234); // BX
}

#[test]
fn test_xchg_ax_r16() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x1111; MOV CX, 0x2222; XCHG AX, CX
    harness.load_program(&[0xB8, 0x11, 0x11, 0xB9, 0x22, 0x22, 0x91], 0);

    harness.step(); // MOV AX, 0x1111
    harness.step(); // MOV CX, 0x2222
    harness.step(); // XCHG AX, CX

    assert_eq!(harness.cpu.regs[0], 0x2222); // AX
    assert_eq!(harness.cpu.regs[1], 0x1111); // CX
}

#[test]
fn test_lea_bx_si() {
    let mut harness = CpuHarness::new();
    // Set BX=0x1000, SI=0x0200
    // LEA AX, [BX+SI]
    harness.load_program(
        &[
            0xBB, 0x00, 0x10, // MOV BX, 0x1000
            0xBE, 0x00, 0x02, // MOV SI, 0x0200
            0x8D, 0x00, // LEA AX, [BX+SI] (ModR/M=00: AX, [BX+SI])
        ],
        0,
    );

    harness.step(); // MOV BX, 0x1000
    harness.step(); // MOV SI, 0x0200
    harness.step(); // LEA AX, [BX+SI]

    // LEA should calculate BX+SI = 0x1000+0x0200 = 0x1200
    assert_eq!(harness.cpu.regs[0], 0x1200); // AX
}

#[test]
fn test_mov_sreg_to_reg() {
    let mut harness = CpuHarness::new();
    // Set DS to 0x1234, then MOV AX, DS
    harness.cpu.segments[3] = 0x1234; // DS
    harness.load_program(
        &[
            0x8C, 0xD8, // MOV AX, DS (ModR/M=D8: reg=DS(011), rm=AX(000), mod=11)
        ],
        0,
    );

    harness.step(); // MOV AX, DS
    assert_eq!(harness.cpu.regs[0], 0x1234); // AX should contain DS value
    assert_eq!(harness.cpu.ip, 2);
}

#[test]
fn test_mov_reg_to_sreg() {
    let mut harness = CpuHarness::new();
    // MOV AX, 0x5678; MOV DS, AX
    harness.load_program(
        &[
            0xB8, 0x78, 0x56, // MOV AX, 0x5678
            0x8E, 0xD8, // MOV DS, AX (ModR/M=D8: reg=DS(011), rm=AX(000), mod=11)
        ],
        0,
    );

    harness.step(); // MOV AX, 0x5678
    assert_eq!(harness.cpu.regs[0], 0x5678); // AX

    harness.step(); // MOV DS, AX
    assert_eq!(harness.cpu.segments[3], 0x5678); // DS should contain AX value
    assert_eq!(harness.cpu.ip, 5);
}

#[test]
fn test_mov_sreg_es_to_reg() {
    let mut harness = CpuHarness::new();
    // Set ES to 0xABCD, then MOV BX, ES
    harness.cpu.segments[0] = 0xABCD; // ES
    harness.load_program(
        &[
            0x8C, 0xC3, // MOV BX, ES (ModR/M=C3: reg=ES(000), rm=BX(011), mod=11)
        ],
        0,
    );

    harness.step(); // MOV BX, ES
    assert_eq!(harness.cpu.regs[3], 0xABCD); // BX should contain ES value
}

#[test]
fn test_mov_sreg_ss_roundtrip() {
    let mut harness = CpuHarness::new();
    // MOV CX, 0x9999; MOV SS, CX; MOV DX, SS
    harness.load_program(
        &[
            0xB9, 0x99, 0x99, // MOV CX, 0x9999
            0x8E, 0xD1, // MOV SS, CX (ModR/M=D1: reg=SS(010), rm=CX(001), mod=11)
            0x8C, 0xD2, // MOV DX, SS (ModR/M=D2: reg=SS(010), rm=DX(010), mod=11)
        ],
        0,
    );

    harness.step(); // MOV CX, 0x9999
    assert_eq!(harness.cpu.regs[1], 0x9999); // CX

    harness.step(); // MOV SS, CX
    assert_eq!(harness.cpu.segments[2], 0x9999); // SS

    harness.step(); // MOV DX, SS
    assert_eq!(harness.cpu.regs[2], 0x9999); // DX should contain SS value
}

#[test]
fn test_lea_bx_si_disp8() {
    let mut harness = CpuHarness::new();
    // Set BX=0x1000, SI=0x0200
    // LEA DX, [BX+SI+0x50]
    harness.load_program(
        &[
            0xBB, 0x00, 0x10, // MOV BX, 0x1000
            0xBE, 0x00, 0x02, // MOV SI, 0x0200
            0x8D, 0x50, 0x50, // LEA DX, [BX+SI+0x50] (ModR/M=50: DX, [BX+SI+disp8])
        ],
        0,
    );

    harness.step(); // MOV BX, 0x1000
    harness.step(); // MOV SI, 0x0200
    harness.step(); // LEA DX, [BX+SI+0x50]

    // LEA should calculate BX+SI+0x50 = 0x1000+0x0200+0x50 = 0x1250
    assert_eq!(harness.cpu.regs[2], 0x1250); // DX
}

#[test]
fn test_lea_bx_si_disp16() {
    let mut harness = CpuHarness::new();
    // Set BX=0x1000, SI=0x0200
    // LEA CX, [BX+SI+0x1234]
    harness.load_program(
        &[
            0xBB, 0x00, 0x10, // MOV BX, 0x1000
            0xBE, 0x00, 0x02, // MOV SI, 0x0200
            0x8D, 0x88, 0x34, 0x12, // LEA CX, [BX+SI+0x1234] (ModR/M=88: CX, [BX+SI+disp16])
        ],
        0,
    );

    harness.step(); // MOV BX, 0x1000
    harness.step(); // MOV SI, 0x0200
    harness.step(); // LEA CX, [BX+SI+0x1234]

    // LEA should calculate BX+SI+0x1234 = 0x1000+0x0200+0x1234 = 0x2434
    assert_eq!(harness.cpu.regs[1], 0x2434); // CX
}

#[test]
fn test_lea_direct_address() {
    let mut harness = CpuHarness::new();
    // LEA AX, [0x5678]
    harness.load_program(
        &[
            0x8D, 0x06, 0x78, 0x56, // LEA AX, [0x5678] (ModR/M=06: AX, direct address)
        ],
        0,
    );

    harness.step(); // LEA AX, [0x5678]

    // LEA should load the address 0x5678 into AX
    assert_eq!(harness.cpu.regs[0], 0x5678); // AX
}

#[test]
fn test_lea_bp_di() {
    let mut harness = CpuHarness::new();
    // Set BP=0x2000, DI=0x0100
    // LEA BX, [BP+DI]
    harness.load_program(
        &[
            0xBD, 0x00, 0x20, // MOV BP, 0x2000
            0xBF, 0x00, 0x01, // MOV DI, 0x0100
            0x8D, 0x1B, // LEA BX, [BP+DI] (ModR/M=1B: BX, [BP+DI])
        ],
        0,
    );

    harness.step(); // MOV BP, 0x2000
    harness.step(); // MOV DI, 0x0100
    harness.step(); // LEA BX, [BP+DI]

    // LEA should calculate BP+DI = 0x2000+0x0100 = 0x2100
    assert_eq!(harness.cpu.regs[3], 0x2100); // BX
}

#[test]
fn test_lea_si_only() {
    let mut harness = CpuHarness::new();
    // Set SI=0x1234
    // LEA AX, [SI]
    harness.load_program(
        &[
            0xBE, 0x34, 0x12, // MOV SI, 0x1234
            0x8D, 0x04, // LEA AX, [SI] (ModR/M=04: AX, [SI])
        ],
        0,
    );

    harness.step(); // MOV SI, 0x1234
    harness.step(); // LEA AX, [SI]

    // LEA should load SI = 0x1234 into AX
    assert_eq!(harness.cpu.regs[0], 0x1234); // AX
}

#[test]
fn test_lea_pointer_arithmetic() {
    let mut harness = CpuHarness::new();
    // Common use case: calculate address of array element
    // Base address in BX=0x1000, index in SI=3, element size=2
    // LEA AX, [BX+SI*1+6] (simulating array[3] with 2-byte elements)
    // Since 8088 doesn't have scaled indexing, we pre-multiply: SI=6
    harness.load_program(
        &[
            0xBB, 0x00, 0x10, // MOV BX, 0x1000 (base address)
            0xBE, 0x06, 0x00, // MOV SI, 0x0006 (3 * 2 = 6)
            0x8D, 0x00, // LEA AX, [BX+SI]
        ],
        0,
    );

    harness.step(); // MOV BX, 0x1000
    harness.step(); // MOV SI, 0x0006
    harness.step(); // LEA AX, [BX+SI]

    // LEA should calculate 0x1000+0x0006 = 0x1006
    assert_eq!(harness.cpu.regs[0], 0x1006); // AX
}

#[test]
fn test_hlt() {
    let mut harness = CpuHarness::new();
    // HLT
    harness.load_program(&[0xF4], 0);

    // Initially, CPU is not halted
    assert_eq!(harness.cpu.halted, false);

    // Execute HLT
    harness.step();

    // CPU should now be halted
    assert_eq!(harness.cpu.halted, true);
    assert_eq!(harness.cpu.ip, 1); // IP should have advanced
}

#[test]
fn test_hlt_stays_halted() {
    let mut harness = CpuHarness::new();
    // HLT; NOP
    harness.load_program(&[0xF4, 0x90], 0);

    // Execute HLT
    harness.step();
    assert_eq!(harness.cpu.halted, true);
    assert_eq!(harness.cpu.ip, 1);

    // Step again - should stay halted and not execute NOP
    harness.step();
    assert_eq!(harness.cpu.halted, true);
    assert_eq!(harness.cpu.ip, 1); // IP should not have advanced to NOP
}

#[test]
fn test_hlt_interrupt_wakes() {
    let mut harness = CpuHarness::new();
    // Set up interrupt vector for IRQ0 (INT 0x08) at address 0x0020
    harness.mem.write_u16(0x08 * 4, 0x0100); // Offset: 0x0100
    harness.mem.write_u16(0x08 * 4 + 2, 0x0000); // Segment: 0x0000

    // Put an IRET at the interrupt handler (0x0000:0x0100)
    harness.mem.write_u8(0x0100, 0xCF); // IRET

    // Unmask IRQ0 in the PIC (clear bit 0 in IMR)
    harness.mem.pic_mut().set_imr(0xFE); // All masked except IRQ0

    // Load program: STI; HLT; NOP
    harness.load_program(&[0xFB, 0xF4, 0x90], 0);

    // Execute STI (enable interrupts)
    harness.step();
    assert_eq!(harness.cpu.halted, false);

    // Execute HLT
    harness.step();
    assert_eq!(harness.cpu.halted, true);

    // Trigger hardware interrupt from PIC (IRQ0)
    // Use edge-triggered mode: set IRQ0 from low to high
    harness.mem.pic_mut().set_irq_level(0, true);

    // Step - interrupt should wake CPU from halt and execute handler
    harness.step();

    // CPU should no longer be halted (interrupt cleared the halt flag)
    assert_eq!(harness.cpu.halted, false);

    // IP should be at interrupt handler (handler was entered)
    // After IRET from the interrupt handler, we'll be back at the instruction after HLT
}

#[test]
fn test_mov_memory_basic() {
    let mut harness = CpuHarness::new();

    // Set DS=0x0100 (to keep physical addresses within 64KB RAM)
    harness.cpu.segments[3] = 0x0100; // DS

    // Set BX to point to offset 0x0050
    harness.cpu.regs[3] = 0x0050; // BX

    // Write test value to DS:BX (physical address = 0x0100 * 16 + 0x0050 = 0x1050)
    let phys_addr = 0x1050;
    harness.mem.write_u8(phys_addr, 0xAA);

    // Verify write
    assert_eq!(
        harness.mem.read_u8(phys_addr),
        0xAA,
        "Memory write/read failed"
    );

    // Test: MOV AL, [BX]
    harness.load_program(&[0x8A, 0x07], 0); // MOV AL, [BX] (ModR/M = 0x07)
    harness.step();

    assert_eq!(
        harness.cpu.read_reg8(0),
        0xAA,
        "MOV AL, [BX] should read from DS:BX"
    );
}

#[test]
fn test_segment_override_mov() {
    let mut harness = CpuHarness::new();

    // Set up segments within 64KB address space
    // ES=0x0100 -> physical base 0x01000
    // DS=0x0200 -> physical base 0x02000
    harness.cpu.segments[0] = 0x0100; // ES
    harness.cpu.segments[3] = 0x0200; // DS

    // Set BX to point to offset 0x0050
    harness.cpu.regs[3] = 0x0050; // BX

    // Write test value to ES:0x0050 (physical address 0x01000 + 0x0050 = 0x01050)
    harness.mem.write_u8(0x01050, 0xAA);

    // Write different value to DS:0x0050 (physical address 0x02000 + 0x0050 = 0x02050)
    harness.mem.write_u8(0x02050, 0x55);

    // Test 1: MOV AL, [BX] without override (should use DS)
    harness.load_program(&[0x8A, 0x07], 0); // MOV AL, [BX]
    harness.step();
    assert_eq!(
        harness.cpu.read_reg8(0),
        0x55,
        "MOV without override should use DS"
    );

    // Test 2: ES: MOV AL, [BX] (should use ES)
    harness.load_program(&[0x26, 0x8A, 0x07], 0); // ES: MOV AL, [BX]
    harness.step();
    assert_eq!(
        harness.cpu.read_reg8(0),
        0xAA,
        "MOV with ES: override should use ES"
    );
}

#[test]
fn test_segment_override_mov_immediate() {
    let mut harness = CpuHarness::new();

    // Set up segments within 64KB address space
    // ES=0x0100 -> physical base 0x01000
    // DS=0x0200 -> physical base 0x02000
    harness.cpu.segments[0] = 0x0100; // ES
    harness.cpu.segments[3] = 0x0200; // DS

    // Set BX to point to offset 0x0050
    harness.cpu.regs[3] = 0x0050; // BX

    // Test: ES: MOV byte [BX], 0xA0
    harness.load_program(&[0x26, 0xC6, 0x07, 0xA0], 0); // ES: MOV [BX], 0xA0
    harness.step();

    // Value should be written to ES:0x0050 (physical address 0x01000 + 0x0050 = 0x01050)
    // Value should NOT be written to DS:0x0050 (physical address 0x02000 + 0x0050 = 0x02050)
    assert_eq!(
        harness.mem.read_u8(0x01050),
        0xA0,
        "Value should be written to ES:BX"
    );
    assert_eq!(
        harness.mem.read_u8(0x02050),
        0x00,
        "Value should NOT be written to DS:BX"
    );
}

#[test]
fn test_segment_override_mov_reg_to_mem() {
    let mut harness = CpuHarness::new();

    // Set up segments within 64KB address space
    // ES=0x0100 -> physical base 0x01000
    // DS=0x0200 -> physical base 0x02000
    harness.cpu.segments[0] = 0x0100; // ES
    harness.cpu.segments[3] = 0x0200; // DS

    // Set BX to point to offset 0x0050
    harness.cpu.regs[3] = 0x0050; // BX

    // Set AL to test value
    harness.cpu.write_reg8(0, 0xCC); // AL = 0xCC

    // Test: ES: MOV [BX], AL
    harness.load_program(&[0x26, 0x88, 0x07], 0); // ES: MOV [BX], AL
    harness.step();

    // Value should be written to ES:0x0050 (physical 0x01050), not DS:0x0050
    assert_eq!(
        harness.mem.read_u8(0x01050),
        0xCC,
        "Value should be written to ES:BX"
    );
    assert_eq!(
        harness.mem.read_u8(0x02050),
        0x00,
        "Value should NOT be written to DS:BX"
    );
}

#[test]
fn test_segment_override_with_displacement() {
    let mut harness = CpuHarness::new();

    // Set up segments
    // ES=0x0100 -> physical base 0x01000
    // DS=0x0200 -> physical base 0x02000
    harness.cpu.segments[0] = 0x0100; // ES
    harness.cpu.segments[3] = 0x0200; // DS

    // Set BX to base offset
    harness.cpu.regs[3] = 0x0100; // BX = 0x0100

    // Write test values
    // ES:BX+0x10 = ES:0x0110 -> physical 0x01110
    harness.mem.write_u8(0x01110, 0xBB);
    // DS:BX+0x10 = DS:0x0110 -> physical 0x02110
    harness.mem.write_u8(0x02110, 0xDD);

    // Test 1: MOV AL, [BX+0x10] without override (should use DS)
    harness.load_program(&[0x8A, 0x47, 0x10], 0); // MOV AL, [BX+0x10] (disp8=0x10)
    harness.step();
    assert_eq!(
        harness.cpu.read_reg8(0),
        0xDD,
        "Should read from DS:BX+disp"
    );

    // Test 2: ES: MOV AL, [BX+0x10] (should use ES)
    harness.load_program(&[0x26, 0x8A, 0x47, 0x10], 0); // ES: MOV AL, [BX+0x10]
    harness.step();
    assert_eq!(
        harness.cpu.read_reg8(0),
        0xBB,
        "Should read from ES:BX+disp"
    );
}

#[test]
fn test_segment_override_with_base_index() {
    let mut harness = CpuHarness::new();

    // Set up segments
    // SS=0x0100 -> physical base 0x01000
    // DS=0x0200 -> physical base 0x02000
    harness.cpu.segments[2] = 0x0100; // SS
    harness.cpu.segments[3] = 0x0200; // DS

    // Set BP and SI
    harness.cpu.regs[5] = 0x0100; // BP = 0x0100
    harness.cpu.regs[6] = 0x0050; // SI = 0x0050

    // Write test values
    // SS:BP+SI = SS:0x0150 -> physical 0x01150 (BP defaults to SS)
    harness.mem.write_u8(0x01150, 0xAA);
    // DS:BP+SI = DS:0x0150 -> physical 0x02150
    harness.mem.write_u8(0x02150, 0xBB);

    // Test 1: MOV AL, [BP+SI] without override (should use SS by default)
    harness.load_program(&[0x8A, 0x02], 0); // MOV AL, [BP+SI] (ModR/M=0x02)
    harness.step();
    assert_eq!(
        harness.cpu.read_reg8(0),
        0xAA,
        "BP-based addressing should default to SS"
    );

    // Test 2: DS: MOV AL, [BP+SI] (should override to DS)
    harness.load_program(&[0x3E, 0x8A, 0x02], 0); // DS: MOV AL, [BP+SI]
    harness.step();
    assert_eq!(
        harness.cpu.read_reg8(0),
        0xBB,
        "DS: override should work with BP+SI"
    );
}

#[test]
fn test_segment_override_word_operations() {
    let mut harness = CpuHarness::new();

    // Set up segments
    // ES=0x0100 -> physical base 0x01000
    // DS=0x0200 -> physical base 0x02000
    harness.cpu.segments[0] = 0x0100; // ES
    harness.cpu.segments[3] = 0x0200; // DS

    // Set BX
    harness.cpu.regs[3] = 0x0050; // BX

    // Write test values (little-endian)
    // ES:BX -> physical 0x01050
    harness.mem.write_u16(0x01050, 0x1234);
    // DS:BX -> physical 0x02050
    harness.mem.write_u16(0x02050, 0x5678);

    // Test: ES: MOV AX, [BX] (16-bit read with segment override)
    harness.load_program(&[0x26, 0x8B, 0x07], 0); // ES: MOV AX, [BX]
    harness.step();
    assert_eq!(
        harness.cpu.regs[0], 0x1234,
        "16-bit read should use ES override"
    );
}

#[test]
fn test_mov_al_moffs() {
    let mut harness = CpuHarness::new();
    // Set up memory at address 0x1234 with a test value
    harness.mem.write_u8(0x1234, 0xAB);

    // MOV AL, [0x1234] - opcode 0xA0 followed by 16-bit offset
    harness.load_program(&[0xA0, 0x34, 0x12], 0);

    harness.step();
    assert_eq!(harness.cpu.read_reg8(0), 0xAB, "AL should be 0xAB"); // AL
    assert_eq!(harness.cpu.ip, 3, "IP should advance by 3");
}

#[test]
fn test_mov_ax_moffs() {
    let mut harness = CpuHarness::new();
    // Set up memory at address 0x5678 with a test value
    harness.mem.write_u16(0x5678, 0xCDEF);

    // MOV AX, [0x5678] - opcode 0xA1 followed by 16-bit offset
    harness.load_program(&[0xA1, 0x78, 0x56], 0);

    harness.step();
    assert_eq!(harness.cpu.regs[0], 0xCDEF, "AX should be 0xCDEF"); // AX
    assert_eq!(harness.cpu.ip, 3, "IP should advance by 3");
}

#[test]
fn test_mov_moffs_al() {
    let mut harness = CpuHarness::new();
    // Set AL to a test value
    harness.cpu.write_reg8(0, 0x42); // AL = 0x42

    // MOV [0x2000], AL - opcode 0xA2 followed by 16-bit offset
    harness.load_program(&[0xA2, 0x00, 0x20], 0);

    harness.step();
    assert_eq!(
        harness.mem.read_u8(0x2000),
        0x42,
        "Memory at 0x2000 should be 0x42"
    );
    assert_eq!(harness.cpu.ip, 3, "IP should advance by 3");
}

#[test]
fn test_mov_moffs_ax() {
    let mut harness = CpuHarness::new();
    // Set AX to a test value
    harness.cpu.regs[0] = 0x9876; // AX = 0x9876

    // MOV [0x3000], AX - opcode 0xA3 followed by 16-bit offset
    harness.load_program(&[0xA3, 0x00, 0x30], 0);

    harness.step();
    assert_eq!(
        harness.mem.read_u16(0x3000),
        0x9876,
        "Memory at 0x3000 should be 0x9876"
    );
    assert_eq!(harness.cpu.ip, 3, "IP should advance by 3");
}

#[test]
fn test_mov_moffs_with_segment_override() {
    let mut harness = CpuHarness::new();

    // Set up segments
    // ES=0x0100 -> physical base 0x01000
    // DS=0x0200 -> physical base 0x02000
    harness.cpu.segments[0] = 0x0100; // ES
    harness.cpu.segments[3] = 0x0200; // DS

    // Test 1: MOV AL, [0x0050] without override (should use DS)
    harness.mem.write_u8(0x02050, 0xAA); // DS:0x0050 -> physical 0x02050
    harness.load_program(&[0xA0, 0x50, 0x00], 0); // MOV AL, [0x0050]
    harness.step();
    assert_eq!(harness.cpu.read_reg8(0), 0xAA, "Should read from DS:0x0050");

    // Test 2: ES: MOV AL, [0x0050] (should use ES)
    harness.mem.write_u8(0x01050, 0xBB); // ES:0x0050 -> physical 0x01050
    harness.load_program(&[0x26, 0xA0, 0x50, 0x00], 0); // ES: MOV AL, [0x0050]
    harness.step();
    assert_eq!(harness.cpu.read_reg8(0), 0xBB, "Should read from ES:0x0050");

    // Test 3: ES: MOV [0x0060], AX (write with segment override)
    harness.cpu.regs[0] = 0x1234; // AX = 0x1234
    harness.load_program(&[0x26, 0xA3, 0x60, 0x00], 0); // ES: MOV [0x0060], AX
    harness.step();
    assert_eq!(
        harness.mem.read_u16(0x01060),
        0x1234,
        "Should write to ES:0x0060"
    );
    assert_eq!(
        harness.mem.read_u16(0x02060),
        0x0000,
        "Should not write to DS:0x0060"
    );
}

#[test]
fn test_cbw_positive() {
    let mut harness = CpuHarness::new();
    // Set AL to a positive value (0x7F = 127)
    // CBW should extend to 0x007F
    harness.load_program(
        &[
            0xB0, 0x7F, // MOV AL, 0x7F
            0x98, // CBW
        ],
        0,
    );

    harness.step(); // MOV AL, 0x7F
    assert_eq!(harness.cpu.read_reg8(0), 0x7F); // AL = 0x7F

    harness.step(); // CBW
    assert_eq!(harness.cpu.read_reg8(4), 0x00); // AH should be 0x00
    assert_eq!(harness.cpu.regs[0], 0x007F); // AX = 0x007F
}

#[test]
fn test_cbw_negative() {
    let mut harness = CpuHarness::new();
    // Set AL to a negative value (0x80 = -128)
    // CBW should extend to 0xFF80
    harness.load_program(
        &[
            0xB0, 0x80, // MOV AL, 0x80
            0x98, // CBW
        ],
        0,
    );

    harness.step(); // MOV AL, 0x80
    assert_eq!(harness.cpu.read_reg8(0), 0x80); // AL = 0x80

    harness.step(); // CBW
    assert_eq!(harness.cpu.read_reg8(4), 0xFF); // AH should be 0xFF
    assert_eq!(harness.cpu.regs[0], 0xFF80); // AX = 0xFF80
}

#[test]
fn test_cbw_zero() {
    let mut harness = CpuHarness::new();
    // Set AL to zero
    // CBW should keep it as 0x0000
    harness.load_program(
        &[
            0xB0, 0x00, // MOV AL, 0x00
            0x98, // CBW
        ],
        0,
    );

    harness.step(); // MOV AL, 0x00
    harness.step(); // CBW
    assert_eq!(harness.cpu.regs[0], 0x0000); // AX = 0x0000
}

#[test]
fn test_cbw_boundary() {
    let mut harness = CpuHarness::new();
    // Test boundary value 0xFF (-1 in signed)
    // CBW should extend to 0xFFFF
    harness.load_program(
        &[
            0xB0, 0xFF, // MOV AL, 0xFF
            0x98, // CBW
        ],
        0,
    );

    harness.step(); // MOV AL, 0xFF
    harness.step(); // CBW
    assert_eq!(harness.cpu.regs[0], 0xFFFF); // AX = 0xFFFF
}

#[test]
fn test_cwd_positive() {
    let mut harness = CpuHarness::new();
    // Set AX to a positive value (0x7FFF = 32767)
    // CWD should extend to DX:AX = 0x0000:0x7FFF
    harness.load_program(
        &[
            0xB8, 0xFF, 0x7F, // MOV AX, 0x7FFF
            0x99, // CWD
        ],
        0,
    );

    harness.step(); // MOV AX, 0x7FFF
    assert_eq!(harness.cpu.regs[0], 0x7FFF); // AX = 0x7FFF

    harness.step(); // CWD
    assert_eq!(harness.cpu.regs[2], 0x0000); // DX should be 0x0000
}

#[test]
fn test_cwd_negative() {
    let mut harness = CpuHarness::new();
    // Set AX to a negative value (0x8000 = -32768)
    // CWD should extend to DX:AX = 0xFFFF:0x8000
    harness.load_program(
        &[
            0xB8, 0x00, 0x80, // MOV AX, 0x8000
            0x99, // CWD
        ],
        0,
    );

    harness.step(); // MOV AX, 0x8000
    assert_eq!(harness.cpu.regs[0], 0x8000); // AX = 0x8000

    harness.step(); // CWD
    assert_eq!(harness.cpu.regs[2], 0xFFFF); // DX should be 0xFFFF
}

#[test]
fn test_cwd_zero() {
    let mut harness = CpuHarness::new();
    // Set AX to zero
    // CWD should keep it as 0x0000:0x0000
    harness.load_program(
        &[
            0xB8, 0x00, 0x00, // MOV AX, 0x0000
            0x99, // CWD
        ],
        0,
    );

    harness.step(); // MOV AX, 0x0000
    harness.step(); // CWD
    assert_eq!(harness.cpu.regs[0], 0x0000); // AX = 0x0000
    assert_eq!(harness.cpu.regs[2], 0x0000); // DX = 0x0000
}

#[test]
fn test_cwd_boundary() {
    let mut harness = CpuHarness::new();
    // Test boundary value 0xFFFF (-1 in signed)
    // CWD should extend to 0xFFFF:0xFFFF
    harness.load_program(
        &[
            0xB8, 0xFF, 0xFF, // MOV AX, 0xFFFF
            0x99, // CWD
        ],
        0,
    );

    harness.step(); // MOV AX, 0xFFFF
    harness.step(); // CWD
    assert_eq!(harness.cpu.regs[0], 0xFFFF); // AX = 0xFFFF
    assert_eq!(harness.cpu.regs[2], 0xFFFF); // DX = 0xFFFF
}

#[test]
fn test_cbw_cwd_sequence() {
    let mut harness = CpuHarness::new();
    // Test CBW followed by CWD: convert 8-bit to 32-bit
    // AL = 0x80 (-128) -> AX = 0xFF80 -> DX:AX = 0xFFFF:0xFF80
    harness.load_program(
        &[
            0xB0, 0x80, // MOV AL, 0x80
            0x98, // CBW
            0x99, // CWD
        ],
        0,
    );

    harness.step(); // MOV AL, 0x80
    assert_eq!(harness.cpu.read_reg8(0), 0x80); // AL = 0x80

    harness.step(); // CBW
    assert_eq!(harness.cpu.regs[0], 0xFF80); // AX = 0xFF80

    harness.step(); // CWD
    assert_eq!(harness.cpu.regs[0], 0xFF80); // AX = 0xFF80 (unchanged)
    assert_eq!(harness.cpu.regs[2], 0xFFFF); // DX = 0xFFFF
}
