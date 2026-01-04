//! Intel 8237 DMA Controller (stub implementation)
//!
//! The IBM PC uses the 8237 DMA controller for direct memory access by peripherals.
//! This is a stub implementation that accepts BIOS initialization sequences without
//! actually implementing DMA transfers. Real DMA is not needed for basic emulation.

use crate::io::IoDevice;
use std::ops::RangeInclusive;

/// DMA page register ports
/// These extend the 16-bit addresses to 20-bit physical addresses
const DMA_PAGE_CH2: u16 = 0x81; // Channel 2 page register
const DMA_PAGE_CH3: u16 = 0x82; // Channel 3 page register
const DMA_PAGE_CH1: u16 = 0x83; // Channel 1 page register
const DMA_PAGE_CH0: u16 = 0x87; // Channel 0 page register

/// Intel 8237 DMA Controller stub
///
/// This implementation accepts reads and writes to DMA ports but does not
/// perform actual DMA transfers. The BIOS expects to be able to initialize
/// the DMA controller, so we track minimal state.
pub struct Dma {
    /// Channel address/count registers (8 bytes, 2 per channel)
    /// Channels 0-3, each has address (low/high) and count (low/high)
    channel_regs: [u8; 8],

    /// Command register (port 0x08)
    command: u8,

    /// Status/Request register (port 0x09)
    status: u8,

    /// Single channel mask register (port 0x0A)
    mask: u8,

    /// Mode register (port 0x0B)
    mode: u8,

    /// Flip-flop for 16-bit register access
    /// Address and count registers are 16-bit but accessed via 8-bit ports
    /// The flip-flop determines whether we're writing low or high byte
    flip_flop: bool,

    /// Page registers for extending addresses to 20-bit
    page_ch0: u8,
    page_ch1: u8,
    page_ch2: u8,
    page_ch3: u8,
}

impl Dma {
    /// Create a new DMA controller
    pub fn new() -> Self {
        Self {
            channel_regs: [0; 8],
            command: 0,
            status: 0,
            mask: 0x0F, // All channels masked by default
            mode: 0,
            flip_flop: false,
            page_ch0: 0,
            page_ch1: 0,
            page_ch2: 0,
            page_ch3: 0,
        }
    }
}

impl IoDevice for Dma {
    fn read_u8(&mut self, port: u16) -> u8 {
        match port {
            // Channel address/count registers (0x00-0x07)
            0x00..=0x07 => {
                let idx = port as usize;
                let value = self.channel_regs[idx];
                // Toggle flip-flop for 16-bit register access
                self.flip_flop = !self.flip_flop;
                value
            }
            // Status register (port 0x08)
            0x08 => {
                let value = self.status;
                value
            }
            // Request register (port 0x09) - write only, return 0xFF on read
            0x09 => 0xFF,
            // Mask register (port 0x0A)
            0x0A => self.mask,
            // Mode register (port 0x0B) - write only, return 0xFF on read
            0x0B => 0xFF,
            // Clear flip-flop (port 0x0C) - write only, return 0xFF on read
            0x0C => 0xFF,
            // Master clear (port 0x0D) - write only, return 0xFF on read
            0x0D => 0xFF,
            // Clear mask register (port 0x0E) - write only, return 0xFF on read
            0x0E => 0xFF,
            // Write all mask bits (port 0x0F) - write only, return 0xFF on read
            0x0F => 0xFF,
            // Page registers
            DMA_PAGE_CH0 => self.page_ch0,
            DMA_PAGE_CH1 => self.page_ch1,
            DMA_PAGE_CH2 => self.page_ch2,
            DMA_PAGE_CH3 => self.page_ch3,
            _ => 0xFF,
        }
    }

    fn write_u8(&mut self, port: u16, value: u8) {
        match port {
            // Channel address/count registers (0x00-0x07)
            0x00..=0x07 => {
                let idx = port as usize;
                self.channel_regs[idx] = value;
                // Toggle flip-flop for 16-bit register access
                self.flip_flop = !self.flip_flop;
            }
            // Command register (port 0x08)
            0x08 => {
                self.command = value;
            }
            // Request register (port 0x09)
            0x09 => {
                // Software DMA request - not implemented
            }
            // Mask register (port 0x0A)
            0x0A => {
                self.mask = value;
            }
            // Mode register (port 0x0B)
            0x0B => {
                self.mode = value;
            }
            // Clear flip-flop (port 0x0C)
            0x0C => {
                self.flip_flop = false;
            }
            // Master clear/reset (port 0x0D)
            0x0D => {
                // Reset DMA controller
                self.command = 0;
                self.status = 0;
                self.mask = 0x0F; // All channels masked
                self.mode = 0;
                self.flip_flop = false;
                self.channel_regs = [0; 8];
            }
            // Clear mask register (port 0x0E)
            0x0E => {
                self.mask = 0; // Unmask all channels
            }
            // Write all mask bits (port 0x0F)
            0x0F => {
                self.mask = value & 0x0F;
            }
            // Page registers
            DMA_PAGE_CH0 => self.page_ch0 = value,
            DMA_PAGE_CH1 => self.page_ch1 = value,
            DMA_PAGE_CH2 => self.page_ch2 = value,
            DMA_PAGE_CH3 => self.page_ch3 = value,
            _ => {}
        }
    }

    fn port_range(&self) -> RangeInclusive<u16> {
        // Return a range that's not used for actual routing
        // We'll handle this specially in memory.rs since the ports are non-contiguous
        0..=0
    }
}
