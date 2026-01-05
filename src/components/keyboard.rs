//! IBM PC Keyboard Controller Emulation
//!
//! The keyboard controller interfaces with the keyboard and generates IRQ1
//! when scancodes are available. Scancodes are fed from an external queue.

use crate::components::pic::Pic;
use crate::io::IoDevice;
use std::collections::VecDeque;
use std::ops::RangeInclusive;
use std::sync::{Arc, RwLock};

/// Keyboard I/O ports
const KEYBOARD_DATA_PORT: u16 = 0x60;
const SYSTEM_CONTROL_PORT_B: u16 = 0x62; // Motherboard control/status
const KEYBOARD_STATUS_PORT: u16 = 0x64;

/// Status register bits
const STATUS_OUTPUT_BUFFER_FULL: u8 = 0x01; // Data available to read

/// DIP switch configuration: 64K RAM, MDA display, no floppy, no 8087
/// Bits 7-6: Number of floppy drives (00 = 1)
/// Bits 5-4: Video mode (11 = MDA 80x25)
/// Bits 3-2: RAM size (11 = 64K)
/// Bit 1: 8087 installed (0 = no)
/// Bit 0: Floppy installed (1 = no)
const DIP_SWITCHES: u8 = 0b00111101; // 0x3D

/// IBM PC Keyboard Controller (8255 Peripheral Chip)
///
/// Receives scancodes from a shared queue and raises IRQ1 when data is available.
/// Port 0x60 returns either DIP switches or keyboard data based on port 0x61 bit 5.
pub struct Keyboard {
    /// Shared queue of keyboard scancodes
    scancode_queue: Arc<RwLock<VecDeque<u8>>>,

    /// Current scancode ready to be read (if any)
    current_scancode: Option<u8>,

    /// Current IRQ line level (true = high/asserted)
    irq_level: bool,

    /// Value latched to be read from port 0x60 (DIP switches or keyboard data)
    latched_data: u8,

    /// Last value written to port 0x61 (System Control Port B)
    port_61_state: u8,
}

impl Keyboard {
    /// Create a new keyboard controller
    ///
    /// # Arguments
    /// * `scancode_queue` - Shared queue for receiving keyboard scancodes
    pub fn new(scancode_queue: Arc<RwLock<VecDeque<u8>>>) -> Self {
        Self {
            scancode_queue,
            current_scancode: None,
            irq_level: false,
            latched_data: DIP_SWITCHES, // Default to DIP switches
            port_61_state: 0x00,
        }
    }

    /// Check if data is available to read
    fn has_data(&self) -> bool {
        self.current_scancode.is_some()
    }

    /// Get the status register value
    fn get_status(&self) -> u8 {
        let mut status = 0u8;
        if self.has_data() {
            status |= STATUS_OUTPUT_BUFFER_FULL;
        }
        status
    }
}

impl IoDevice for Keyboard {
    fn port_range(&self) -> RangeInclusive<u16> {
        KEYBOARD_DATA_PORT..=KEYBOARD_STATUS_PORT
    }

    fn read_u8(&mut self, port: u16) -> u8 {
        match port {
            KEYBOARD_DATA_PORT => {
                // Return latched data (either DIP switches or keyboard scancode)
                self.latched_data
            }
            SYSTEM_CONTROL_PORT_B => {
                // System Control Port B - motherboard status
                // Bit 7: Parity check occurred
                // Bit 6: Channel check occurred
                // Bit 5: Timer 2 output condition
                // Bit 4: Toggles with each refresh request
                // Bit 3: Channel check status
                // Bit 2: Parity check status
                // Bit 1: Speaker data status
                // Bit 0: Timer 2 gate to speaker status
                0x00
            }
            KEYBOARD_STATUS_PORT => {
                // Return status register
                self.get_status()
            }
            _ => 0xFF,
        }
    }

    fn write_u8(&mut self, port: u16, value: u8) {
        match port {
            SYSTEM_CONTROL_PORT_B => {
                self.port_61_state = value;

                // Bit 7 selects DIP switches (1) or keyboard data (0)
                if value & 0x80 != 0 {
                    // Latch DIP switches
                    self.latched_data = DIP_SWITCHES;
                } else {
                    // Latch keyboard scancode if available
                    if let Some(scancode) = self.current_scancode.take() {
                        self.latched_data = scancode;
                    }
                    // If no scancode available, latched_data remains unchanged
                }
            }
            KEYBOARD_STATUS_PORT => {
                // Command register - not implemented yet
                let _ = value;
            }
            _ => {
                // Ignore other writes
            }
        }
    }

