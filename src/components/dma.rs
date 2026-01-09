//! Intel 8237A DMA Controller
//!
//! The IBM PC uses the 8237A DMA controller for direct memory access by peripherals.
//! This implementation supports all standard 8237A features:
//! - 4 independent DMA channels
//! - Single, block, demand, and cascade transfer modes
//! - Auto-initialization for continuous transfers
//! - Address increment/decrement
//!
//! ## IBM PC Channel Assignments
//! - Channel 0: Memory refresh (DRAM refresh)
//! - Channel 1: User / Sound cards
//! - Channel 2: Floppy disk controller
//! - Channel 3: Hard disk controller (XT)
//!
//! ## Port Map
//! - 0x00-0x07: Channel address/count registers
//! - 0x08-0x0F: Control registers
//! - 0x81, 0x82, 0x83, 0x87: Page registers (extend to 20-bit addressing)

use crate::io::IoDevice;
use std::ops::RangeInclusive;

// =============================================================================
// Constants
// =============================================================================

/// DMA page register ports (extend 16-bit addresses to 20-bit)
pub const DMA_PAGE_CH0: u16 = 0x87;
pub const DMA_PAGE_CH1: u16 = 0x83;
pub const DMA_PAGE_CH2: u16 = 0x81;
pub const DMA_PAGE_CH3: u16 = 0x82;

// =============================================================================
// Enums
// =============================================================================

/// DMA transfer direction (from memory's perspective)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaDirection {
    /// Pseudo-transfer: address counting without data movement
    Verify = 0b00,
    /// Device → Memory (e.g., reading from floppy disk)
    Write = 0b01,
    /// Memory → Device (e.g., writing to floppy disk)
    Read = 0b10,
    /// Invalid/illegal transfer type
    Invalid = 0b11,
}

impl From<u8> for DmaDirection {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => DmaDirection::Verify,
            0b01 => DmaDirection::Write,
            0b10 => DmaDirection::Read,
            _ => DmaDirection::Invalid,
        }
    }
}

/// DMA transfer mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaTransferMode {
    /// Transfer while DREQ is asserted
    Demand = 0b00,
    /// One byte per DREQ pulse (used by FDC)
    Single = 0b01,
    /// Continuous transfer until count exhausted
    Block = 0b10,
    /// For cascading another DMA controller
    Cascade = 0b11,
}

impl From<u8> for DmaTransferMode {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => DmaTransferMode::Demand,
            0b01 => DmaTransferMode::Single,
            0b10 => DmaTransferMode::Block,
            _ => DmaTransferMode::Cascade,
        }
    }
}

// =============================================================================
// DmaChannel
// =============================================================================

/// Per-channel DMA state
#[derive(Debug, Clone)]
pub struct DmaChannel {
    /// Base address register (latched when programmed)
    base_address: u16,
    /// Base count register (latched when programmed)
    base_count: u16,
    /// Current address register (active during transfer)
    current_address: u16,
    /// Current count register (decrements during transfer)
    current_count: u16,
    /// Page register (extends address to 20-bit)
    page: u8,
    /// Mode register for this channel
    mode: u8,
    /// DREQ line state (set by device)
    dreq: bool,
    /// Channel is masked (disabled)
    masked: bool,
    /// Terminal count reached (latched until status read)
    terminal_count: bool,
}

impl DmaChannel {
    /// Create a new DMA channel in reset state
    pub fn new() -> Self {
        Self {
            base_address: 0,
            base_count: 0,
            current_address: 0,
            current_count: 0,
            page: 0,
            mode: 0,
            dreq: false,
            masked: true, // Channels start masked
            terminal_count: false,
        }
    }

    /// Reset channel to initial state
    pub fn reset(&mut self) {
        self.base_address = 0;
        self.base_count = 0;
        self.current_address = 0;
        self.current_count = 0;
        // Page register is NOT reset by master clear
        self.mode = 0;
        self.dreq = false;
        self.masked = true;
        self.terminal_count = false;
    }

    /// Get the transfer direction for this channel
    #[inline]
    pub fn direction(&self) -> DmaDirection {
        DmaDirection::from((self.mode >> 2) & 0b11)
    }

    /// Get the transfer mode for this channel
    #[inline]
    pub fn transfer_mode(&self) -> DmaTransferMode {
        DmaTransferMode::from((self.mode >> 6) & 0b11)
    }

