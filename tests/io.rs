//! Tests for IO instructions (IN/OUT)

use ezpc::cpu::harness::CpuHarness;
use ezpc::io::IoDevice;
use std::ops::RangeInclusive;

/// Simple test device that echoes writes back on reads
struct TestDevice {
    ports: [u8; 256],
}

impl TestDevice {
    fn new() -> Self {
        Self { ports: [0; 256] }
    }
}

impl IoDevice for TestDevice {
    fn port_range(&self) -> RangeInclusive<u16> {
        0x00..=0xFF
    }

    fn read_u8(&mut self, port: u16) -> u8 {
        self.ports[port as usize]
    }

    fn write_u8(&mut self, port: u16, value: u8) {
        self.ports[port as usize] = value;
    }
}

#[test]
fn test_out_in_imm8() {
    let mut harness = CpuHarness::new();

    // Register test device
    let device = Box::new(TestDevice::new());
    harness.mem.register_io_device(device);

    // Program: MOV AL, 0x42; OUT 0x55, AL; MOV AL, 0x00; IN AL, 0x55
    harness.load_program(
        &[
            0xB0, 0x42, // MOV AL, 0x42
            0xE6, 0x55, // OUT 0x55, AL
            0xB0, 0x00, // MOV AL, 0x00 (clear AL)
            0xE4, 0x55, // IN AL, 0x55
        ],
        0,
    );

    // MOV AL, 0x42
    harness.step();
    assert_eq!(harness.cpu.read_reg8(0), 0x42);

    // OUT 0x55, AL
    harness.step();

    // MOV AL, 0x00
    harness.step();
    assert_eq!(harness.cpu.read_reg8(0), 0x00);

    // IN AL, 0x55
    harness.step();
    assert_eq!(harness.cpu.read_reg8(0), 0x42); // Should read back what we wrote
}

#[test]
fn test_out_in_imm16() {
    let mut harness = CpuHarness::new();

    // Register test device
    let device = Box::new(TestDevice::new());
    harness.mem.register_io_device(device);

    // Program: MOV AX, 0x1234; OUT 0x55, AX; MOV AX, 0; IN AX, 0x55
    harness.load_program(
        &[
            0xB8, 0x34, 0x12, // MOV AX, 0x1234
            0xE7, 0x55, // OUT 0x55, AX (ports 0x55-0x56)
            0xB8, 0x00, 0x00, // MOV AX, 0x0000
            0xE5, 0x55, // IN AX, 0x55
        ],
        0,
    );

    // MOV AX, 0x1234
    harness.step();
    assert_eq!(harness.cpu.read_reg16(0), 0x1234);

    // OUT 0x55, AX
    harness.step();

    // MOV AX, 0x0000
    harness.step();
    assert_eq!(harness.cpu.read_reg16(0), 0x0000);

    // IN AX, 0x55
    harness.step();
    assert_eq!(harness.cpu.read_reg16(0), 0x1234); // Should read back what we wrote
}

#[test]
fn test_mov_dx_simple() {
    let mut harness = CpuHarness::new();

    // Simple test: just MOV DX, imm16
    harness.load_program(&[0xBA, 0x55, 0x00], 0); // MOV DX, 0x0055

    harness.step();
    assert_eq!(harness.cpu.regs[2], 0x0055, "DX should be 0x0055");
}

#[test]
fn test_out_in_dx() {
    let mut harness = CpuHarness::new();

    // Register test device
    let device = Box::new(TestDevice::new());
    harness.mem.register_io_device(device);

    // Program: MOV DX, 0x55; MOV AL, 0x99; OUT DX, AL; MOV AL, 0; IN AL, DX
    harness.load_program(
        &[
            0xBA, 0x55, 0x00, // MOV DX, 0x0055
            0xB0, 0x99, // MOV AL, 0x99
            0xEE, // OUT DX, AL
            0xB0, 0x00, // MOV AL, 0x00
            0xEC, // IN AL, DX
        ],
        0,
    );

    // MOV DX, 0x0055
    harness.step();
    assert_eq!(harness.cpu.regs[2], 0x0055, "DX should be 0x0055 after MOV");

    // MOV AL, 0x99
    harness.step();
    assert_eq!(harness.cpu.read_reg8(0), 0x99);

    // OUT DX, AL
    harness.step();

    // MOV AL, 0x00
    harness.step();
    assert_eq!(harness.cpu.read_reg8(0), 0x00);

    // IN AL, DX
    harness.step();
    assert_eq!(harness.cpu.read_reg8(0), 0x99); // Should read back what we wrote
}

