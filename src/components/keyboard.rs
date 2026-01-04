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
const KEYBOARD_STATUS_PORT: u16 = 0x64;

/// Status register bits
const STATUS_OUTPUT_BUFFER_FULL: u8 = 0x01; // Data available to read

/// IBM PC Keyboard Controller
///
/// Receives scancodes from a shared queue and raises IRQ1 when data is available.
pub struct Keyboard {
    /// Shared queue of keyboard scancodes
    scancode_queue: Arc<RwLock<VecDeque<u8>>>,

    /// Current scancode ready to be read (if any)
    current_scancode: Option<u8>,

    /// Current IRQ line level (true = high/asserted)
    irq_level: bool,
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
                // Read scancode and clear buffer
                if let Some(scancode) = self.current_scancode.take() {
                    scancode
                } else {
                    0xFF // No data available
                }
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
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), 0xFF);
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

        // Read the scancode
        let scancode = kbd.read_u8(KEYBOARD_DATA_PORT);
        assert_eq!(scancode, 0x1E);
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
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), 0x1E);

        // Lower IRQ after read
        kbd.tick(1, &mut pic);

        // Second tick gets second scancode
        kbd.tick(1, &mut pic);
        assert_eq!(kbd.read_u8(KEYBOARD_DATA_PORT), 0x9E);
    }
}