    /// Returns true if address should increment, false if decrement
    #[inline]
    pub fn address_increment(&self) -> bool {
        (self.mode & 0b0010_0000) == 0
    }

    /// Returns true if auto-init is enabled
    #[inline]
    pub fn auto_init(&self) -> bool {
        (self.mode & 0b0001_0000) != 0
    }

    /// Get the 20-bit physical address for the current transfer
    #[inline]
    pub fn physical_address(&self) -> u32 {
        ((self.page as u32) << 16) | (self.current_address as u32)
    }

    /// Check if this channel is ready for transfer
    #[inline]
    pub fn is_active(&self) -> bool {
        self.dreq && !self.masked && self.direction() != DmaDirection::Invalid
    }

    /// Advance the transfer by one byte
    /// Returns true if terminal count is reached
    pub fn advance(&mut self) -> bool {
        // Update address
        if self.address_increment() {
            self.current_address = self.current_address.wrapping_add(1);
        } else {
            self.current_address = self.current_address.wrapping_sub(1);
        }

        // Decrement count (TC occurs when count underflows from 0)
        if self.current_count == 0 {
            self.terminal_count = true;

            // Auto-init: reload base values
            if self.auto_init() {
                self.current_address = self.base_address;
                self.current_count = self.base_count;
            }

            return true;
        }

        self.current_count = self.current_count.wrapping_sub(1);
        false
    }

    /// Set address register (called twice due to flip-flop)
    pub fn set_address_byte(&mut self, high: bool, value: u8) {
        if high {
            self.base_address = (self.base_address & 0x00FF) | ((value as u16) << 8);
            self.current_address = (self.current_address & 0x00FF) | ((value as u16) << 8);
        } else {
            self.base_address = (self.base_address & 0xFF00) | (value as u16);
            self.current_address = (self.current_address & 0xFF00) | (value as u16);
        }
    }

    /// Get address register byte (called twice due to flip-flop)
    pub fn get_address_byte(&self, high: bool) -> u8 {
        if high {
            (self.current_address >> 8) as u8
        } else {
            self.current_address as u8
        }
    }

    /// Set count register (called twice due to flip-flop)
    pub fn set_count_byte(&mut self, high: bool, value: u8) {
        if high {
            self.base_count = (self.base_count & 0x00FF) | ((value as u16) << 8);
            self.current_count = (self.current_count & 0x00FF) | ((value as u16) << 8);
        } else {
            self.base_count = (self.base_count & 0xFF00) | (value as u16);
            self.current_count = (self.current_count & 0xFF00) | (value as u16);
        }
    }

    /// Get count register byte (called twice due to flip-flop)
    pub fn get_count_byte(&self, high: bool) -> u8 {
        if high {
            (self.current_count >> 8) as u8
        } else {
            self.current_count as u8
        }
    }
}

impl Default for DmaChannel {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// DmaCapable Trait
// =============================================================================

/// Trait for devices that can perform DMA transfers
///
/// Devices like the FDC and sound cards implement this trait.
/// The MemoryBus uses this to coordinate transfers between
/// memory and the device.
pub trait DmaCapable {
    /// Check if device is requesting DMA transfer
    fn dma_dreq(&self) -> bool;

    /// Provide a byte for memory write (device → memory)
    /// Called when DMA direction is Write
    /// Returns None if no data available
    fn dma_read_byte(&mut self) -> Option<u8>;

    /// Receive a byte from memory read (memory → device)
    /// Called when DMA direction is Read
    fn dma_write_byte(&mut self, value: u8);

    /// Called when terminal count is reached
    fn dma_terminal_count(&mut self);
}

// =============================================================================
// Dma Controller
// =============================================================================

/// Intel 8237A DMA Controller
#[derive(Debug, Clone)]
pub struct Dma {
    /// Four DMA channels
    pub channels: [DmaChannel; 4],

    /// Command register (port 0x08 write)
    /// Bit 0: Memory-to-memory enable
    /// Bit 1: Channel 0 address hold enable
    /// Bit 2: Controller disable
    /// Bit 3: Compressed timing
    /// Bit 4: Rotating priority
    /// Bit 5: Extended write selection
    /// Bit 6: DREQ sense active low
    /// Bit 7: DACK sense active low
    command: u8,

    /// Flip-flop for 16-bit register access
    /// Address and count registers are 16-bit but accessed via 8-bit ports.
    /// The flip-flop toggles between low and high byte.
    flip_flop: bool,

