//! Intel 8259 Programmable Interrupt Controller (PIC)
//!
//! The IBM PC uses a single 8259 PIC in edge-triggered mode to manage
//! hardware interrupts from peripherals.

use crate::io::IoDevice;
use std::ops::RangeInclusive;

/// PIC I/O ports
const PIC_COMMAND_PORT: u16 = 0x20;
const PIC_DATA_PORT: u16 = 0x21;

/// EOI (End of Interrupt) command
const EOI_COMMAND: u8 = 0x20;

/// Initialization sequence state
#[derive(Debug, Clone, Copy, PartialEq)]
enum InitState {
    /// Normal operation mode - data port writes go to IMR
    Ready,
    /// Waiting for ICW2 (vector offset)
    WaitIcw2,
    /// Waiting for ICW3 (cascade mode only - not used in IBM PC)
    WaitIcw3,
    /// Waiting for ICW4 (if IC4 bit was set in ICW1)
    WaitIcw4,
}

/// Intel 8259 PIC state
///
/// The 8259 manages 8 IRQ lines (IRQ0-IRQ7) and converts them into
/// interrupt vectors for the CPU. The original IBM PC uses edge-triggered
/// mode where interrupts are triggered on the rising edge (low-to-high transition)
/// of the IRQ line.
pub struct Pic {
    /// Interrupt Mask Register (IMR) - bit set = IRQ masked
    imr: u8,

    /// Interrupt Request Register (IRR) - pending interrupt requests
    irr: u8,

    /// In-Service Register (ISR) - currently serviced interrupts
    isr: u8,

    /// Previous IRQ line states for edge detection
    /// Used to detect rising edges (low-to-high transitions)
    irq_prev: u8,

    /// Base interrupt vector offset (configured via ICW2)
    /// For IBM PC: typically 0x08 for IRQ0-7 -> INT 0x08-0x0F
    vector_offset: u8,

    /// Initialization sequence state
    init_state: InitState,

    /// ICW1 flags stored during initialization
    /// Bit 0 (IC4): 1 = ICW4 needed
    /// Bit 1 (SNGL): 1 = single mode (no cascade)
    icw1_flags: u8,

    /// Auto EOI mode (from ICW4 bit 1)
    auto_eoi: bool,

    /// Read register select (from OCW3)
    /// false = read IRR, true = read ISR
    read_isr: bool,
}

impl Pic {
    /// Create a new 8259 PIC
    ///
    /// Initializes with all interrupts masked and no pending requests.
    /// The vector_offset should be set to 0x08 for the IBM PC.
    pub fn new(vector_offset: u8) -> Self {
        Self {
            imr: 0xFF, // All interrupts masked by default
            irr: 0,
            isr: 0,
            irq_prev: 0,
            vector_offset,
            init_state: InitState::Ready,
            icw1_flags: 0,
            auto_eoi: false,
            read_isr: false,
        }
    }

    /// Set an IRQ line level (edge-triggered mode)
    ///
    /// In edge-triggered mode, the PIC detects a rising edge (transition from
    /// low to high) and latches the interrupt request in the IRR.
    ///
    /// # Arguments
    /// * `irq` - IRQ line number (0-7)
    /// * `level` - true for high, false for low
    pub fn set_irq_level(&mut self, irq: u8, level: bool) {
        debug_assert!(irq < 8, "IRQ must be 0-7");

        let bit = 1 << irq;
        let prev_level = (self.irq_prev & bit) != 0;
        let new_level = level;

        // Update the previous state
        if level {
            self.irq_prev |= bit;
        } else {
            self.irq_prev &= !bit;
        }

        // Edge-triggered: detect rising edge (low-to-high transition)
        if !prev_level && new_level {
            // Latch the interrupt request in IRR
            self.irr |= bit;
        }
    }

    /// Check if interrupt output line should be active
    ///
    /// Returns true if there are any unmasked pending interrupts that
    /// are not currently being serviced.
    pub fn intr_out(&self) -> bool {
        // Pending interrupts that aren't masked
        let pending = self.irr & !self.imr;
        pending != 0
    }

