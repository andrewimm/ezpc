//! IO device support for peripherals
//!
//! The 8088 uses a separate IO address space (0x0000-0xFFFF) accessed via
//! IN/OUT instructions. Peripherals implement the IoDevice trait and register
//! with the MemoryBus.

use std::ops::RangeInclusive;

/// Trait for IO peripheral devices
pub trait IoDevice {
    /// Read a byte from a port within this device's range
    fn read_u8(&mut self, port: u16) -> u8;

    /// Write a byte to a port within this device's range
    fn write_u8(&mut self, port: u16, value: u8);

    /// Get the port range this device handles (inclusive)
    fn port_range(&self) -> RangeInclusive<u16>;

    /// Update device state based on CPU cycles
    ///
    /// Called after each CPU instruction with the number of cycles consumed.
    /// Devices can signal interrupts through the provided PIC reference.
    /// Default implementation does nothing - devices can override if needed.
    fn tick(&mut self, _cycles: u16, _pic: &mut crate::components::pic::Pic) {
        // Default: do nothing
    }
}
