//! Intel 8253/8254 Programmable Interval Timer (PIT)
//!
//! The PIT has 3 counters (channels):
//! - Counter 0 (port 0x40): System timer, connected to IRQ0
//! - Counter 1 (port 0x41): DRAM refresh timer (mostly ignored on modern systems)
//! - Counter 2 (port 0x42): Speaker control (PC speaker tone generation)
//! - Control Word (port 0x43): Write-only configuration register
//!
//! Input clock: 1.193182 MHz (14.31818 MHz crystal / 12)

use crate::components::pic::Pic;
use crate::io::IoDevice;
use std::ops::RangeInclusive;

/// PIT I/O port constants
const PIT_COUNTER_0: u16 = 0x40;
const PIT_COUNTER_1: u16 = 0x41;
const PIT_COUNTER_2: u16 = 0x42;
const PIT_CONTROL: u16 = 0x43;

/// PIT input clock frequency in Hz
/// The original IBM PC uses a 14.31818 MHz crystal divided by 12
const PIT_CLOCK_HZ: f64 = 1_193_182.0;

/// How many CPU cycles (at 4.77 MHz) per PIT tick
/// 4.77 MHz / 1.193182 MHz = ~4 cycles per PIT tick
const CPU_CYCLES_PER_PIT_TICK: u16 = 4;

/// Counter access modes
#[derive(Debug, Clone, Copy, PartialEq)]
enum AccessMode {
    LatchCount,   // Command to latch current count
    LowByteOnly,  // Read/write low byte only
    HighByteOnly, // Read/write high byte only
    LowThenHigh,  // Read/write low byte, then high byte
}

/// Counter operating modes (0-5)
#[derive(Debug, Clone, Copy, PartialEq)]
enum CounterMode {
    Mode0, // Interrupt on terminal count
    Mode1, // Hardware retriggerable one-shot
    Mode2, // Rate generator (most common for system timer)
    Mode3, // Square wave generator
    Mode4, // Software triggered strobe
    Mode5, // Hardware triggered strobe
}

/// State of a single PIT counter
struct Counter {
    /// Current count value (decrements with each tick)
    count: u16,

    /// Reload value (loaded into count when it reaches 0)
    reload_value: u16,

    /// Latched count (for read operations)
    latch: Option<u16>,

    /// Access mode (how to read/write the counter)
    access_mode: AccessMode,

    /// Operating mode (0-5)
    mode: CounterMode,

    /// BCD mode flag (false = binary, true = BCD)
    /// We'll only implement binary mode
    bcd: bool,

    /// Byte toggle for LowThenHigh access mode
    /// false = expecting low byte, true = expecting high byte
    byte_toggle: bool,

    /// Output pin state (high/low)
    output: bool,

    /// Gate input state (for modes that use it)
    gate: bool,

    /// Null count flag (true if count hasn't been loaded yet)
    null_count: bool,
}

impl Counter {
    fn new() -> Self {
        Self {
            count: 0,
            reload_value: 0,
            latch: None,
            access_mode: AccessMode::LowThenHigh,
            mode: CounterMode::Mode0,
            bcd: false,
            byte_toggle: false,
            output: false,
            gate: true, // Counter 0 and 1 gate always high
            null_count: true,
        }
    }

    /// Load a new count value (handles both byte modes)
    fn write_count(&mut self, value: u8) {
        match self.access_mode {
            AccessMode::LowByteOnly => {
                // For 8-bit modes, 0 means 256 (0x100)
                let count = if value == 0 { 0x100 } else { value as u16 };
                self.reload_value = count;
                self.count = count;
                self.null_count = false;
            }
            AccessMode::HighByteOnly => {
                // For 8-bit modes, 0 means 256 (0x100)
                let count = if value == 0 { 0x100 } else { value as u16 };
                self.reload_value = count;
                self.count = count;
                self.null_count = false;
            }
            AccessMode::LowThenHigh => {
                if !self.byte_toggle {
                    // Receiving low byte
                    self.reload_value = (self.reload_value & 0xFF00) | (value as u16);
                    self.byte_toggle = true;
                } else {
                    // Receiving high byte
                    self.reload_value = (self.reload_value & 0x00FF) | ((value as u16) << 8);
                    // For 16-bit mode, 0x0000 means 65536, but we store 0 as a special marker
                    // The tick() method will handle reload_value == 0 specially
                    self.count = self.reload_value;
                    self.byte_toggle = false;
                    self.null_count = false;
                }
            }
            AccessMode::LatchCount => {
                // Latch command doesn't write
            }
        }
    }

    /// Read current count value (handles both byte modes and latching)
    fn read_count(&mut self) -> u8 {
        let count_to_read = self.latch.unwrap_or(self.count);

        match self.access_mode {
            AccessMode::LowByteOnly => {
                self.latch = None; // Clear latch after read
                (count_to_read & 0xFF) as u8
            }
            AccessMode::HighByteOnly => {
                self.latch = None;
                (count_to_read >> 8) as u8
            }
            AccessMode::LowThenHigh => {
                if !self.byte_toggle {
                    // Return low byte first
                    self.byte_toggle = true;
                    (count_to_read & 0xFF) as u8
                } else {
                    // Return high byte
                    self.byte_toggle = false;
                    self.latch = None; // Clear latch after reading both bytes
                    (count_to_read >> 8) as u8
                }
            }
            AccessMode::LatchCount => 0xFF, // Should not happen
        }
    }