    /// Interrupt Acknowledge - return the interrupt vector
    ///
    /// Called by the CPU when it's ready to service an interrupt.
    /// Returns the interrupt vector number for the highest priority
    /// pending interrupt.
    ///
    /// The 8259 uses fixed priority with IRQ0 as highest and IRQ7 as lowest.
    ///
    /// This also:
    /// - Clears the interrupt from IRR
    /// - Sets the corresponding bit in ISR (marking it as in-service)
    pub fn inta(&mut self) -> u8 {
        // Find highest priority unmasked pending interrupt
        let pending = self.irr & !self.imr;

        if pending == 0 {
            // No pending interrupts - return spurious interrupt vector
            // Spurious interrupt is typically IRQ7 (vector_offset + 7)
            return self.vector_offset + 7;
        }

        // Find the lowest bit set (highest priority)
        let irq = pending.trailing_zeros() as u8;

        let bit = 1 << irq;

        // Clear from IRR
        self.irr &= !bit;

        // Set in ISR (interrupt now being serviced)
        self.isr |= bit;

        // Return interrupt vector
        self.vector_offset + irq
    }

    /// Get the IMR (Interrupt Mask Register)
    pub fn get_imr(&self) -> u8 {
        self.imr
    }

    /// Set the IMR (Interrupt Mask Register)
    ///
    /// Bit set = interrupt masked (disabled)
    /// Bit clear = interrupt enabled
    pub fn set_imr(&mut self, value: u8) {
        self.imr = value;
    }

    /// Get the IRR (Interrupt Request Register) - for debugging
    pub fn get_irr(&self) -> u8 {
        self.irr
    }

    /// Get the ISR (In-Service Register) - for debugging
    pub fn get_isr(&self) -> u8 {
        self.isr
    }

    /// End of Interrupt (EOI) command
    ///
    /// Clears the highest priority bit in the ISR, indicating the
    /// interrupt has been fully serviced.
    pub fn eoi(&mut self) {
        if self.isr != 0 {
            // Clear the highest priority (lowest bit) in ISR
            let irq = self.isr.trailing_zeros() as u8;
            self.isr &= !(1 << irq);
        }
    }
}

impl IoDevice for Pic {
    fn port_range(&self) -> RangeInclusive<u16> {
        PIC_COMMAND_PORT..=PIC_DATA_PORT
    }

    fn read_u8(&mut self, port: u16) -> u8 {
        match port {
            PIC_COMMAND_PORT => {
                // Reading from command port returns ISR or IRR based on OCW3 setting
                if self.read_isr {
                    self.isr
                } else {
                    self.irr
                }
            }
            PIC_DATA_PORT => {
                // Reading from data port returns IMR
                self.imr
            }
            _ => {
                println!("[PIC] Unhandled read from port 0x{:04X}", port);
                0xFF
            }
        }
    }