    /// Temporary register (for memory-to-memory transfers)
    temp: u8,
}

impl Dma {
    /// Create a new DMA controller in reset state
    pub fn new() -> Self {
        Self {
            channels: [
                DmaChannel::new(),
                DmaChannel::new(),
                DmaChannel::new(),
                DmaChannel::new(),
            ],
            command: 0,
            flip_flop: false,
            temp: 0,
        }
    }

    /// Master reset - resets controller to initial state
    pub fn master_reset(&mut self) {
        for channel in &mut self.channels {
            channel.reset();
        }
        self.command = 0;
        self.flip_flop = false;
        self.temp = 0;
    }

    /// Check if the controller is disabled
    #[inline]
    pub fn is_disabled(&self) -> bool {
        (self.command & 0b0000_0100) != 0
    }

    /// Check if a channel is active and ready for transfer
    #[inline]
    pub fn is_channel_active(&self, channel: u8) -> bool {
        if self.is_disabled() {
            return false;
        }
        self.channels[channel as usize].is_active()
    }

    /// Set DREQ line state for a channel
    pub fn set_dreq(&mut self, channel: u8, active: bool) {
        self.channels[channel as usize].dreq = active;
    }

    /// Get DREQ line state for a channel
    #[inline]
    pub fn get_dreq(&self, channel: u8) -> bool {
        self.channels[channel as usize].dreq
    }

    /// Check if terminal count was reached for a channel
    #[inline]
    pub fn terminal_count(&self, channel: u8) -> bool {
        self.channels[channel as usize].terminal_count
    }

    /// Clear terminal count flag for a channel
    pub fn clear_terminal_count(&mut self, channel: u8) {
        self.channels[channel as usize].terminal_count = false;
    }

    /// Get the current physical address for a channel
    #[inline]
    pub fn current_address(&self, channel: u8) -> u32 {
        self.channels[channel as usize].physical_address()
    }

    /// Get the current count for a channel
    #[inline]
    pub fn current_count(&self, channel: u8) -> u16 {
        self.channels[channel as usize].current_count
    }

    /// Get the transfer direction for a channel
    #[inline]
    pub fn direction(&self, channel: u8) -> DmaDirection {
        self.channels[channel as usize].direction()
    }

    /// Get the transfer mode for a channel
    #[inline]
    pub fn transfer_mode(&self, channel: u8) -> DmaTransferMode {
        self.channels[channel as usize].transfer_mode()
    }

    /// Advance transfer for a channel by one byte
    /// Returns true if terminal count was reached
    pub fn advance(&mut self, channel: u8) -> bool {
        self.channels[channel as usize].advance()
    }

    /// Build status register value
    fn build_status(&self) -> u8 {
        let mut status = 0u8;

        // Bits 0-3: Terminal count status (channels 0-3)
        for (i, ch) in self.channels.iter().enumerate() {
            if ch.terminal_count {
                status |= 1 << i;
            }
        }

        // Bits 4-7: DREQ status (channels 0-3)
        for (i, ch) in self.channels.iter().enumerate() {
            if ch.dreq {
                status |= 1 << (i + 4);
            }
        }

        status
    }

    /// Handle write to mode register (port 0x0B)
    fn write_mode(&mut self, value: u8) {
        let channel = (value & 0b11) as usize;
        self.channels[channel].mode = value;
    }

    /// Handle write to single mask register (port 0x0A)
    fn write_single_mask(&mut self, value: u8) {
        let channel = (value & 0b11) as usize;
        let set_mask = (value & 0b100) != 0;
        self.channels[channel].masked = set_mask;
    }

    /// Handle write to request register (port 0x09)
    fn write_request(&mut self, value: u8) {
        let channel = (value & 0b11) as usize;
        let set_request = (value & 0b100) != 0;
        // Software DMA request - sets DREQ internally
        self.channels[channel].dreq = set_request;
    }

    /// Handle read/write to channel registers (ports 0x00-0x07)
    fn channel_register_read(&mut self, port: u16) -> u8 {
        let channel = (port >> 1) as usize;
        let is_count = (port & 1) != 0;
        let high = self.flip_flop;

        self.flip_flop = !self.flip_flop;

        if is_count {
            self.channels[channel].get_count_byte(high)
        } else {
            self.channels[channel].get_address_byte(high)
        }
    }