    /// Decrement counter (returns true if it wrapped to 0)
    fn tick(&mut self) -> bool {
        if self.null_count {
            return false; // Counter not initialized
        }

        // Handle count of 0 specially - it represents 65536, so wrap to 0xFFFF
        // This happens either on initial load with count=0, or after reload
        if self.count == 0 {
            self.count = 0xFFFF;
            return false;
        }

        self.count -= 1;

        if self.count == 0 {
            // Counter reached zero - fire IRQ and reload
            // Note: reload_value of 0 means 65536, so we set count to 0
            // (next tick will wrap it to 0xFFFF and continue counting)
            self.count = self.reload_value;
            return true;
        }

        false
    }
}

/// Programmable Interval Timer
pub struct Pit {
    /// The three counters
    counters: [Counter; 3],

    /// Accumulated CPU cycles (fractional tracking for PIT clock conversion)
    cycle_accumulator: u16,

    /// Track if counter 0 should raise IRQ0
    irq0_pending: bool,
}

impl Pit {
    pub fn new() -> Self {
        Self {
            counters: [Counter::new(), Counter::new(), Counter::new()],
            cycle_accumulator: 0,
            irq0_pending: false,
        }
    }

    /// Parse and execute control word
    fn write_control(&mut self, value: u8) {
        let counter_select = (value >> 6) & 0x03;
        let access_mode_bits = (value >> 4) & 0x03;
        let mode_bits = (value >> 1) & 0x07;
        let bcd = (value & 0x01) != 0;

        // Read-back command (not implementing for minimal version)
        if counter_select == 0b11 {
            unimplemented!("PIT read-back command not supported");
        }

        // Check for BCD mode
        if bcd {
            panic!("PIT BCD mode not supported");
        }

        let counter = &mut self.counters[counter_select as usize];

        // Handle latch count command
        if access_mode_bits == 0b00 {
            // Latch current count
            if counter.latch.is_none() {
                counter.latch = Some(counter.count);
            }
            return;
        }

        // Set access mode
        counter.access_mode = match access_mode_bits {
            0b01 => AccessMode::LowByteOnly,
            0b10 => AccessMode::HighByteOnly,
            0b11 => AccessMode::LowThenHigh,
            _ => AccessMode::LowThenHigh, // Shouldn't happen
        };

        // Set mode
        counter.mode = match mode_bits {
            0b000 => CounterMode::Mode0,
            0b001 => CounterMode::Mode1,
            0b010 | 0b110 => CounterMode::Mode2,
            0b011 | 0b111 => CounterMode::Mode3,
            0b100 => CounterMode::Mode4,
            0b101 => CounterMode::Mode5,
            _ => CounterMode::Mode0,
        };

        counter.bcd = bcd;
        counter.byte_toggle = false; // Reset toggle on new control word
        counter.null_count = true; // Wait for count to be loaded
    }

    /// Update PIT state based on CPU cycles
    /// Returns true if IRQ0 should be raised
    fn tick_internal(&mut self, cpu_cycles: u16) -> bool {
        // Accumulate CPU cycles and convert to PIT ticks
        self.cycle_accumulator += cpu_cycles;

        let mut irq0_triggered = false;

        // Process accumulated PIT ticks
        while self.cycle_accumulator >= CPU_CYCLES_PER_PIT_TICK {
            self.cycle_accumulator -= CPU_CYCLES_PER_PIT_TICK;

            // Tick counter 0 (system timer)
            if self.counters[0].tick() {
                // Counter 0 wrapped - raise IRQ0
                // In mode 2 (rate generator) or mode 3 (square wave),
                // output pulse occurs on reload
                irq0_triggered = true;
            }

            // Tick counter 1 (DRAM refresh) - we don't care about output
            self.counters[1].tick();

            // Tick counter 2 (speaker) - stub for now
            self.counters[2].tick();
        }

        irq0_triggered
    }
}

impl IoDevice for Pit {
    fn port_range(&self) -> RangeInclusive<u16> {
        PIT_COUNTER_0..=PIT_CONTROL
    }

    fn read_u8(&mut self, port: u16) -> u8 {
        match port {
            PIT_COUNTER_0 => self.counters[0].read_count(),
            PIT_COUNTER_1 => self.counters[1].read_count(),
            PIT_COUNTER_2 => self.counters[2].read_count(),
            PIT_CONTROL => {
                // Control port is write-only, reading returns 0xFF
                0xFF
            }
            _ => 0xFF,
        }
    }

    fn write_u8(&mut self, port: u16, value: u8) {
        match port {
            PIT_COUNTER_0 => self.counters[0].write_count(value),
            PIT_COUNTER_1 => self.counters[1].write_count(value),
            PIT_COUNTER_2 => self.counters[2].write_count(value),
            PIT_CONTROL => self.write_control(value),
            _ => {}
        }
    }