    fn write_u8(&mut self, port: u16, value: u8) {
        match port {
            PIC_COMMAND_PORT => {
                // Check if this is ICW1 (bit 4 set)
                if (value & 0x10) != 0 {
                    // ICW1 - start initialization sequence
                    #[cfg(debug_assertions)]
                    println!(
                        "[PIC] ICW1: 0x{:02X} (IC4={}, SNGL={})",
                        value,
                        (value & 0x01) != 0,
                        (value & 0x02) != 0
                    );

                    self.icw1_flags = value;
                    self.init_state = InitState::WaitIcw2;

                    // ICW1 clears ISR and IMR
                    self.isr = 0;
                    self.imr = 0;
                    self.read_isr = false;
                } else if (value & 0x08) != 0 {
                    // OCW3 - bits 4:3 = 01
                    // Bit 1:0 determine read register select
                    if (value & 0x02) != 0 {
                        self.read_isr = (value & 0x01) != 0;
                        #[cfg(debug_assertions)]
                        println!(
                            "[PIC] OCW3: read {} on next command port read",
                            if self.read_isr { "ISR" } else { "IRR" }
                        );
                    }
                } else {
                    // OCW2 - bits 4:3 = 00
                    // Handle EOI commands
                    let eoi_type = (value >> 5) & 0x07;
                    match eoi_type {
                        0b001 => {
                            // Non-specific EOI
                            self.eoi();
                        }
                        0b011 => {
                            // Specific EOI - clear specific IRQ from ISR
                            let irq = value & 0x07;
                            self.isr &= !(1 << irq);
                            #[cfg(debug_assertions)]
                            println!("[PIC] Specific EOI for IRQ{}", irq);
                        }
                        _ => {
                            #[cfg(debug_assertions)]
                            println!("[PIC] OCW2 command: 0x{:02X} (type {})", value, eoi_type);
                        }
                    }
                }
            }
            PIC_DATA_PORT => {
                // Data port behavior depends on initialization state
                match self.init_state {
                    InitState::WaitIcw2 => {
                        // ICW2 - interrupt vector offset (upper 5 bits)
                        self.vector_offset = value & 0xF8;
                        #[cfg(debug_assertions)]
                        println!("[PIC] ICW2: vector offset = 0x{:02X}", self.vector_offset);

                        // Check if we need ICW3 (cascade mode)
                        if (self.icw1_flags & 0x02) == 0 {
                            // Cascade mode - need ICW3
                            self.init_state = InitState::WaitIcw3;
                        } else if (self.icw1_flags & 0x01) != 0 {
                            // Single mode, ICW4 needed
                            self.init_state = InitState::WaitIcw4;
                        } else {
                            // Single mode, no ICW4
                            self.init_state = InitState::Ready;
                            #[cfg(debug_assertions)]
                            println!("[PIC] Initialization complete (no ICW4)");
                        }
                    }
                    InitState::WaitIcw3 => {
                        // ICW3 - cascade configuration (not used in IBM PC)
                        #[cfg(debug_assertions)]
                        println!("[PIC] ICW3: 0x{:02X} (cascade config)", value);

                        if (self.icw1_flags & 0x01) != 0 {
                            self.init_state = InitState::WaitIcw4;
                        } else {
                            self.init_state = InitState::Ready;
                            #[cfg(debug_assertions)]
                            println!("[PIC] Initialization complete (no ICW4)");
                        }
                    }
                    InitState::WaitIcw4 => {
                        // ICW4 - mode configuration
                        self.auto_eoi = (value & 0x02) != 0;
                        #[cfg(debug_assertions)]
                        println!(
                            "[PIC] ICW4: 0x{:02X} (8086 mode={}, AEOI={})",
                            value,
                            (value & 0x01) != 0,
                            self.auto_eoi
                        );

                        self.init_state = InitState::Ready;
                        #[cfg(debug_assertions)]
                        println!("[PIC] Initialization complete");
                    }
                    InitState::Ready => {
                        // Normal operation - write to IMR (OCW1)
                        #[cfg(debug_assertions)]
                        if self.imr != value {
                            println!(
                                "[PIC] IMR change: 0x{:02X} -> 0x{:02X} (IRQ6 {})",
                                self.imr,
                                value,
                                if (value & 0x40) != 0 {
                                    "MASKED"
                                } else {
                                    "enabled"
                                }
                            );
                        }
                        self.imr = value;
                    }
                }
            }
            _ => {
                println!(
                    "[PIC] Unhandled write to port 0x{:04X} = 0x{:02X}",
                    port, value
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pic_new() {
        let pic = Pic::new(0x08);
        assert_eq!(pic.get_imr(), 0xFF); // All masked
        assert_eq!(pic.get_irr(), 0);
        assert_eq!(pic.get_isr(), 0);
        assert!(!pic.intr_out());
    }

    #[test]
    fn test_edge_triggered_rising_edge() {
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00); // Enable all interrupts

        // Set IRQ0 low
        pic.set_irq_level(0, false);
        assert_eq!(pic.get_irr(), 0);
        assert!(!pic.intr_out());

        // Rising edge: low -> high
        pic.set_irq_level(0, true);
        assert_eq!(pic.get_irr(), 0x01); // IRQ0 latched
        assert!(pic.intr_out());
    }

    #[test]
    fn test_edge_triggered_no_falling_edge() {
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Set high
        pic.set_irq_level(0, true);
        assert_eq!(pic.get_irr(), 0x01);

        // Falling edge should not trigger
        pic.set_irq_level(0, false);
        assert_eq!(pic.get_irr(), 0x01); // Still latched
    }

    #[test]
    fn test_edge_triggered_repeated_high() {
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // First rising edge
        pic.set_irq_level(0, false);
        pic.set_irq_level(0, true);
        assert_eq!(pic.get_irr(), 0x01);

        // Staying high should not re-trigger
        pic.set_irq_level(0, true);
        assert_eq!(pic.get_irr(), 0x01);
    }

    #[test]
    fn test_masked_interrupt() {
        let mut pic = Pic::new(0x08);
        pic.set_imr(0xFF); // All masked

        // Trigger IRQ0 (rising edge)
        pic.set_irq_level(0, false);
        pic.set_irq_level(0, true);

        // IRR should be set but intr_out should be false (masked)
        assert_eq!(pic.get_irr(), 0x01);
        assert!(!pic.intr_out());

        // Unmask IRQ0
        pic.set_imr(0xFE);
        assert!(pic.intr_out());
    }

    #[test]
    fn test_inta_returns_vector() {
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Trigger IRQ3
        pic.set_irq_level(3, false);
        pic.set_irq_level(3, true);

        assert!(pic.intr_out());

        // Acknowledge interrupt
        let vector = pic.inta();
        assert_eq!(vector, 0x08 + 3); // 0x0B

        // Should be moved from IRR to ISR
        assert_eq!(pic.get_irr(), 0);
        assert_eq!(pic.get_isr(), 0x08); // Bit 3 set
        assert!(!pic.intr_out()); // No more pending
    }

    #[test]
    fn test_priority_order() {
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Trigger multiple IRQs
        pic.set_irq_level(5, false);
        pic.set_irq_level(5, true);
        pic.set_irq_level(2, false);
        pic.set_irq_level(2, true);
        pic.set_irq_level(7, false);
        pic.set_irq_level(7, true);

        // IRQ2 should be serviced first (highest priority)
        let vector = pic.inta();
        assert_eq!(vector, 0x08 + 2);

        // Then IRQ5
        let vector = pic.inta();
        assert_eq!(vector, 0x08 + 5);

        // Then IRQ7
        let vector = pic.inta();
        assert_eq!(vector, 0x08 + 7);

        // No more interrupts
        assert!(!pic.intr_out());
    }

    #[test]
    fn test_eoi() {
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Trigger and acknowledge IRQ1
        pic.set_irq_level(1, false);
        pic.set_irq_level(1, true);
        let _vector = pic.inta();

        assert_eq!(pic.get_isr(), 0x02); // Bit 1 set

        // Send EOI
        pic.eoi();
        assert_eq!(pic.get_isr(), 0); // Cleared
    }

    #[test]
    fn test_spurious_interrupt() {
        let mut pic = Pic::new(0x08);
        // No pending interrupts
        let vector = pic.inta();
        // Should return spurious vector (offset + 7)
        assert_eq!(vector, 0x08 + 7);
    }

    // I/O Port tests
    #[test]
    fn test_io_port_range() {
        let pic = Pic::new(0x08);
        let range = pic.port_range();
        assert_eq!(*range.start(), 0x20);
        assert_eq!(*range.end(), 0x21);
    }

    #[test]
    fn test_io_write_read_imr() {
        let mut pic = Pic::new(0x08);

        // Write to data port (IMR)
        pic.write_u8(PIC_DATA_PORT, 0xAB);

        // Read back from data port
        let value = pic.read_u8(PIC_DATA_PORT);
        assert_eq!(value, 0xAB);
        assert_eq!(pic.get_imr(), 0xAB);
    }

    #[test]
    fn test_io_eoi_command() {
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Trigger and acknowledge IRQ2
        pic.set_irq_level(2, false);
        pic.set_irq_level(2, true);
        let _vector = pic.inta();

        assert_eq!(pic.get_isr(), 0x04); // Bit 2 set

        // Send EOI via I/O port
        pic.write_u8(PIC_COMMAND_PORT, EOI_COMMAND);

        // ISR should be cleared
        assert_eq!(pic.get_isr(), 0);
    }

    #[test]
    fn test_io_read_command_port_returns_irr() {
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00);

        // Trigger IRQ0 and IRQ3
        pic.set_irq_level(0, false);
        pic.set_irq_level(0, true);
        pic.set_irq_level(3, false);
        pic.set_irq_level(3, true);

        // Reading command port should return IRR
        let value = pic.read_u8(PIC_COMMAND_PORT);
        assert_eq!(value, 0x09); // Bits 0 and 3 set
    }

    #[test]
    fn test_io_imr_affects_interrupts() {
        let mut pic = Pic::new(0x08);

        // Trigger IRQ1
        pic.set_irq_level(1, false);
        pic.set_irq_level(1, true);

        // Mask IRQ1 via I/O port
        pic.write_u8(PIC_DATA_PORT, 0xFF);
        assert!(!pic.intr_out()); // Should be masked

        // Unmask IRQ1
        pic.write_u8(PIC_DATA_PORT, 0xFD); // All except bit 1
        assert!(pic.intr_out()); // Should now signal interrupt
    }

    #[test]
    fn test_icw_initialization_sequence() {
        let mut pic = Pic::new(0x00); // Initial offset doesn't matter

        // IBM PC BIOS initialization sequence:
        // ICW1: 0x13 = single mode, ICW4 needed
        pic.write_u8(PIC_COMMAND_PORT, 0x13);
        // After ICW1, ISR and IMR should be cleared
        assert_eq!(pic.get_isr(), 0);
        assert_eq!(pic.get_imr(), 0);

        // ICW2: vector offset = 0x08
        pic.write_u8(PIC_DATA_PORT, 0x08);
        // Vector offset should be set
        let vector_offset = pic.vector_offset;
        assert_eq!(vector_offset, 0x08);

        // ICW4: 0x09 = 8086 mode, normal EOI
        pic.write_u8(PIC_DATA_PORT, 0x09);

        // Now in ready state - writes to data port should be IMR
        pic.write_u8(PIC_DATA_PORT, 0xFB); // Mask all except IRQ2
        assert_eq!(pic.get_imr(), 0xFB);

        // Verify interrupt delivery with new vector offset
        pic.set_irq_level(2, false);
        pic.set_irq_level(2, true);
        let vector = pic.inta();
        assert_eq!(vector, 0x08 + 2); // Should be 0x0A
    }

    #[test]
    fn test_icw_reinit_does_not_corrupt_imr() {
        let mut pic = Pic::new(0x08);

        // Set initial IMR
        pic.write_u8(PIC_DATA_PORT, 0x55);
        assert_eq!(pic.get_imr(), 0x55);

        // Start reinitialization - this should clear IMR
        pic.write_u8(PIC_COMMAND_PORT, 0x13);
        assert_eq!(pic.get_imr(), 0); // ICW1 clears IMR

        // Complete initialization sequence
        pic.write_u8(PIC_DATA_PORT, 0x08); // ICW2
        pic.write_u8(PIC_DATA_PORT, 0x09); // ICW4

        // Now write to IMR
        pic.write_u8(PIC_DATA_PORT, 0xAA);
        assert_eq!(pic.get_imr(), 0xAA); // Should update IMR, not something else
    }
}
