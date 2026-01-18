//! CPU test harness for instruction testing
//!
//! Provides a minimal environment for testing CPU instructions without
//! a full emulator. Contains just CPU state and memory bus.

use crate::cpu::Cpu;
use crate::memory::MemoryBus;

/// Test harness for CPU instruction testing
///
/// Provides a minimal environment with CPU and memory for testing
/// individual instructions or short sequences.
pub struct CpuHarness {
    /// CPU state
    pub cpu: Cpu,

    /// Memory bus
    pub mem: MemoryBus,
}

impl CpuHarness {
    /// Create a new test harness with initialized CPU and memory
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            mem: MemoryBus::new(),
        }
    }

    /// Load a program at the specified address
    ///
    /// Sets CS:IP to point to the loaded program.
    /// Clears the decode cache since the loaded code may overwrite previously cached instructions.
    pub fn load_program(&mut self, code: &[u8], segment: u16) {
        // Load code into memory at segment:0
        for (i, &byte) in code.iter().enumerate() {
            let addr = (segment as u32) * 16 + i as u32;
            self.mem.write_u8(addr, byte);
        }

        // Set CS:IP to point to the program
        self.cpu.segments[1] = segment; // CS
        self.cpu.ip = 0;

        // Clear decode cache - loaded code may overwrite previously cached instructions
        self.cpu.decode_cache.clear();
    }

    /// Execute one instruction
    ///
    /// Returns the number of cycles consumed by the instruction.
    pub fn step(&mut self) -> u16 {
        self.cpu.step(&mut self.mem)
    }

    /// Execute multiple instructions
    pub fn step_n(&mut self, n: usize) {
        for _ in 0..n {
            self.step();
        }
    }

    /// Reset CPU to initial state
    pub fn reset(&mut self) {
        self.cpu.reset();
    }
}

impl Default for CpuHarness {
    fn default() -> Self {
        Self::new()
    }
}
