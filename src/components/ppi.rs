//! Intel 8255 Programmable Peripheral Interface (PPI) Emulation
//!
//! The 8255 PPI handles:
//! - Port 0x60 (Port A): DIP switches or keyboard scancode
//! - Port 0x61 (Port B): System control (speaker, keyboard clock, etc.)
//! - Port 0x62 (Port C): System status bits

use crate::components::keyboard::Keyboard;
use crate::components::pic::Pic;
use crate::io::IoDevice;
use std::collections::VecDeque;
use std::ops::RangeInclusive;
use std::sync::{Arc, RwLock};

/// PPI I/O ports
const PPI_PORT_A: u16 = 0x60; // Data port (DIP switches or keyboard)
const PPI_PORT_B: u16 = 0x61; // System control port B
const PPI_PORT_C: u16 = 0x62; // System control port C

/// DIP switch configuration: 64K RAM, MDA display, no floppy, no 8087
/// Bits 7-6: Number of floppy drives (00 = 1)
/// Bits 5-4: Video mode (11 = MDA 80x25)
/// Bits 3-2: RAM size (11 = 64K)
/// Bit 1: 8087 installed (0 = no)
/// Bit 0: Floppy installed (1 = no)
const DIP_SWITCHES: u8 = 0b00111101; // 0x3D

/// Keyboard reset state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyboardResetState {
    /// Normal operation (bit 6 = 0, clock enabled)
    Idle,
    /// Reset in progress (bit 6 = 1, clock disabled)
    ResetAsserted,
}

/// Intel 8255 PPI for IBM PC
///
/// Handles keyboard data, DIP switches, and system control ports.
/// Owns the Keyboard scancode buffer internally.
pub struct Ppi {
    /// Keyboard scancode buffer (owned by PPI)
    keyboard: Keyboard,

    /// Latched scancode ready to be read from Port A
    latched_scancode: Option<u8>,

    /// Internal interrupt state (determines IRQ1 level)
    interrupt_pending: bool,

    /// Current value of Port B (system control)
    port_b_state: u8,

    /// DIP switch configuration
    dip_switches: u8,

    /// Keyboard reset state machine
    reset_state: KeyboardResetState,

    /// Flag indicating reset just completed and we need to trigger keyboard reset
    reset_pending: bool,
}

impl Ppi {
    /// Create a new PPI with default DIP switch configuration
    pub fn new(scancode_queue: Arc<RwLock<VecDeque<u8>>>) -> Self {
        Self {
            keyboard: Keyboard::new(scancode_queue),
            latched_scancode: None,
            interrupt_pending: false,
            port_b_state: 0x00,
            dip_switches: DIP_SWITCHES,
            reset_state: KeyboardResetState::Idle,
            reset_pending: false,
        }
    }

    /// Create a new PPI with custom DIP switch configuration
    pub fn with_dip_switches(scancode_queue: Arc<RwLock<VecDeque<u8>>>, dip_switches: u8) -> Self {
        Self {
            keyboard: Keyboard::new(scancode_queue),
            latched_scancode: None,
            interrupt_pending: false,
            port_b_state: 0x00,
            dip_switches,
            reset_state: KeyboardResetState::Idle,
            reset_pending: false,
        }
    }

    /// Get the keyboard scancode queue for GUI integration
    pub fn scancode_queue(&self) -> Arc<RwLock<VecDeque<u8>>> {
        self.keyboard.scancode_queue()
    }
}

impl IoDevice for Ppi {
    fn port_range(&self) -> RangeInclusive<u16> {
        PPI_PORT_A..=PPI_PORT_C
    }

