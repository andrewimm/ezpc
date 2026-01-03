//! Memory bus emulation
//!
//! The IBM PC memory layout:
//! - 0x00000-0x9FFFF: RAM (up to 640KB, we start with 64KB)
//! - 0xA0000-0xBFFFF: Video memory (not implemented yet)
//! - 0xC0000-0xFFFFF: ROM and BIOS

use crate::io::IoDevice;

/// Memory bus for the IBM PC
pub struct MemoryBus {
    /// RAM - starting with 64KB
    ram: [u8; 65536],

    /// ROM - BIOS and extension ROMs (64KB space)
    rom: [u8; 65536],

    /// Registered IO devices for IN/OUT instructions
    io_devices: Vec<Box<dyn IoDevice>>,
}

impl MemoryBus {
    /// Create a new memory bus with zeroed RAM
    pub fn new() -> Self {
        Self {
            ram: [0; 65536],
            rom: [0; 65536],
            io_devices: Vec::new(),
        }
    }

    /// Read a byte from memory
    #[inline(always)]
    pub fn read_u8(&self, addr: u32) -> u8 {
        let addr = addr as usize;
        if addr < 0x10000 {
            // RAM (first 64KB)
            self.ram[addr]
        } else if addr >= 0xF0000 {
            // ROM/BIOS area (last 64KB)
            self.rom[addr - 0xF0000]
        } else {
            // Unmapped memory returns 0xFF
            0xFF
        }
    }

    /// Write a byte to memory
    #[inline(always)]
    pub fn write_u8(&mut self, addr: u32, value: u8) {
        let addr = addr as usize;
        if addr < 0x10000 {
            // RAM (first 64KB)
            self.ram[addr] = value;
        }
        // ROM writes are ignored
    }

    /// Read a word (little-endian) from memory
    #[inline(always)]
    pub fn read_u16(&self, addr: u32) -> u16 {
        let lo = self.read_u8(addr) as u16;
        let hi = self.read_u8(addr + 1) as u16;
        lo | (hi << 8)
    }

    /// Write a word (little-endian) to memory
    #[inline(always)]
    pub fn write_u16(&mut self, addr: u32, value: u16) {
        self.write_u8(addr, value as u8);
        self.write_u8(addr + 1, (value >> 8) as u8);
    }

    /// Load data into RAM at specified offset
    pub fn load(&mut self, data: &[u8], offset: usize) {
        let end = (offset + data.len()).min(self.ram.len());
        self.ram[offset..end].copy_from_slice(&data[..end - offset]);
    }

    /// Register an IO peripheral device
    pub fn register_io_device(&mut self, device: Box<dyn IoDevice>) {
        self.io_devices.push(device);
    }

    /// Read a byte from an IO port
    #[inline(always)]
    pub fn io_read_u8(&mut self, port: u16) -> u8 {
        for device in &mut self.io_devices {
            if device.port_range().contains(&port) {
                return device.read_u8(port);
            }
        }
        0xFF // Unmapped port returns 0xFF
    }

    /// Write a byte to an IO port
    #[inline(always)]
    pub fn io_write_u8(&mut self, port: u16, value: u8) {
        for device in &mut self.io_devices {
            if device.port_range().contains(&port) {
                device.write_u8(port, value);
                return;
            }
        }
        // Writes to unmapped ports are ignored
    }

    /// Read a word (little-endian) from an IO port
    #[inline(always)]
    pub fn io_read_u16(&mut self, port: u16) -> u16 {
        let lo = self.io_read_u8(port) as u16;
        let hi = self.io_read_u8(port + 1) as u16;
        lo | (hi << 8)
    }

    /// Write a word (little-endian) to an IO port
    #[inline(always)]
    pub fn io_write_u16(&mut self, port: u16, value: u16) {
        self.io_write_u8(port, value as u8);
        self.io_write_u8(port + 1, (value >> 8) as u8);
    }
}
