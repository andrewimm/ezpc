//! Memory bus emulation
//!
//! The IBM PC memory layout:
//! - 0x00000-0x9FFFF: RAM (up to 640KB, we start with 64KB)
//! - 0xA0000-0xBFFFF: Video memory (not implemented yet)
//! - 0xC0000-0xFFFFF: ROM and BIOS

use crate::components::mda::Mda;
use crate::components::pic::Pic;
use crate::io::IoDevice;

/// PIC I/O ports (hardwired for performance)
const PIC_PORT_BASE: u16 = 0x20;
const PIC_PORT_END: u16 = 0x21;

/// MDA memory range (hardwired for performance)
const MDA_VRAM_BASE: u32 = 0xB0000;
const MDA_VRAM_END: u32 = 0xB0FFF;

/// MDA I/O ports (hardwired for performance)
const MDA_PORT_BASE: u16 = 0x3B0;
const MDA_PORT_END: u16 = 0x3BF;

/// Memory bus for the IBM PC
pub struct MemoryBus {
    /// RAM - starting with 64KB
    ram: [u8; 65536],

    /// ROM - BIOS and extension ROMs (64KB space)
    rom: [u8; 65536],

    /// 8259 PIC (Programmable Interrupt Controller)
    /// Hardwired at ports 0x20-0x21 for performance
    pic: Pic,

    /// MDA (Monochrome Display Adapter)
    /// Hardwired at 0xB0000-0xB0FFF (video RAM) and 0x3B0-0x3BF (ports)
    mda: Mda,

    /// Registered IO devices for IN/OUT instructions
    io_devices: Vec<Box<dyn IoDevice>>,
}

impl MemoryBus {
    /// Create a new memory bus with zeroed RAM
    pub fn new() -> Self {
        Self {
            ram: [0; 65536],
            rom: [0; 65536],
            pic: Pic::new(0x08), // IRQ0-7 map to INT 0x08-0x0F
            mda: Mda::new(),
            io_devices: Vec::new(),
        }
    }

    /// Load ROM data at the end of ROM space
    ///
    /// The ROM is loaded at the end of the 64KB ROM area (0xF0000-0xFFFFF),
    /// ensuring the reset vector at 0xFFFF0 contains the ROM's code.
    pub fn load_rom(&mut self, rom_data: &[u8]) {
        if rom_data.is_empty() {
            return;
        }

        if rom_data.len() > self.rom.len() {
            panic!(
                "ROM size {} bytes exceeds ROM space of {} bytes",
                rom_data.len(),
                self.rom.len()
            );
        }

        // Load ROM at the end of ROM space
        let offset = self.rom.len() - rom_data.len();
        self.rom[offset..].copy_from_slice(rom_data);
    }

    /// Read a byte from memory
    #[inline(always)]
    pub fn read_u8(&self, addr: u32) -> u8 {
        if addr < 0x10000 {
            // RAM (first 64KB)
            self.ram[addr as usize]
        } else if addr >= MDA_VRAM_BASE && addr <= MDA_VRAM_END {
            // MDA video RAM (0xB0000-0xB0FFF)
            let offset = (addr - MDA_VRAM_BASE) as u16;
            self.mda.read_vram(offset)
        } else if addr >= 0xF0000 {
            // ROM/BIOS area (last 64KB)
            self.rom[(addr - 0xF0000) as usize]
        } else {
            // Unmapped memory returns 0xFF
            0xFF
        }
    }

    /// Write a byte to memory
    #[inline(always)]
    pub fn write_u8(&mut self, addr: u32, value: u8) {
        if addr < 0x10000 {
            // RAM (first 64KB)
            self.ram[addr as usize] = value;
        } else if addr >= MDA_VRAM_BASE && addr <= MDA_VRAM_END {
            // MDA video RAM (0xB0000-0xB0FFF)
            let offset = (addr - MDA_VRAM_BASE) as u16;
            self.mda.write_vram(offset, value);
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

    /// Get a reference to the PIC (Programmable Interrupt Controller)
    pub fn pic(&self) -> &Pic {
        &self.pic
    }

    /// Get a mutable reference to the PIC (Programmable Interrupt Controller)
    pub fn pic_mut(&mut self) -> &mut Pic {
        &mut self.pic
    }

    /// Get a reference to the MDA (Monochrome Display Adapter)
    pub fn mda(&self) -> &Mda {
        &self.mda
    }

    /// Get a mutable reference to the MDA (Monochrome Display Adapter)
    pub fn mda_mut(&mut self) -> &mut Mda {
        &mut self.mda
    }

    /// Read a byte from an IO port
    #[inline(always)]
    pub fn io_read_u8(&mut self, port: u16) -> u8 {
        // PIC is hardwired for performance
        if port >= PIC_PORT_BASE && port <= PIC_PORT_END {
            return self.pic.read_u8(port);
        }

        // MDA is hardwired for performance
        if port >= MDA_PORT_BASE && port <= MDA_PORT_END {
            return self.mda.read_u8(port);
        }

        // Check other IO devices
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
        // PIC is hardwired for performance
        if port >= PIC_PORT_BASE && port <= PIC_PORT_END {
            self.pic.write_u8(port, value);
            return;
        }

        // MDA is hardwired for performance
        if port >= MDA_PORT_BASE && port <= MDA_PORT_END {
            self.mda.write_u8(port, value);
            return;
        }

        // Check other IO devices
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

    /// Update peripherals based on CPU cycles
    ///
    /// Called after each CPU instruction with the number of cycles consumed.
    /// Distributes the tick to MDA and all registered IO devices.
    pub fn tick(&mut self, cycles: u16) {
        // Update MDA
        self.mda.tick(cycles);

        // Update all IO devices
        for device in &mut self.io_devices {
            device.tick(cycles);
        }
    }
}