    fn channel_register_write(&mut self, port: u16, value: u8) {
        let channel = (port >> 1) as usize;
        let is_count = (port & 1) != 0;
        let high = self.flip_flop;

        self.flip_flop = !self.flip_flop;

        if is_count {
            self.channels[channel].set_count_byte(high, value);
        } else {
            self.channels[channel].set_address_byte(high, value);
        }
    }
}

impl Default for Dma {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// IoDevice Implementation
// =============================================================================

impl IoDevice for Dma {
    fn read_u8(&mut self, port: u16) -> u8 {
        match port {
            // Channel address/count registers (0x00-0x07)
            0x00..=0x07 => self.channel_register_read(port),

            // Status register (port 0x08)
            0x08 => {
                let status = self.build_status();
                // Reading status clears TC flags
                for ch in &mut self.channels {
                    ch.terminal_count = false;
                }
                status
            }

            // Request register (port 0x09) - write only
            0x09 => 0xFF,

            // Single mask register (port 0x0A) - can read mask bits
            0x0A => {
                let mut mask = 0u8;
                for (i, ch) in self.channels.iter().enumerate() {
                    if ch.masked {
                        mask |= 1 << i;
                    }
                }
                mask
            }

            // Mode register (port 0x0B) - write only
            0x0B => 0xFF,

            // Clear flip-flop (port 0x0C) - write only
            0x0C => 0xFF,

            // Temporary register (port 0x0D)
            0x0D => self.temp,

            // Clear mask register (port 0x0E) - write only
            0x0E => 0xFF,

            // Write all mask bits (port 0x0F) - write only
            0x0F => 0xFF,

            // Page registers
            DMA_PAGE_CH0 => self.channels[0].page,
            DMA_PAGE_CH1 => self.channels[1].page,
            DMA_PAGE_CH2 => self.channels[2].page,
            DMA_PAGE_CH3 => self.channels[3].page,

            _ => 0xFF,
        }
    }

    fn write_u8(&mut self, port: u16, value: u8) {
        match port {
            // Channel address/count registers (0x00-0x07)
            0x00..=0x07 => self.channel_register_write(port, value),

            // Command register (port 0x08)
            0x08 => {
                self.command = value;
            }

            // Request register (port 0x09)
            0x09 => self.write_request(value),

            // Single mask register (port 0x0A)
            0x0A => self.write_single_mask(value),

            // Mode register (port 0x0B)
            0x0B => self.write_mode(value),

            // Clear flip-flop (port 0x0C)
            0x0C => {
                self.flip_flop = false;
            }

            // Master clear/reset (port 0x0D)
            0x0D => {
                self.master_reset();
            }

            // Clear mask register (port 0x0E) - unmask all channels
            0x0E => {
                for ch in &mut self.channels {
                    ch.masked = false;
                }
            }

            // Write all mask bits (port 0x0F)
            0x0F => {
                for (i, ch) in self.channels.iter_mut().enumerate() {
                    ch.masked = (value & (1 << i)) != 0;
                }
            }

            // Page registers
            DMA_PAGE_CH0 => self.channels[0].page = value,
            DMA_PAGE_CH1 => self.channels[1].page = value,
            DMA_PAGE_CH2 => self.channels[2].page = value,
            DMA_PAGE_CH3 => self.channels[3].page = value,

            _ => {}
        }
    }

