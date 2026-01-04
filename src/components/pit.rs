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
                self.reload_value = (self.reload_value & 0xFF00) | (value as u16);
                self.count = self.reload_value;
                self.null_count = false;
            }
            AccessMode::HighByteOnly => {
                self.reload_value = (self.reload_value & 0x00FF) | ((value as u16) << 8);
                self.count = self.reload_value;
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

        if self.count == 0 {
            // Reload and signal wrap
            self.count = self.reload_value;
            return true;
        } else {
            self.count -= 1;
            if self.count == 0 {
                self.count = self.reload_value;
                return true;
            }
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
}