#[test]
fn test_out_in_dx_word() {
    let mut harness = CpuHarness::new();

    // Register test device
    let device = Box::new(TestDevice::new());
    harness.mem.register_io_device(device);

    // Program: MOV DX, 0x55; MOV AX, 0xABCD; OUT DX, AX; MOV AX, 0; IN AX, DX
    harness.load_program(
        &[
            0xBA, 0x55, 0x00, // MOV DX, 0x0055
            0xB8, 0xCD, 0xAB, // MOV AX, 0xABCD
            0xEF, // OUT DX, AX
            0xB8, 0x00, 0x00, // MOV AX, 0x0000
            0xED, // IN AX, DX
        ],
        0,
    );

    // MOV DX, 0x0055
    harness.step();
    assert_eq!(harness.cpu.regs[2], 0x0055);

    // MOV AX, 0xABCD
    harness.step();
    assert_eq!(harness.cpu.read_reg16(0), 0xABCD);

    // OUT DX, AX
    harness.step();

    // MOV AX, 0x0000
    harness.step();
    assert_eq!(harness.cpu.read_reg16(0), 0x0000);

    // IN AX, DX
    harness.step();
    assert_eq!(harness.cpu.read_reg16(0), 0xABCD); // Should read back what we wrote
}

#[test]
fn test_unmapped_port() {
    let mut harness = CpuHarness::new();

    // Register test device (ports 0x00-0xFF)
    let device = Box::new(TestDevice::new());
    harness.mem.register_io_device(device);

    // Program: IN AL, 0x20 (port 0x20 within device range, will be 0 initially)
    harness.load_program(&[0xE4, 0x20], 0);

    // IN AL, 0x20 (port is mapped, should return 0 initially)
    harness.step();
    assert_eq!(harness.cpu.read_reg8(0), 0x00);

    // Now test truly unmapped port - need a device with limited range
    let mut harness2 = CpuHarness::new();

    // Create device that only handles ports 0x20-0x2F
    struct LimitedDevice {
        data: u8,
    }

    impl IoDevice for LimitedDevice {
        fn port_range(&self) -> RangeInclusive<u16> {
            0x20..=0x2F
        }

        fn read_u8(&mut self, _port: u16) -> u8 {
            self.data
        }

        fn write_u8(&mut self, _port: u16, value: u8) {
            self.data = value;
        }
    }

    harness2
        .mem
        .register_io_device(Box::new(LimitedDevice { data: 0 }));

    // Program: IN AL, 0x50 (unmapped port)
    harness2.load_program(&[0xE4, 0x50], 0);

    // IN AL, 0x50 (unmapped port should return 0xFF)
    harness2.step();
    assert_eq!(harness2.cpu.read_reg8(0), 0xFF);
}

#[test]
fn test_write_to_unmapped_port() {
    let mut harness = CpuHarness::new();

    // Register test device with limited range (ports 0x20-0x2F)
    struct LimitedDevice {
        data: u8,
    }

    impl IoDevice for LimitedDevice {
        fn port_range(&self) -> RangeInclusive<u16> {
            0x20..=0x2F
        }

        fn read_u8(&mut self, _port: u16) -> u8 {
            self.data
        }

        fn write_u8(&mut self, _port: u16, value: u8) {
            self.data = value;
        }
    }

    harness
        .mem
        .register_io_device(Box::new(LimitedDevice { data: 0 }));

    // Program: MOV AL, 0x42; OUT 0x50, AL (unmapped port)
    harness.load_program(
        &[
            0xB0, 0x42, // MOV AL, 0x42
            0xE6, 0x50, // OUT 0x50, AL (unmapped port)
        ],
        0,
    );

    // MOV AL, 0x42
    harness.step();

    // OUT 0x50, AL (should not crash)
    harness.step();
    // No assertion - just ensuring it doesn't crash
}