    fn read_u8(&mut self, port: u16) -> u8 {
        match port {
            PPI_PORT_A => {
                // Bit 7 of Port B selects what Port A returns:
                // 1 = DIP switches, 0 = keyboard scancode
                if self.port_b_state & 0x80 != 0 {
                    // Return DIP switches
                    self.dip_switches
                } else {
                    // Return latched scancode (or DIP switches if none)
                    let data = self.latched_scancode.unwrap_or(self.dip_switches);

                    // Clear latched scancode after reading (in keyboard mode)
                    self.latched_scancode = None;

                    // Lower internal interrupt state
                    self.interrupt_pending = false;

                    data
                }
            }

            PPI_PORT_B => {
                // Read back Port B state
                // Real hardware has some read-only status bits here
                // For now just return 0x00 like the original implementation
                0x00
            }

            PPI_PORT_C => {
                // System Control Port C
                // Bit 7: Parity error (always 0 - no parity checking)
                // Bit 6: I/O channel check (always 0)
                // Bit 5: Timer 2 output state
                // Bit 4: RAM parity error (always 0)
                // Bits 3-0: Various system status
                0x00
            }

            _ => 0xFF,
        }
    }

    fn write_u8(&mut self, port: u16, value: u8) {
        match port {
            PPI_PORT_A => {
                // Port A is typically input-only on IBM PC
                // Writes are ignored
            }

            PPI_PORT_B => {
                // Detect keyboard reset via bit 6 transitions
                // Bit 6: 0 = keyboard clock enabled, 1 = keyboard clock disabled (reset)
                let old_bit6 = (self.port_b_state & 0x40) != 0;
                let new_bit6 = (value & 0x40) != 0;

                match (old_bit6, new_bit6) {
                    (false, true) => {
                        // Keyboard clock disabled (reset asserted)
                        self.reset_state = KeyboardResetState::ResetAsserted;
                        // Clear any latched scancode during reset
                        self.latched_scancode = None;
                        self.interrupt_pending = false;
                    }
                    (true, false) => {
                        // Keyboard clock re-enabled (reset released)
                        // Schedule keyboard reset for next tick
                        if self.reset_state == KeyboardResetState::ResetAsserted {
                            self.reset_pending = true;
                            self.reset_state = KeyboardResetState::Idle;
                        }
                    }
                    _ => {
                        // No transition on bit 6
                    }
                }

                self.port_b_state = value;
            }

            PPI_PORT_C => {
                // Port C writes are typically ignored on IBM PC
                let _ = value;
            }

            _ => {}
        }
    }

    fn tick(&mut self, _cycles: u16, pic: &mut Pic) {
        // Step 1: Set PIC IRQ1 to match internal interrupt state
        if self.interrupt_pending {
            pic.set_irq_level(1, true);
        } else {
            pic.set_irq_level(1, false);
        }

        // Step 2: Handle keyboard reset if pending
        if self.reset_pending {
            self.keyboard.reset();
            self.reset_pending = false;
        }

        // Step 3: If no latched scancode, check keyboard for buffered scancodes
        if self.latched_scancode.is_none() {
            if let Some(scancode) = self.keyboard.pop_scancode() {
                // Latch the new scancode
                self.latched_scancode = Some(scancode);

                // Raise internal interrupt state
                self.interrupt_pending = true;

                // Immediately raise IRQ1
                pic.set_irq_level(1, true);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppi_new() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let ppi = Ppi::new(queue);
        assert_eq!(ppi.latched_scancode, None);
        assert!(!ppi.interrupt_pending);
        assert_eq!(ppi.dip_switches, DIP_SWITCHES);
    }

    #[test]
    fn test_ppi_port_range() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let ppi = Ppi::new(queue);
        let range = ppi.port_range();
        assert_eq!(*range.start(), 0x60);
        assert_eq!(*range.end(), 0x62);
    }

    #[test]
    fn test_port_60_returns_dip_switches_by_default() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut ppi = Ppi::new(queue);
        // Port 0x60 should return DIP switches by default (no scancode latched)
        assert_eq!(ppi.read_u8(PPI_PORT_A), DIP_SWITCHES);
    }

    #[test]
    fn test_port_61_write_bit7_high_returns_dip_switches() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut ppi = Ppi::new(queue);

        // Write to port 0x61 with bit 7 = 1 (select DIP switches)
        ppi.write_u8(PPI_PORT_B, 0x80);

