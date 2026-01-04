//! GDB Remote Debugging Support
//!
//! Implements the GDB Remote Serial Protocol over a Unix socket.
//! Uses non-blocking I/O with a helper thread to avoid blocking emulation.

mod commands;
mod protocol;
mod socket;

use crate::cpu::Cpu;
use crate::memory::MemoryBus;
use protocol::format_packet;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

/// Debugger execution state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DebugState {
    /// Execute normally
    Running,
    /// Halted, waiting for GDB command
    Paused,
    /// Execute 1 instruction then pause
    SingleStep,
}

/// GDB Remote Debugger
pub struct GdbDebugger {
    /// Incoming packets from GDB client
    incoming_packets: Arc<RwLock<VecDeque<String>>>,

    /// Outgoing packets to GDB client
    outgoing_packets: Arc<RwLock<VecDeque<String>>>,

    /// Socket listener thread handle
    _socket_thread: JoinHandle<()>,

    /// Current execution state
    state: DebugState,

    /// Breakpoints (linear addresses: seg*16 + offset)
    breakpoints: Vec<u32>,

    /// Statistics
    packets_processed: usize,
}

impl GdbDebugger {
    /// Create new debugger and start socket listener
    pub fn new(socket_path: &str) -> Self {
        let incoming = Arc::new(RwLock::new(VecDeque::new()));
        let outgoing = Arc::new(RwLock::new(VecDeque::new()));

        let socket_thread = socket::spawn_socket_listener(
            socket_path.to_string(),
            incoming.clone(),
            outgoing.clone(),
        );

        Self {
            incoming_packets: incoming,
            outgoing_packets: outgoing,
            _socket_thread: socket_thread,
            state: DebugState::Paused, // Start paused, waiting for GDB
            breakpoints: Vec::new(),
            packets_processed: 0,
        }
    }

    /// Check if emulation is paused
    pub fn is_paused(&self) -> bool {
        self.state == DebugState::Paused
    }

    /// Check if in single-step mode
    pub fn is_single_stepping(&self) -> bool {
        self.state == DebugState::SingleStep
    }

    /// Pause execution
    pub fn pause(&mut self) {
        self.state = DebugState::Paused;
    }

    /// Resume execution
    pub fn resume(&mut self) {
        self.state = DebugState::Running;
    }

    /// Execute one instruction then pause
    pub fn single_step(&mut self) {
        self.state = DebugState::SingleStep;
    }

    /// Add breakpoint at linear address
    pub fn add_breakpoint(&mut self, addr: u32) {
        if !self.breakpoints.contains(&addr) {
            self.breakpoints.push(addr);
        }
    }

    /// Remove breakpoint at linear address
    pub fn remove_breakpoint(&mut self, addr: u32) {
        self.breakpoints.retain(|&a| a != addr);
    }

    /// Check if current IP matches a breakpoint
    pub fn check_breakpoint(&self, cpu: &Cpu) -> bool {
        let linear_addr = (cpu.segments[1] as u32 * 16) + cpu.ip as u32;
        self.breakpoints.contains(&linear_addr)
    }

    /// Send a packet to GDB client
    fn send_packet(&mut self, data: &str) {
        let packet = format_packet(data);
        self.outgoing_packets.write().unwrap().push_back(packet);
    }

    /// Send halt reason to GDB (SIGTRAP)
    pub fn send_halt_reason(&mut self) {
        self.send_packet("S05"); // Signal 5 = SIGTRAP
    }

    /// Process incoming GDB commands
    pub fn process_commands(&mut self, cpu: &mut Cpu, mem: &mut MemoryBus) {
        // Process all pending packets
        loop {
            let packet = {
                let mut queue = self.incoming_packets.write().unwrap();
                queue.pop_front()
            };

            let Some(packet) = packet else {
                break;
            };

            self.packets_processed += 1;

            eprintln!("GDB: Received command: {}", packet);

            // Check if this is a deferred-response command (s, c)
            let deferred = packet.starts_with('s') || packet.starts_with('c');

            // Handle command
            let response = commands::handle_command(&packet, cpu, mem, self);

            // Send response
            if !response.is_empty() {
                eprintln!("GDB: Sending response: {}", response);
                self.send_packet(&response);
            } else if !deferred {
                // Empty response for unsupported commands (but not for s/c)
                eprintln!("GDB: Empty response (not supported)");
                self.send_packet("");
            } else {
                // Deferred response (s/c) - will send S05 later
                eprintln!("GDB: Deferred response (will send halt reason after execution)");
            }
        }
    }

    /// Called after single-step instruction completes
    pub fn finish_single_step(&mut self) {
        if self.state == DebugState::SingleStep {
            self.pause();
            self.send_halt_reason();
        }
    }
}
