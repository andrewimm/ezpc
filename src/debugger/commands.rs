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

        // v-commands (vCont, vMustReplyEmpty, etc.)
        'v' => String::new(), // Not supported

        // p command (read single register)
        'p' => read_single_register(cpu, cmd),

        // P command (write single register)
        'P' => String::new(), // Not supported, use 'G' instead

        // D command (detach)
        'D' => {
            debugger.pause();
            "OK".to_string()
        }

        // Unsupported command - return empty string
        _ => String::new(),
    }
}

/// Return halt reason (SIGTRAP = signal 5)
fn halt_reason() -> String {
    "S05".to_string()
}

/// Read a single register: p<n>
/// Register numbers:
/// 0-7: AX, CX, DX, BX, SP, BP, SI, DI
/// 8: IP
/// 9: FLAGS
/// 10-13: CS, SS, DS, ES
/// 14-15: FS, GS (not on 8086)
fn read_single_register(cpu: &mut Cpu, cmd: &str) -> String {
    // Parse register number from p<n>
    let reg_num = match usize::from_str_radix(&cmd[1..], 16) {
        Ok(n) => n,
        Err(_) => return "E01".to_string(),
    };

    let value = match reg_num {
        // General purpose registers
        0..=7 => cpu.regs[reg_num],
        // IP
        8 => cpu.ip,
        // FLAGS
        9 => cpu.get_flags(),
        // CS
        10 => cpu.segments[1],
        // SS
        11 => cpu.segments[2],
        // DS
        12 => cpu.segments[3],
        // ES
        13 => cpu.segments[0],
        // FS, GS (not on 8086)
        14 | 15 => 0,
        _ => return "E01".to_string(),
    };

    // Return as 32-bit little-endian hex (upper 16 bits = 0 for 8086)
    format!("{:02x}{:02x}0000", value & 0xFF, (value >> 8) & 0xFF)
}

/// Read all registers and return as hex string
///
/// GDB expects registers in this order for i8086:
/// AX, CX, DX, BX, SP, BP, SI, DI, IP, FLAGS, CS, SS, DS, ES, FS, GS
/// Each register is 32-bit little-endian (8 hex chars)
/// For 8086's 16-bit registers, upper 16 bits are 0
fn read_all_registers(cpu: &mut Cpu) -> String {
    let mut result = String::new();

    // Helper to format 16-bit value as 32-bit little-endian
    let format_reg =
        |val: u16| -> String { format!("{:02x}{:02x}0000", val & 0xFF, (val >> 8) & 0xFF) };

    // General purpose registers (AX, CX, DX, BX, SP, BP, SI, DI)
    for i in 0..8 {
        result.push_str(&format_reg(cpu.regs[i]));
    }

    // IP (EIP in 32-bit, but only lower 16 bits used)
    result.push_str(&format_reg(cpu.ip));

    // FLAGS (EFLAGS in 32-bit)
    result.push_str(&format_reg(cpu.get_flags()));

    // Segment registers (CS, SS, DS, ES)
    result.push_str(&format_reg(cpu.segments[1])); // CS
    result.push_str(&format_reg(cpu.segments[2])); // SS
    result.push_str(&format_reg(cpu.segments[3])); // DS
    result.push_str(&format_reg(cpu.segments[0])); // ES

    // FS, GS (not on 8086)
    result.push_str("00000000");
    result.push_str("00000000");

    result
}

/// Write all registers from hex string
fn write_all_registers(cpu: &mut Cpu, cmd: &str) -> String {
    // Command format: G<hex-data>
    // Each register is 32-bit (8 hex chars) in little-endian
    let data = &cmd[1..]; // Skip 'G'

    if data.len() < 128 {
        // Need at least 16 registers * 4 bytes * 2 hex/byte = 128 chars
        return "E01".to_string();
    }

    let mut pos = 0;

    // Helper to parse 32-bit little-endian, return only lower 16 bits
    let parse_u32_as_u16 = |s: &str| -> Option<u16> {
        let byte0 = u8::from_str_radix(&s[0..2], 16).ok()?;
        let byte1 = u8::from_str_radix(&s[2..4], 16).ok()?;
        // Ignore upper 16 bits (bytes 2 and 3) for 8086
        Some((byte1 as u16) << 8 | byte0 as u16)
    };

    // General purpose registers (8 registers)
    for i in 0..8 {
        if let Some(val) = parse_u32_as_u16(&data[pos..pos + 8]) {
            cpu.regs[i] = val;
        }
        pos += 8;
    }

    // IP
    if let Some(val) = parse_u32_as_u16(&data[pos..pos + 8]) {
        cpu.ip = val;
    }
    pos += 8;

    // FLAGS (skip for now - lazy flag system makes this complex)
    pos += 8;

    // Segment registers (CS, SS, DS, ES)
    if let Some(val) = parse_u32_as_u16(&data[pos..pos + 8]) {
        cpu.segments[1] = val; // CS
    }
    pos += 8;

    if let Some(val) = parse_u32_as_u16(&data[pos..pos + 8]) {
        cpu.segments[2] = val; // SS
    }
    pos += 8;

    if let Some(val) = parse_u32_as_u16(&data[pos..pos + 8]) {
        cpu.segments[3] = val; // DS
    }
    pos += 8;

    if let Some(val) = parse_u32_as_u16(&data[pos..pos + 8]) {
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
