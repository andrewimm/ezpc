//! GDB Remote Serial Protocol command handlers
//!
//! Implements the core GDB commands for debugging the emulated CPU.

use super::GdbDebugger;
use crate::cpu::Cpu;
use crate::memory::MemoryBus;

/// Handle a GDB command and return response
pub fn handle_command(
    cmd: &str,
    cpu: &mut Cpu,
    mem: &mut MemoryBus,
    debugger: &mut GdbDebugger,
) -> String {
    if cmd.is_empty() {
        return String::new();
    }

    match cmd.chars().next().unwrap() {
        // Halt reason
        '?' => halt_reason(),

        // Read all registers
        'g' => read_all_registers(cpu),

        // Write all registers
        'G' => write_all_registers(cpu, cmd),

        // Read memory: m<addr>,<len>
        'm' => read_memory(mem, cmd),

        // Write memory: M<addr>,<len>:bytes
        'M' => write_memory(mem, cmd),

        // Continue execution
        'c' => {
            debugger.resume();
            String::new() // No immediate response
        }

        // Single step
        's' => {
            debugger.single_step();
            String::new() // No immediate response, will send S05 after step
        }

        // Kill / detach
        'k' => {
            debugger.pause();
            "OK".to_string()
        }

        // Set thread for operations (Hg / Hc)
        'H' => "OK".to_string(), // We don't have threads

        // Query commands
        'q' => handle_query(cmd),

        // Insert breakpoint: Z0,<addr>,<kind>
        'Z' => insert_breakpoint(debugger, cmd),

        // Remove breakpoint: z0,<addr>,<kind>
        'z' => remove_breakpoint(debugger, cmd),

        // Unsupported command - return empty string
        _ => String::new(),
    }
}

/// Return halt reason (SIGTRAP = signal 5)
fn halt_reason() -> String {
    "S05".to_string()
}

/// Read all registers and return as hex string
///
/// GDB expects registers in this order for i8086:
/// AX, CX, DX, BX, SP, BP, SI, DI, IP, FLAGS, CS, SS, DS, ES, FS, GS
/// Each register is 16-bit little-endian (4 hex digits, bytes swapped)
fn read_all_registers(cpu: &mut Cpu) -> String {
    let mut result = String::new();

    // General purpose registers (AX, CX, DX, BX, SP, BP, SI, DI)
    for i in 0..8 {
        let reg = cpu.regs[i];
        result.push_str(&format!("{:02x}{:02x}", reg & 0xFF, reg >> 8));
    }

    // IP
    result.push_str(&format!("{:02x}{:02x}", cpu.ip & 0xFF, cpu.ip >> 8));

    // FLAGS
    let flags = cpu.get_flags();
    result.push_str(&format!("{:02x}{:02x}", flags & 0xFF, flags >> 8));

    // Segment registers (CS, SS, DS, ES)
    // Note: GDB expects CS, SS, DS, ES order (not ES, CS, SS, DS)
    result.push_str(&format!(
        "{:02x}{:02x}",
        cpu.segments[1] & 0xFF,
        cpu.segments[1] >> 8
    )); // CS
    result.push_str(&format!(
        "{:02x}{:02x}",
        cpu.segments[2] & 0xFF,
        cpu.segments[2] >> 8
    )); // SS
    result.push_str(&format!(
        "{:02x}{:02x}",
        cpu.segments[3] & 0xFF,
        cpu.segments[3] >> 8
    )); // DS
    result.push_str(&format!(
        "{:02x}{:02x}",
        cpu.segments[0] & 0xFF,
        cpu.segments[0] >> 8
    )); // ES

    // FS, GS (8086 doesn't have these, return 0)
    result.push_str("0000");
    result.push_str("0000");

    result
}

/// Write all registers from hex string
fn write_all_registers(cpu: &mut Cpu, cmd: &str) -> String {
    // Command format: G<hex-data>
    // Each register is 2 bytes (4 hex chars) in little-endian
    let data = &cmd[1..]; // Skip 'G'

    if data.len() < 80 {
        // Need at least 16 registers * 2 bytes * 2 hex/byte = 64 chars
        return "E01".to_string();
    }

    let mut pos = 0;

    // Helper to parse 16-bit little-endian
    let parse_u16 = |s: &str| -> Option<u16> {
        let low = u8::from_str_radix(&s[0..2], 16).ok()?;
        let high = u8::from_str_radix(&s[2..4], 16).ok()?;
        Some((high as u16) << 8 | low as u16)
    };

    // General purpose registers
    for i in 0..8 {
        if let Some(val) = parse_u16(&data[pos..pos + 4]) {
            cpu.regs[i] = val;
        }
        pos += 4;
    }

    // IP
    if let Some(val) = parse_u16(&data[pos..pos + 4]) {
        cpu.ip = val;
    }
    pos += 4;

    // FLAGS (skip for now - lazy flag system makes this complex)
    pos += 4;

    // Segment registers (CS, SS, DS, ES)
    if let Some(val) = parse_u16(&data[pos..pos + 4]) {
        cpu.segments[1] = val; // CS
    }
    pos += 4;

    if let Some(val) = parse_u16(&data[pos..pos + 4]) {
        cpu.segments[2] = val; // SS
    }
    pos += 4;

    if let Some(val) = parse_u16(&data[pos..pos + 4]) {
        cpu.segments[3] = val; // DS
    }
    pos += 4;

    if let Some(val) = parse_u16(&data[pos..pos + 4]) {
        cpu.segments[0] = val; // ES
    }

    "OK".to_string()
}

