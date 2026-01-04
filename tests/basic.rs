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