    fn tick(&mut self, cycles: u16, pic: &mut Pic) {
        if self.tick_internal(cycles) {
            // Counter 0 triggered - raise IRQ0
            pic.set_irq_level(0, true);
            self.irq0_pending = true;
        } else if self.irq0_pending {
            // Lower IRQ0 after it's been raised
            // Edge-triggered mode means we raise then lower
            pic.set_irq_level(0, false);
            self.irq0_pending = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pit_new() {
        let pit = Pit::new();
        assert_eq!(pit.counters[0].null_count, true);
        assert_eq!(pit.cycle_accumulator, 0);
        assert_eq!(pit.irq0_pending, false);
    }

    #[test]
    fn test_pit_port_range() {
        let pit = Pit::new();
        let range = pit.port_range();
        assert_eq!(*range.start(), 0x40);
        assert_eq!(*range.end(), 0x43);
    }

    #[test]
    fn test_control_word_mode2() {
        let mut pit = Pit::new();
        // Counter 0, low+high byte, mode 2, binary
        pit.write_control(0b00110100); // 0x34

        assert_eq!(pit.counters[0].access_mode, AccessMode::LowThenHigh);
        assert_eq!(pit.counters[0].mode, CounterMode::Mode2);
        assert_eq!(pit.counters[0].bcd, false);
    }

    #[test]
    fn test_counter_write_low_high() {
        let mut pit = Pit::new();
        pit.write_control(0b00110100); // Counter 0, low+high, mode 2

        // Write low byte (0x00)
        pit.write_u8(PIT_COUNTER_0, 0x00);
        assert_eq!(pit.counters[0].byte_toggle, true);

        // Write high byte (0x10) -> reload value = 0x1000
        pit.write_u8(PIT_COUNTER_0, 0x10);
        assert_eq!(pit.counters[0].reload_value, 0x1000);
        assert_eq!(pit.counters[0].count, 0x1000);
        assert_eq!(pit.counters[0].null_count, false);
    }

    #[test]
    fn test_counter_read_low_high() {
        let mut pit = Pit::new();
        pit.write_control(0b00110100);
        pit.write_u8(PIT_COUNTER_0, 0x34);
        pit.write_u8(PIT_COUNTER_0, 0x12);

        // Latch count
        pit.write_control(0b00000000); // Latch counter 0

        // Read low then high
        let low = pit.read_u8(PIT_COUNTER_0);
        let high = pit.read_u8(PIT_COUNTER_0);

        assert_eq!(low, 0x34);
        assert_eq!(high, 0x12);
    }

    #[test]
    fn test_counter_tick_and_reload() {
        let mut pit = Pit::new();
        pit.write_control(0b00110100);

        // Set reload value to 4
        pit.write_u8(PIT_COUNTER_0, 0x04);
        pit.write_u8(PIT_COUNTER_0, 0x00);

        assert_eq!(pit.counters[0].count, 4);

        // Tick 4 times should wrap
        assert!(!pit.counters[0].tick()); // 4 -> 3
        assert!(!pit.counters[0].tick()); // 3 -> 2
        assert!(!pit.counters[0].tick()); // 2 -> 1
        assert!(pit.counters[0].tick()); // 1 -> 0 -> 4 (reload)

        assert_eq!(pit.counters[0].count, 4);
    }

    #[test]
    fn test_counter_zero_means_65536() {
        let mut pit = Pit::new();
        pit.write_control(0b00110100); // Counter 0, low+high, mode 2

        // Write reload value of 0 (which means 65536)
        pit.write_u8(PIT_COUNTER_0, 0x00);
        pit.write_u8(PIT_COUNTER_0, 0x00);

        assert_eq!(pit.counters[0].count, 0);
        assert_eq!(pit.counters[0].reload_value, 0);

        // First tick should NOT fire - count 0 wraps to 0xFFFF
        assert!(!pit.counters[0].tick());
        assert_eq!(pit.counters[0].count, 0xFFFF);

        // Tick down to 1
        for _ in 0..0xFFFE {
            assert!(!pit.counters[0].tick());
        }
        assert_eq!(pit.counters[0].count, 1);

        // Final tick should fire and reload to 0
        assert!(pit.counters[0].tick());
        assert_eq!(pit.counters[0].count, 0);

        // Next tick wraps to 0xFFFF again (starting new 65536 count)
        assert!(!pit.counters[0].tick());
        assert_eq!(pit.counters[0].count, 0xFFFF);
    }

    #[test]
    fn test_irq0_generation() {
        let mut pit = Pit::new();
        let mut pic = Pic::new(0x08);
        pic.set_imr(0x00); // Unmask all

        // Configure counter 0 for short interval
        pit.write_control(0b00110100);
        pit.write_u8(PIT_COUNTER_0, 0x04); // Reload = 4
        pit.write_u8(PIT_COUNTER_0, 0x00);

        // Tick with enough cycles to trigger (4 PIT ticks * 4 CPU cycles = 16)
        pit.tick(16, &mut pic);

        // Should have raised IRQ0
        assert!(pic.intr_out());
    }
}