    fn tick(&mut self, _cycles: u16, pic: &mut Pic) {
        // If we don't have a pending scancode, try to fetch one from the queue
        if self.current_scancode.is_none() {
            let scancode = {
                let mut queue = self.scancode_queue.write().unwrap();
                queue.pop_front()
            };

            if let Some(scancode) = scancode {
                // Store the scancode for reading
                self.current_scancode = Some(scancode);

                // If keyboard mode is enabled (bit 7 = 0), latch the scancode immediately
                if self.port_61_state & 0x80 == 0 {
                    self.latched_data = scancode;
                }

                // Raise IRQ1 if not already raised
                if !self.irq_level {
                    pic.set_irq_level(1, true); // IRQ1 for keyboard
                    self.irq_level = true;
                }
            } else if self.irq_level {
                // No data available and IRQ is still high, lower it
                pic.set_irq_level(1, false);
                self.irq_level = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_new() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let kbd = Keyboard::new(queue);
        assert!(!kbd.has_data());
        assert_eq!(kbd.get_status(), 0);
    }

    #[test]
    fn test_keyboard_port_range() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let kbd = Keyboard::new(queue);
        let range = kbd.port_range();
        assert_eq!(*range.start(), 0x60);
        assert_eq!(*range.end(), 0x64);
    }

    #[test]
    fn test_keyboard_status_no_data() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue);
        assert_eq!(kbd.read_u8(KEYBOARD_STATUS_PORT), 0);
    }

    #[test]
    fn test_keyboard_read_with_no_data() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue);
        // Port 0x60 now returns DIP switches by default (not 0xFF)
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), DIP_SWITCHES);
    }

    #[test]
    fn test_keyboard_tick_raises_irq() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue.clone());
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00); // Unmask all interrupts

        // Add scancode to queue
        queue.write().unwrap().push_back(0x1E); // 'A' key make code

        // Tick should detect the scancode and raise IRQ1
        kbd.tick(1, &mut pic);

        assert!(kbd.has_data());
        assert_eq!(kbd.get_status(), STATUS_OUTPUT_BUFFER_FULL);
        assert!(pic.intr_out()); // IRQ1 should be pending
    }

    #[test]
    fn test_keyboard_read_scancode() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue.clone());
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Add and process scancode
        queue.write().unwrap().push_back(0x1E);
        kbd.tick(1, &mut pic);

        // Latch keyboard data by writing to port 0x61 with bit 7 = 0
        kbd.write_u8(SYSTEM_CONTROL_PORT_B, 0x00);

        // Read the scancode
        let scancode = kbd.read_u8(KEYBOARD_DATA_PORT);
        assert_eq!(scancode, 0x1E);

        // current_scancode should be consumed by the latch operation
        assert!(!kbd.has_data());
    }

    #[test]
    fn test_keyboard_irq_lowered_after_read() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue.clone());
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Add scancode and raise IRQ
        queue.write().unwrap().push_back(0x1E);
        kbd.tick(1, &mut pic);
        assert!(pic.intr_out());

        // Read scancode
        let _scancode = kbd.read_u8(KEYBOARD_DATA_PORT);

        // Acknowledge the interrupt (CPU would do INTA and EOI)
        let _vector = pic.inta();
        pic.eoi();

        // Next tick should lower IRQ line (no more data)
        kbd.tick(1, &mut pic);
        assert!(!pic.intr_out()); // No pending interrupts
    }

    #[test]
    fn test_keyboard_multiple_scancodes() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue.clone());
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Add multiple scancodes
        queue.write().unwrap().push_back(0x1E); // 'A' make
        queue.write().unwrap().push_back(0x9E); // 'A' break

        // First tick gets first scancode
        kbd.tick(1, &mut pic);
        kbd.write_u8(SYSTEM_CONTROL_PORT_B, 0x00); // Latch keyboard data (bit 7 = 0)
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), 0x1E);

        // Lower IRQ after read
        kbd.tick(1, &mut pic);

        // Second tick gets second scancode
        kbd.tick(1, &mut pic);
        kbd.write_u8(SYSTEM_CONTROL_PORT_B, 0x00); // Latch keyboard data (bit 7 = 0)
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), 0x9E);
    }

    #[test]
    fn test_system_control_port_b_no_parity_errors() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue);
        // Port 0x62 bits 7-6 should always be 0 (parity checking disabled)
        let value = kbd.read_u8(SYSTEM_CONTROL_PORT_B);
        assert_eq!(value & 0xC0, 0x00);
    }

    #[test]
    fn test_port_60_returns_dip_switches_by_default() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue);
        // Port 0x60 should return DIP switches by default
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), DIP_SWITCHES);
    }

    #[test]
    fn test_port_61_write_bit7_high_latches_dip_switches() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue);

        // Write to port 0x61 with bit 7 = 1 (latch DIP switches)
        kbd.write_u8(SYSTEM_CONTROL_PORT_B, 0x80);

        // Port 0x60 should return DIP switches
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), DIP_SWITCHES);
    }

    #[test]
    fn test_port_61_write_bit7_low_latches_keyboard_data() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue.clone());
        let mut pic = Pic::new(0x08);

        // Add scancode to queue and tick to make it available
        queue.write().unwrap().push_back(0x1E); // 'A' key
        kbd.tick(1, &mut pic);

        // Write to port 0x61 with bit 7 = 0 (latch keyboard data)
        kbd.write_u8(SYSTEM_CONTROL_PORT_B, 0x00);

        // Port 0x60 should now return the keyboard scancode
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), 0x1E);
    }

    #[test]
    fn test_port_61_read_returns_status() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue);

        // Port 0x61 reads return status bits, not written values
        assert_eq!(kbd.read_u8(SYSTEM_CONTROL_PORT_B), 0x00);

        // Write a value (to control latching behavior)
        kbd.write_u8(SYSTEM_CONTROL_PORT_B, 0xAB);

        // Read still returns status (0x00), not the written value
        assert_eq!(kbd.read_u8(SYSTEM_CONTROL_PORT_B), 0x00);
    }

    #[test]
    fn test_latched_data_persists_across_reads() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue.clone());
        let mut pic = Pic::new(0x08);

        // Add scancode and latch it
        queue.write().unwrap().push_back(0x1E);
        kbd.tick(1, &mut pic);
        kbd.write_u8(SYSTEM_CONTROL_PORT_B, 0x00); // Latch keyboard data (bit 7 = 0)

        // Multiple reads should return the same latched value
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), 0x1E);
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), 0x1E);
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), 0x1E);
    }

    #[test]
    fn test_switching_between_dip_and_keyboard_data() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue.clone());
        let mut pic = Pic::new(0x08);

        // Add scancode and make it available
        queue.write().unwrap().push_back(0x9E);
        kbd.tick(1, &mut pic);

        // Latch keyboard data (bit 7 = 0)
        kbd.write_u8(SYSTEM_CONTROL_PORT_B, 0x00);
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), 0x9E);

        // Switch to DIP switches (bit 7 = 1)
        kbd.write_u8(SYSTEM_CONTROL_PORT_B, 0x80);
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), DIP_SWITCHES);

        // Should still be DIP switches on subsequent reads
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), DIP_SWITCHES);
    }

    #[test]
    fn test_latch_keyboard_with_no_data_available() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue);

        // Start with DIP switches by setting bit 7 = 1
        kbd.write_u8(SYSTEM_CONTROL_PORT_B, 0x80);
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), DIP_SWITCHES);

        // Try to latch keyboard data when none is available (bit 7 = 0)
        kbd.write_u8(SYSTEM_CONTROL_PORT_B, 0x00);

        // Should still have DIP switches (latched_data unchanged, no scancode to latch)
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), DIP_SWITCHES);
    }

    #[test]
    fn test_keyboard_mode_enabled_before_scancode_arrives() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue.clone());
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Enable keyboard mode BEFORE scancode arrives (bit 7 = 0)
        // Note: port_61_state already defaults to 0x00, but we write explicitly here
        kbd.write_u8(SYSTEM_CONTROL_PORT_B, 0x00);

        // At this point, no scancode available, so still DIP switches
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), DIP_SWITCHES);

        // Now add a scancode to the queue
        queue.write().unwrap().push_back(0x1E);

        // Tick should fetch the scancode AND auto-latch it because bit 7 = 0
        kbd.tick(1, &mut pic);

        // Should now read the keyboard scancode without needing another write to port 0x61
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), 0x1E);
    }
}