/// Read memory: m<addr>,<len>
/// addr is hex address (linear), len is hex length
fn read_memory(mem: &mut MemoryBus, cmd: &str) -> String {
    // Parse command: m<addr>,<len>
    let parts: Vec<&str> = cmd[1..].split(',').collect();
    if parts.len() != 2 {
        return "E01".to_string();
    }

    let addr = match u32::from_str_radix(parts[0], 16) {
        Ok(a) => a,
        Err(_) => return "E01".to_string(),
    };

    let len = match usize::from_str_radix(parts[1], 16) {
        Ok(l) => l,
        Err(_) => return "E01".to_string(),
    };

    // Read memory and format as hex
    let mut result = String::new();
    for i in 0..len {
        let byte_addr = addr + i as u32;
        let byte = mem.read_u8(byte_addr);
        result.push_str(&format!("{:02x}", byte));
    }

    result
}

/// Write memory: M<addr>,<len>:bytes
fn write_memory(mem: &mut MemoryBus, cmd: &str) -> String {
    // Parse command: M<addr>,<len>:<hex-bytes>
    let parts: Vec<&str> = cmd[1..].split(&[',' as char, ':' as char][..]).collect();
    if parts.len() != 3 {
        return "E01".to_string();
    }

    let addr = match u32::from_str_radix(parts[0], 16) {
        Ok(a) => a,
        Err(_) => return "E01".to_string(),
    };

    let len = match usize::from_str_radix(parts[1], 16) {
        Ok(l) => l,
        Err(_) => return "E01".to_string(),
    };

    let data = parts[2];

    // Write bytes to memory
    for i in 0..len {
        let hex_byte = &data[i * 2..i * 2 + 2];
        let byte = match u8::from_str_radix(hex_byte, 16) {
            Ok(b) => b,
            Err(_) => return "E01".to_string(),
        };

        let byte_addr = addr + i as u32;
        mem.write_u8(byte_addr, byte);
    }

    "OK".to_string()
}

/// Handle query commands (qXXX)
fn handle_query(cmd: &str) -> String {
    if cmd.starts_with("qSupported") {
        // Report our capabilities
        "PacketSize=4096".to_string()
    } else if cmd == "qAttached" {
        // We're attached to the process
        "1".to_string()
    } else if cmd == "qC" {
        // Current thread ID (we don't have threads, use 0)
        "QC0".to_string()
    } else if cmd == "qfThreadInfo" {
        // First thread in list
        "m0".to_string()
    } else if cmd == "qsThreadInfo" {
        // End of thread list
        "l".to_string()
    } else if cmd == "qTStatus" {
        // Trace status (not supported)
        String::new()
    } else if cmd == "qOffsets" {
        // Relocation offsets (none)
        String::new()
    } else if cmd.starts_with("qSymbol") {
        // Symbol lookup (not supported)
        "OK".to_string()
    } else {
        // Unknown query
        String::new()
    }
}

/// Insert breakpoint: Z0,<addr>,<kind>
fn insert_breakpoint(debugger: &mut GdbDebugger, cmd: &str) -> String {
    // Parse: Z0,<addr>,<kind>
    let parts: Vec<&str> = cmd[2..].split(',').collect();
    if parts.len() < 2 {
        return "E01".to_string();
    }

    let addr = match u32::from_str_radix(parts[0], 16) {
        Ok(a) => a,
        Err(_) => return "E01".to_string(),
    };

    debugger.add_breakpoint(addr);
    "OK".to_string()
}

/// Remove breakpoint: z0,<addr>,<kind>
fn remove_breakpoint(debugger: &mut GdbDebugger, cmd: &str) -> String {
    // Parse: z0,<addr>,<kind>
    let parts: Vec<&str> = cmd[2..].split(',').collect();
    if parts.len() < 2 {
        return "E01".to_string();
    }

    let addr = match u32::from_str_radix(parts[0], 16) {
        Ok(a) => a,
        Err(_) => return "E01".to_string(),
    };

    debugger.remove_breakpoint(addr);
    "OK".to_string()
}