        // Port 0x60 should return DIP switches
        assert_eq!(ppi.read_u8(PPI_PORT_A), DIP_SWITCHES);
    }

    #[test]
    fn test_port_61_write_bit7_low_returns_scancode() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut ppi = Ppi::new(queue.clone());
        let mut pic = Pic::new(0x08);

        // Add scancode to queue and tick to latch it
        queue.write().unwrap().push_back(0x1E); // 'A' key
        ppi.tick(1, &mut pic);

        // Write to port 0x61 with bit 7 = 0 (select keyboard data)
        ppi.write_u8(PPI_PORT_B, 0x00);

        // Port 0x60 should return the keyboard scancode
        assert_eq!(ppi.read_u8(PPI_PORT_A), 0x1E);
    }

    #[test]
    fn test_reading_scancode_clears_it() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut ppi = Ppi::new(queue.clone());
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Add scancode and tick
        queue.write().unwrap().push_back(0x1E);
        ppi.tick(1, &mut pic);

        // Read the scancode
        ppi.write_u8(PPI_PORT_B, 0x00); // Keyboard mode
        let data = ppi.read_u8(PPI_PORT_A);
        assert_eq!(data, 0x1E);

        // Scancode should be cleared
        assert_eq!(ppi.latched_scancode, None);
        assert!(!ppi.interrupt_pending);
    }

    #[test]
    fn test_tick_raises_irq() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut ppi = Ppi::new(queue.clone());
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Add scancode to queue
        queue.write().unwrap().push_back(0x1E);

        // Tick should latch scancode and raise IRQ1
        ppi.tick(1, &mut pic);

        assert_eq!(ppi.latched_scancode, Some(0x1E));
        assert!(ppi.interrupt_pending);
        assert!(pic.intr_out());
    }

    #[test]
    fn test_irq_lowered_after_read() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut ppi = Ppi::new(queue.clone());
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Add scancode and raise IRQ
        queue.write().unwrap().push_back(0x1E);
        ppi.tick(1, &mut pic);
        assert!(pic.intr_out());

        // Read scancode (clears interrupt_pending)
        let _data = ppi.read_u8(PPI_PORT_A);

        // Acknowledge the interrupt
        let _vector = pic.inta();
        pic.eoi();

        // Next tick should lower IRQ line
        ppi.tick(1, &mut pic);
        assert!(!pic.intr_out());
    }

    #[test]
    fn test_keyboard_reset_basic() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut ppi = Ppi::new(queue);
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // 1. Assert reset (bit 6 = 1)
        ppi.write_u8(PPI_PORT_B, 0x40);
        assert_eq!(ppi.reset_state, KeyboardResetState::ResetAsserted);

        // 2. Release reset (bit 6 = 0)
        ppi.write_u8(PPI_PORT_B, 0x00);
        assert!(ppi.reset_pending);

        // 3. Tick should call keyboard.reset() and latch 0xAA
        ppi.tick(1, &mut pic);
        assert!(!ppi.reset_pending);
        assert_eq!(ppi.latched_scancode, Some(0xAA));
        assert!(pic.intr_out());

        // 4. Read should return 0xAA
        assert_eq!(ppi.read_u8(PPI_PORT_A), 0xAA);
    }

    #[test]
    fn test_keyboard_reset_clears_pending() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut ppi = Ppi::new(queue.clone());
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Add a scancode and latch it
        queue.write().unwrap().push_back(0x1E);
        ppi.tick(1, &mut pic);
        assert_eq!(ppi.latched_scancode, Some(0x1E));

        // Assert reset - should clear the latched scancode
        ppi.write_u8(PPI_PORT_B, 0x40);
        assert_eq!(ppi.latched_scancode, None);

        // Release reset
        ppi.write_u8(PPI_PORT_B, 0x00);

        // Tick should latch 0xAA (from keyboard.reset())
        ppi.tick(1, &mut pic);
        assert_eq!(ppi.read_u8(PPI_PORT_A), 0xAA);
    }

    #[test]
    fn test_keyboard_reset_clears_queue() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut ppi = Ppi::new(queue.clone());
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Add multiple scancodes to queue
        queue.write().unwrap().push_back(0x1E);
        queue.write().unwrap().push_back(0x30);
        queue.write().unwrap().push_back(0x2E);

        // Perform reset
        ppi.write_u8(PPI_PORT_B, 0x40);
        ppi.write_u8(PPI_PORT_B, 0x00);

        // Tick - keyboard.reset() clears queue and adds 0xAA
        ppi.tick(1, &mut pic);
        assert_eq!(ppi.read_u8(PPI_PORT_A), 0xAA);

        // Queue should be empty (only 0xAA was there, and we consumed it)
        ppi.tick(1, &mut pic);
        assert_eq!(ppi.latched_scancode, None);
    }

    #[test]
    fn test_multiple_scancodes() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut ppi = Ppi::new(queue.clone());
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Add multiple scancodes
        queue.write().unwrap().push_back(0x1E); // 'A' make
        queue.write().unwrap().push_back(0x9E); // 'A' break

        // First tick gets first scancode
        ppi.tick(1, &mut pic);
        assert_eq!(ppi.read_u8(PPI_PORT_A), 0x1E);

        // Second tick gets second scancode
        ppi.tick(1, &mut pic);
        assert_eq!(ppi.read_u8(PPI_PORT_A), 0x9E);
    }

    #[test]
    fn test_port_c_returns_zero() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut ppi = Ppi::new(queue);
        assert_eq!(ppi.read_u8(PPI_PORT_C), 0x00);

        // Writes should be ignored
        ppi.write_u8(PPI_PORT_C, 0xFF);
        assert_eq!(ppi.read_u8(PPI_PORT_C), 0x00);
    }

    #[test]
    fn test_port_b_returns_zero() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut ppi = Ppi::new(queue);
        assert_eq!(ppi.read_u8(PPI_PORT_B), 0x00);
    }

    #[test]
    fn test_dip_mode_read_does_not_clear_scancode() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut ppi = Ppi::new(queue.clone());
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Add and latch scancode
        queue.write().unwrap().push_back(0x1E);
        ppi.tick(1, &mut pic);

        // Switch to DIP mode
        ppi.write_u8(PPI_PORT_B, 0x80);

        // Read DIP switches - should NOT clear the latched scancode
        assert_eq!(ppi.read_u8(PPI_PORT_A), DIP_SWITCHES);
        assert_eq!(ppi.latched_scancode, Some(0x1E)); // Still there

        // Switch back to keyboard mode
        ppi.write_u8(PPI_PORT_B, 0x00);

        // Should still have the scancode
        assert_eq!(ppi.read_u8(PPI_PORT_A), 0x1E);
    }

    #[test]
    fn test_reset_only_on_falling_edge() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut ppi = Ppi::new(queue);
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Write bit 6=1 multiple times (no transition after first)
        ppi.write_u8(PPI_PORT_B, 0x40);
        ppi.write_u8(PPI_PORT_B, 0x40);
        ppi.write_u8(PPI_PORT_B, 0x40);

        // Should be in ResetAsserted state
        assert_eq!(ppi.reset_state, KeyboardResetState::ResetAsserted);
        assert!(!ppi.reset_pending);

        // Single falling edge (1→0) should trigger reset
        ppi.write_u8(PPI_PORT_B, 0x00);
        assert!(ppi.reset_pending);

        // Tick should generate exactly one 0xAA
        ppi.tick(1, &mut pic);
        assert_eq!(ppi.read_u8(PPI_PORT_A), 0xAA);

        // Additional writes with bit 6=0 (no transition) should not generate more 0xAA
        ppi.write_u8(PPI_PORT_B, 0x00);
        ppi.write_u8(PPI_PORT_B, 0x00);
        ppi.tick(1, &mut pic);

        // No new scancode should be generated
        assert_eq!(ppi.latched_scancode, None);
    }
}
