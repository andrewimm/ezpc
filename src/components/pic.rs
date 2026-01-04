//! Intel 8259 Programmable Interrupt Controller (PIC)
//!
//! The IBM PC uses a single 8259 PIC in edge-triggered mode to manage
//! hardware interrupts from peripherals.

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
}
