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
}