    fn port_range(&self) -> RangeInclusive<u16> {
        // DMA is hardwired in MemoryBus, not routed via this range
        0..=0
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dma_new() {
        let dma = Dma::new();
        assert_eq!(dma.command, 0);
        assert!(!dma.flip_flop);
        for ch in &dma.channels {
            assert!(ch.masked);
            assert!(!ch.dreq);
            assert_eq!(ch.current_address, 0);
            assert_eq!(ch.current_count, 0);
        }
    }

    #[test]
    fn test_master_reset() {
        let mut dma = Dma::new();

        // Set some state
        dma.command = 0xFF;
        dma.flip_flop = true;
        dma.channels[0].current_address = 0x1234;
        dma.channels[0].masked = false;
        dma.channels[0].page = 0x12;

        // Reset
        dma.master_reset();

        assert_eq!(dma.command, 0);
        assert!(!dma.flip_flop);
        assert_eq!(dma.channels[0].current_address, 0);
        assert!(dma.channels[0].masked);
        // Page register is NOT reset
        assert_eq!(dma.channels[0].page, 0x12);
    }

    #[test]
    fn test_mode_register_parsing() {
        let mut ch = DmaChannel::new();

        // Mode: Single transfer, Write (device→memory), increment, no auto-init
        // 01 0 0 01 10 = 0x44 + channel
        ch.mode = 0b0100_0100;
        assert_eq!(ch.direction(), DmaDirection::Write);
        assert_eq!(ch.transfer_mode(), DmaTransferMode::Single);
        assert!(ch.address_increment());
        assert!(!ch.auto_init());

        // Mode: Block transfer, Read (memory→device), decrement, auto-init
        // 10 1 1 10 00 = 0xA8 + channel
        ch.mode = 0b1011_1000;
        assert_eq!(ch.direction(), DmaDirection::Read);
        assert_eq!(ch.transfer_mode(), DmaTransferMode::Block);
        assert!(!ch.address_increment());
        assert!(ch.auto_init());
    }

    #[test]
    fn test_address_count_flip_flop() {
        let mut dma = Dma::new();

        // Clear flip-flop
        dma.write_u8(0x0C, 0);
        assert!(!dma.flip_flop);

        // Write address low byte to channel 0
        dma.write_u8(0x00, 0x34);
        assert!(dma.flip_flop);
        assert_eq!(dma.channels[0].current_address, 0x0034);
        assert_eq!(dma.channels[0].base_address, 0x0034);

        // Write address high byte to channel 0
        dma.write_u8(0x00, 0x12);
        assert!(!dma.flip_flop);
        assert_eq!(dma.channels[0].current_address, 0x1234);
        assert_eq!(dma.channels[0].base_address, 0x1234);

        // Write count low byte to channel 0
        dma.write_u8(0x01, 0xFF);
        assert!(dma.flip_flop);

        // Write count high byte to channel 0
        dma.write_u8(0x01, 0x00);
        assert!(!dma.flip_flop);
        assert_eq!(dma.channels[0].current_count, 0x00FF);
    }

    #[test]
    fn test_read_address_count() {
        let mut dma = Dma::new();

        // Set up channel 2
        dma.channels[2].current_address = 0xABCD;
        dma.channels[2].current_count = 0x1234;

        // Clear flip-flop
        dma.write_u8(0x0C, 0);

        // Read address (port 0x04 = channel 2 address)
        let low = dma.read_u8(0x04);
        assert_eq!(low, 0xCD);
        let high = dma.read_u8(0x04);
        assert_eq!(high, 0xAB);

        // Clear flip-flop
        dma.write_u8(0x0C, 0);

        // Read count (port 0x05 = channel 2 count)
        let low = dma.read_u8(0x05);
        assert_eq!(low, 0x34);
        let high = dma.read_u8(0x05);
        assert_eq!(high, 0x12);
    }

    #[test]
    fn test_page_registers() {
        let mut dma = Dma::new();

        dma.write_u8(DMA_PAGE_CH0, 0x01);
        dma.write_u8(DMA_PAGE_CH1, 0x02);
        dma.write_u8(DMA_PAGE_CH2, 0x03);
        dma.write_u8(DMA_PAGE_CH3, 0x04);

        assert_eq!(dma.read_u8(DMA_PAGE_CH0), 0x01);
        assert_eq!(dma.read_u8(DMA_PAGE_CH1), 0x02);
        assert_eq!(dma.read_u8(DMA_PAGE_CH2), 0x03);
        assert_eq!(dma.read_u8(DMA_PAGE_CH3), 0x04);

        assert_eq!(dma.channels[0].page, 0x01);
        assert_eq!(dma.channels[1].page, 0x02);
        assert_eq!(dma.channels[2].page, 0x03);
        assert_eq!(dma.channels[3].page, 0x04);
    }

    #[test]
    fn test_physical_address() {
        let mut ch = DmaChannel::new();
        ch.page = 0x12;
        ch.current_address = 0x3456;

        assert_eq!(ch.physical_address(), 0x12_3456);
    }

    #[test]
    fn test_mask_register() {
        let mut dma = Dma::new();

        // All channels start masked
        assert_eq!(dma.read_u8(0x0A), 0x0F);

        // Unmask channel 2: write 0x02 (bit 2 = 0 means unmask)
        dma.write_u8(0x0A, 0x02);
        assert!(!dma.channels[2].masked);
        assert_eq!(dma.read_u8(0x0A), 0x0B); // 1011 = all masked except ch2

        // Mask channel 2: write 0x06 (bit 2 = 1 means mask)
        dma.write_u8(0x0A, 0x06);
        assert!(dma.channels[2].masked);

        // Unmask all channels (port 0x0E)
        dma.write_u8(0x0E, 0);
        assert_eq!(dma.read_u8(0x0A), 0x00);

        // Mask channels 0 and 3 (port 0x0F)
        dma.write_u8(0x0F, 0x09);
        assert!(dma.channels[0].masked);
        assert!(!dma.channels[1].masked);
        assert!(!dma.channels[2].masked);
        assert!(dma.channels[3].masked);
    }

    #[test]
    fn test_mode_register() {
        let mut dma = Dma::new();

        // Set mode for channel 2: Single, Write, increment, no auto-init
        // 01 0 0 01 10 = 0x46
        dma.write_u8(0x0B, 0x46);
        assert_eq!(dma.channels[2].mode, 0x46);
        assert_eq!(dma.channels[2].direction(), DmaDirection::Write);
        assert_eq!(dma.channels[2].transfer_mode(), DmaTransferMode::Single);
    }

    #[test]
    fn test_status_register() {
        let mut dma = Dma::new();

        // Set TC on channels 0 and 2
        dma.channels[0].terminal_count = true;
        dma.channels[2].terminal_count = true;

        // Set DREQ on channel 1
        dma.channels[1].dreq = true;

        let status = dma.read_u8(0x08);
        assert_eq!(status & 0x0F, 0b0101); // TC bits: channels 0 and 2
        assert_eq!(status & 0xF0, 0b0010_0000); // DREQ bit: channel 1

        // Reading status clears TC flags
        assert!(!dma.channels[0].terminal_count);
        assert!(!dma.channels[2].terminal_count);
    }

    #[test]
    fn test_advance_with_increment() {
        let mut ch = DmaChannel::new();
        ch.current_address = 0x1000;
        ch.current_count = 2;
        ch.mode = 0b0100_0100; // Single, Write, increment

        // First advance: count 2 -> 1
        assert!(!ch.advance());
        assert_eq!(ch.current_address, 0x1001);
        assert_eq!(ch.current_count, 1);

        // Second advance: count 1 -> 0
        assert!(!ch.advance());
        assert_eq!(ch.current_address, 0x1002);
        assert_eq!(ch.current_count, 0);

        // Third advance: count 0 -> underflow = TC
        assert!(ch.advance());
        assert_eq!(ch.current_address, 0x1003);
        assert!(ch.terminal_count);
    }

    #[test]
    fn test_advance_with_decrement() {
        let mut ch = DmaChannel::new();
        ch.current_address = 0x1003;
        ch.current_count = 1;
        ch.mode = 0b0110_0100; // Single, Write, decrement

        // First advance: count 1 -> 0
        assert!(!ch.advance());
        assert_eq!(ch.current_address, 0x1002);
        assert_eq!(ch.current_count, 0);

        // Second advance: TC
        assert!(ch.advance());
        assert_eq!(ch.current_address, 0x1001);
        assert!(ch.terminal_count);
    }

    #[test]
    fn test_auto_init() {
        let mut ch = DmaChannel::new();
        ch.base_address = 0x2000;
        ch.base_count = 0x00FF;
        ch.current_address = 0x20FF;
        ch.current_count = 0;
        ch.mode = 0b0101_0100; // Single, Write, increment, AUTO-INIT

        // Advance to TC
        assert!(ch.advance());
        assert!(ch.terminal_count);

        // Verify auto-init reloaded base values
        assert_eq!(ch.current_address, 0x2000);
        assert_eq!(ch.current_count, 0x00FF);
    }

    #[test]
    fn test_is_channel_active() {
        let mut dma = Dma::new();

        // Channel not active: masked
        assert!(!dma.is_channel_active(2));

        // Unmask channel 2
        dma.channels[2].masked = false;
        assert!(!dma.is_channel_active(2)); // Still not active: no DREQ

        // Set DREQ
        dma.set_dreq(2, true);
        // Still need valid mode
        dma.channels[2].mode = 0b0100_0100; // Single, Write
        assert!(dma.is_channel_active(2));

        // Disable controller
        dma.command = 0b0000_0100;
        assert!(!dma.is_channel_active(2));
    }

    #[test]
    fn test_request_register() {
        let mut dma = Dma::new();

        // Software request for channel 1
        dma.write_u8(0x09, 0b0000_0101); // Set request for channel 1
        assert!(dma.channels[1].dreq);

        // Clear request for channel 1
        dma.write_u8(0x09, 0b0000_0001); // Clear request for channel 1
        assert!(!dma.channels[1].dreq);
    }
}
