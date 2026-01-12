//! NEC μPD765A / Intel 8272A Floppy Disk Controller
//!
//! The FDC handles floppy disk operations in the IBM PC.
//! It uses DMA channel 2 for data transfers and IRQ6 for interrupts.
//!
//! ## I/O Ports
//! - 0x3F2: Digital Output Register (DOR) - motor/drive select/reset
//! - 0x3F4: Main Status Register (MSR) - controller status (read-only)
//! - 0x3F5: Data Register - command/result/data FIFO
//! - 0x3F7: Digital Input Register (DIR, read) / Config Control (CCR, write)

use crate::components::dma::DmaCapable;
use crate::components::pic::Pic;
use crate::io::IoDevice;
use std::collections::VecDeque;
use std::ops::RangeInclusive;

// =============================================================================
// Constants
// =============================================================================

/// FDC I/O port base
const FDC_PORT_BASE: u16 = 0x3F0;
const FDC_PORT_END: u16 = 0x3F7;

/// Individual port offsets
const FDC_DOR: u16 = 0x3F2; // Digital Output Register
const FDC_MSR: u16 = 0x3F4; // Main Status Register
const FDC_DATA: u16 = 0x3F5; // Data Register (FIFO)
const FDC_DIR: u16 = 0x3F7; // Digital Input Register / CCR

/// MSR bit definitions
const MSR_RQM: u8 = 0x80; // Request for Master - ready for transfer
const MSR_DIO: u8 = 0x40; // Data direction: 0=CPU→FDC, 1=FDC→CPU
const MSR_NDM: u8 = 0x20; // Non-DMA mode
const MSR_CB: u8 = 0x10; // Controller Busy

/// DOR bit definitions
const DOR_DRIVE_SEL_MASK: u8 = 0x03; // Drive select bits 0-1
const DOR_RESET: u8 = 0x04; // ~RESET: 0=reset, 1=normal
const DOR_DMA_ENABLE: u8 = 0x08; // DMA/IRQ enable
const DOR_MOTOR_A: u8 = 0x10; // Motor A enable
const DOR_MOTOR_B: u8 = 0x20; // Motor B enable
const DOR_MOTOR_C: u8 = 0x40; // Motor C enable
const DOR_MOTOR_D: u8 = 0x80; // Motor D enable

/// Command opcodes (lower 5 bits, some have MT/MF/SK modifiers in upper bits)
const CMD_SPECIFY: u8 = 0x03;
const CMD_SENSE_DRIVE_STATUS: u8 = 0x04;
const CMD_WRITE_DATA: u8 = 0x05;
const CMD_READ_DATA: u8 = 0x06;
const CMD_RECALIBRATE: u8 = 0x07;
const CMD_SENSE_INTERRUPT: u8 = 0x08;
const CMD_READ_ID: u8 = 0x0A;
const CMD_FORMAT_TRACK: u8 = 0x0D;
const CMD_SEEK: u8 = 0x0F;

/// Status Register 0 (ST0) bits
const ST0_IC_MASK: u8 = 0xC0; // Interrupt code
const ST0_IC_NORMAL: u8 = 0x00; // Normal termination
const ST0_IC_ABNORMAL: u8 = 0x40; // Abnormal termination
const ST0_IC_INVALID: u8 = 0x80; // Invalid command
const ST0_IC_READY_CHANGE: u8 = 0xC0; // Ready signal changed
const ST0_SE: u8 = 0x20; // Seek End
const ST0_EC: u8 = 0x10; // Equipment Check (track 0 not found)
const ST0_NR: u8 = 0x08; // Not Ready
const ST0_HD: u8 = 0x04; // Head address
const ST0_DS_MASK: u8 = 0x03; // Drive select

/// Status Register 1 (ST1) bits
const ST1_EN: u8 = 0x80; // End of cylinder
const ST1_DE: u8 = 0x20; // Data Error (CRC error)
const ST1_OR: u8 = 0x10; // Overrun
const ST1_ND: u8 = 0x04; // No Data (sector not found)
const ST1_NW: u8 = 0x02; // Not Writable
const ST1_MA: u8 = 0x01; // Missing Address Mark

/// Status Register 2 (ST2) bits
const ST2_CM: u8 = 0x40; // Control Mark (deleted data)
const ST2_DD: u8 = 0x20; // Data Error in Data Field
const ST2_WC: u8 = 0x10; // Wrong Cylinder
const ST2_SH: u8 = 0x08; // Scan Equal Hit
const ST2_SN: u8 = 0x04; // Scan Not Satisfied
const ST2_BC: u8 = 0x02; // Bad Cylinder
const ST2_MD: u8 = 0x01; // Missing Address Mark in Data Field

// =============================================================================
// Enums
// =============================================================================

/// FDC state machine phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdcPhase {
    /// Idle, waiting for command
    Idle,
    /// Receiving command bytes
    Command,
    /// Executing operation (seek, read, write, etc.)
    Execution,
    /// Sending result bytes
    Result,
}

/// FDC command being executed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdcCommand {
    None,
    Specify,
    SenseDriveStatus,
    Recalibrate,
    SenseInterrupt,
    Seek,
    ReadData,
    WriteData,
    ReadId,
    FormatTrack,
    Invalid,
}

// =============================================================================
// DriveState
// =============================================================================

/// Per-drive state
#[derive(Debug, Clone)]
pub struct DriveState {
    /// Current cylinder (head position)
    pub cylinder: u8,
    /// Motor is running
    pub motor_on: bool,
    /// Disk change flag (set when disk is changed/ejected)
    pub disk_changed: bool,
}

impl DriveState {
    pub fn new() -> Self {
        Self {
            cylinder: 0,
            motor_on: false,
            disk_changed: true, // Start with disk changed (no disk)
        }
    }

    pub fn reset(&mut self) {
        self.cylinder = 0;
        // Motor state and disk_changed are not affected by controller reset
    }
}

impl Default for DriveState {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Fdc
// =============================================================================

/// NEC μPD765A Floppy Disk Controller
#[derive(Debug)]
pub struct Fdc {
    // State machine
    phase: FdcPhase,
    current_command: FdcCommand,

    // Command/result buffers
    command_buffer: Vec<u8>,
    command_bytes_expected: usize,
    result_buffer: Vec<u8>,
    result_index: usize,

    // Registers
    dor: u8,
    ccr: u8,

    // Drive state (4 drives max)
    drives: [DriveState; 4],

    // Current operation parameters
    head: u8,
    sector: u8,
    sector_size: u8,
    eot: u8, // End of track (last sector number)

    // DMA transfer state
    dma_pending: bool,
    transfer_buffer: Vec<u8>,
    transfer_index: usize,

    // Interrupt state
    irq_pending: bool,

    // Sense Interrupt queue: (ST0, cylinder) pairs
    // Generated by Seek/Recalibrate completion or reset
    pending_interrupts: VecDeque<(u8, u8)>,

    // Specify command parameters
    step_rate_time: u8,
    head_unload_time: u8,
    head_load_time: u8,
    non_dma_mode: bool,

    // Reset state tracking
    reset_pending: bool,
}

impl Fdc {
    /// Create a new FDC in reset state
    pub fn new() -> Self {
        Self {
            phase: FdcPhase::Idle,
            current_command: FdcCommand::None,
            command_buffer: Vec::with_capacity(9),
            command_bytes_expected: 0,
            result_buffer: Vec::with_capacity(7),
            result_index: 0,
            dor: 0, // Controller starts in reset (bit 2 = 0)
            ccr: 0,
            drives: [
                DriveState::new(),
                DriveState::new(),
                DriveState::new(),
                DriveState::new(),
            ],
            head: 0,
            sector: 0,
            sector_size: 2, // 512 bytes default
            eot: 0,
            dma_pending: false,
            transfer_buffer: Vec::new(),
            transfer_index: 0,
            irq_pending: false,
            pending_interrupts: VecDeque::new(),
            step_rate_time: 0,
            head_unload_time: 0,
            head_load_time: 0,
            non_dma_mode: false,
            reset_pending: false,
        }
    }

    /// Enter reset state (DOR bit 2 goes low)
    fn enter_reset(&mut self) {
        self.phase = FdcPhase::Idle;
        self.current_command = FdcCommand::None;
        self.command_buffer.clear();
        self.command_bytes_expected = 0;
        self.result_buffer.clear();
        self.result_index = 0;
        self.dma_pending = false;
        self.transfer_buffer.clear();
        self.transfer_index = 0;
        self.irq_pending = false;
        self.pending_interrupts.clear();
        self.reset_pending = true;
    }

    /// Complete reset sequence (DOR bit 2 goes high)
    fn complete_reset(&mut self) {
        // Reset drive positions
        for drive in &mut self.drives {
            drive.reset();
        }

        // Queue sense interrupt results for all 4 drives (reset polling)
        self.pending_interrupts.clear();
        for drive_num in 0..4u8 {
            // ST0: Ready changed (0xC0) + drive number
            let st0 = ST0_IC_READY_CHANGE | drive_num;
            self.pending_interrupts.push_back((st0, 0));
        }

        self.irq_pending = true;
        self.reset_pending = false;
    }

    /// Build the Main Status Register value
    fn build_msr(&self) -> u8 {
        let mut msr = 0u8;

        // Check if controller is in reset
        if (self.dor & DOR_RESET) == 0 {
            return 0; // Controller in reset, not ready
        }

        match self.phase {
            FdcPhase::Idle => {
                // Ready to receive command
                msr |= MSR_RQM;
            }
            FdcPhase::Command => {
                // Ready to receive more command bytes
                msr |= MSR_RQM;
            }
            FdcPhase::Execution => {
                // Busy executing
                msr |= MSR_CB;
                if self.non_dma_mode && self.dma_pending {
                    msr |= MSR_RQM | MSR_NDM;
                    // Set DIO based on transfer direction
                    if self.current_command == FdcCommand::ReadData {
                        msr |= MSR_DIO; // FDC → CPU
                    }
                }
            }
            FdcPhase::Result => {
                // Ready to send result bytes
                msr |= MSR_RQM | MSR_DIO | MSR_CB;
            }
        }

        // Set drive busy bits (bits 0-3)
        // For simplicity, we mark the selected drive as busy during execution
        if self.phase == FdcPhase::Execution {
            let drive = self.dor & DOR_DRIVE_SEL_MASK;
            msr |= 1 << drive;
        }

        msr
    }

    /// Get the number of command bytes expected for a given opcode
    fn command_length(opcode: u8) -> usize {
        // Extract base command (lower 5 bits)
        let base_cmd = opcode & 0x1F;

        match base_cmd {
            CMD_SPECIFY => 3,
            CMD_SENSE_DRIVE_STATUS => 2,
            CMD_RECALIBRATE => 2,
            CMD_SENSE_INTERRUPT => 1,
            CMD_SEEK => 3,
            CMD_READ_DATA | CMD_WRITE_DATA => 9,
            CMD_READ_ID => 2,
            CMD_FORMAT_TRACK => 6,
            _ => 1, // Invalid commands are 1 byte
        }
    }

    /// Handle a command byte written to the data register
    fn write_command_byte(&mut self, value: u8) {
        if self.phase == FdcPhase::Idle {
            // First byte - determine command
            self.command_buffer.clear();
            self.command_buffer.push(value);
            self.command_bytes_expected = Self::command_length(value);

            if self.command_bytes_expected == 1 {
                // Single-byte command, execute immediately
                self.execute_command();
            } else {
                self.phase = FdcPhase::Command;
            }
        } else if self.phase == FdcPhase::Command {
            self.command_buffer.push(value);

            if self.command_buffer.len() >= self.command_bytes_expected {
                self.execute_command();
            }
        }
    }

    /// Execute the buffered command
    fn execute_command(&mut self) {
        let opcode = self.command_buffer[0];
        let base_cmd = opcode & 0x1F;

        match base_cmd {
            CMD_SPECIFY => self.cmd_specify(),
            CMD_SENSE_DRIVE_STATUS => self.cmd_sense_drive_status(),
            CMD_RECALIBRATE => self.cmd_recalibrate(),
            CMD_SENSE_INTERRUPT => self.cmd_sense_interrupt(),
            CMD_SEEK => self.cmd_seek(),
            CMD_READ_DATA => self.cmd_read_data(),
            CMD_WRITE_DATA => self.cmd_write_data(),
            CMD_READ_ID => self.cmd_read_id(),
            CMD_FORMAT_TRACK => self.cmd_format_track(),
            _ => self.cmd_invalid(),
        }
    }

    /// Specify command (0x03): Set timing parameters
    fn cmd_specify(&mut self) {
        // Byte 0: Command (0x03)
        // Byte 1: SRT (bits 7-4) | HUT (bits 3-0)
        // Byte 2: HLT (bits 7-1) | ND (bit 0)

        let byte1 = self.command_buffer.get(1).copied().unwrap_or(0);
        let byte2 = self.command_buffer.get(2).copied().unwrap_or(0);

        self.step_rate_time = (byte1 >> 4) & 0x0F;
        self.head_unload_time = byte1 & 0x0F;
        self.head_load_time = (byte2 >> 1) & 0x7F;
        self.non_dma_mode = (byte2 & 0x01) != 0;

        // No result phase for Specify
        self.phase = FdcPhase::Idle;
        self.current_command = FdcCommand::None;
    }

    /// Sense Drive Status command (0x04): Get drive status
    fn cmd_sense_drive_status(&mut self) {
        // Byte 0: Command (0x04)
        // Byte 1: HD (bit 2) | DS1,DS0 (bits 1-0)

        let byte1 = self.command_buffer.get(1).copied().unwrap_or(0);
        let drive = (byte1 & 0x03) as usize;
        let head = (byte1 >> 2) & 0x01;

        // Build Status Register 3
        let mut st3 = (drive as u8) | (head << 2);

        // Add status bits
        if self.drives[drive].cylinder == 0 {
            st3 |= 0x10; // Track 0
        }
        // Two-sided drive
        st3 |= 0x08;
        // Write protected (always report not protected for now)
        // Ready
        st3 |= 0x20;

        self.result_buffer.clear();
        self.result_buffer.push(st3);
        self.result_index = 0;
        self.phase = FdcPhase::Result;
        self.current_command = FdcCommand::SenseDriveStatus;
    }

    /// Recalibrate command (0x07): Move head to track 0
    fn cmd_recalibrate(&mut self) {
        // Byte 0: Command (0x07)
        // Byte 1: DS1,DS0 (bits 1-0)

        let byte1 = self.command_buffer.get(1).copied().unwrap_or(0);
        let drive = (byte1 & 0x03) as usize;

        // Move head to track 0
        self.drives[drive].cylinder = 0;

        // Queue interrupt with seek end status
        let st0 = ST0_IC_NORMAL | ST0_SE | (drive as u8);
        self.pending_interrupts.push_back((st0, 0));
        self.irq_pending = true;

        // No result phase - must use Sense Interrupt Status
        self.phase = FdcPhase::Idle;
        self.current_command = FdcCommand::None;
    }

    /// Sense Interrupt Status command (0x08): Get interrupt status
    fn cmd_sense_interrupt(&mut self) {
        self.result_buffer.clear();

        if let Some((st0, cylinder)) = self.pending_interrupts.pop_front() {
            self.result_buffer.push(st0);
            self.result_buffer.push(cylinder);
            self.result_index = 0;
            self.phase = FdcPhase::Result;

            // Clear IRQ if no more pending interrupts
            if self.pending_interrupts.is_empty() {
                self.irq_pending = false;
            }
        } else {
            // No pending interrupt - invalid command
            self.result_buffer.push(ST0_IC_INVALID);
            self.result_index = 0;
            self.phase = FdcPhase::Result;
        }

        self.current_command = FdcCommand::SenseInterrupt;
    }

    /// Seek command (0x0F): Move head to specified track
    fn cmd_seek(&mut self) {
        // Byte 0: Command (0x0F)
        // Byte 1: HD (bit 2) | DS1,DS0 (bits 1-0)
        // Byte 2: NCN (new cylinder number)

        let byte1 = self.command_buffer.get(1).copied().unwrap_or(0);
        let byte2 = self.command_buffer.get(2).copied().unwrap_or(0);

        let drive = (byte1 & 0x03) as usize;
        let new_cylinder = byte2;

        // Move head to new cylinder
        self.drives[drive].cylinder = new_cylinder;

        // Queue interrupt with seek end status
        let st0 = ST0_IC_NORMAL | ST0_SE | (drive as u8);
        self.pending_interrupts.push_back((st0, new_cylinder));
        self.irq_pending = true;

        // No result phase - must use Sense Interrupt Status
        self.phase = FdcPhase::Idle;
        self.current_command = FdcCommand::None;
    }

    /// Read Data command (0x06/0x46/0x66/0xC6/0xE6): Read sectors
    fn cmd_read_data(&mut self) {
        // Byte 0: MT MF SK 0 0110 (command with modifiers)
        // Byte 1: HD DS1 DS0
        // Byte 2: C (cylinder)
        // Byte 3: H (head)
        // Byte 4: R (sector)
        // Byte 5: N (sector size: 0=128, 1=256, 2=512, 3=1024)
        // Byte 6: EOT (end of track)
        // Byte 7: GPL (gap length)
        // Byte 8: DTL (data length if N=0)

        let byte1 = self.command_buffer.get(1).copied().unwrap_or(0);
        let drive = (byte1 & 0x03) as usize;
        self.head = (byte1 >> 2) & 0x01;

        let _cylinder = self.command_buffer.get(2).copied().unwrap_or(0);
        let _head_param = self.command_buffer.get(3).copied().unwrap_or(0);
        self.sector = self.command_buffer.get(4).copied().unwrap_or(1);
        self.sector_size = self.command_buffer.get(5).copied().unwrap_or(2);
        self.eot = self.command_buffer.get(6).copied().unwrap_or(9);

        // Without disk images, we return an error result
        // In a full implementation, this would:
        // 1. Read sectors from disk image into transfer_buffer
        // 2. Set dma_pending = true
        // 3. DMA controller would transfer data to memory
        // 4. On terminal count, set up result phase

        // For now, return "No Data" error (sector not found)
        self.setup_read_write_result(drive as u8, ST1_ND, 0);
    }

    /// Write Data command (0x05/0x45/0xC5): Write sectors
    fn cmd_write_data(&mut self) {
        // Same parameter format as Read Data

        let byte1 = self.command_buffer.get(1).copied().unwrap_or(0);
        let drive = (byte1 & 0x03) as usize;
        self.head = (byte1 >> 2) & 0x01;

        self.sector = self.command_buffer.get(4).copied().unwrap_or(1);
        self.sector_size = self.command_buffer.get(5).copied().unwrap_or(2);
        self.eot = self.command_buffer.get(6).copied().unwrap_or(9);

        // Without disk images, return error
        self.setup_read_write_result(drive as u8, ST1_ND, 0);
    }

    /// Read ID command (0x0A/0x4A): Read sector ID
    fn cmd_read_id(&mut self) {
        // Byte 0: MF 0 0 01010
        // Byte 1: HD DS1 DS0

        let byte1 = self.command_buffer.get(1).copied().unwrap_or(0);
        let drive = (byte1 & 0x03) as usize;
        self.head = (byte1 >> 2) & 0x01;

        // Without disk images, return error
        self.setup_read_write_result(drive as u8, ST1_MA, 0);
    }

    /// Format Track command (0x0D): Format a track
    fn cmd_format_track(&mut self) {
        // Byte 0: MF 0 01101
        // Byte 1: HD DS1 DS0
        // Byte 2: N (sector size)
        // Byte 3: SC (sectors per track)
        // Byte 4: GPL (gap length)
        // Byte 5: D (fill byte)

        let byte1 = self.command_buffer.get(1).copied().unwrap_or(0);
        let drive = (byte1 & 0x03) as usize;
        self.head = (byte1 >> 2) & 0x01;

        // Without disk images, return error (not writable)
        self.setup_read_write_result(drive as u8, ST1_NW, 0);
    }

    /// Handle invalid/unrecognized command
    fn cmd_invalid(&mut self) {
        self.result_buffer.clear();
        self.result_buffer.push(ST0_IC_INVALID);
        self.result_index = 0;
        self.phase = FdcPhase::Result;
        self.current_command = FdcCommand::Invalid;
    }

    /// Set up result phase for read/write commands
    fn setup_read_write_result(&mut self, drive: u8, st1: u8, st2: u8) {
        let cylinder = self.drives[drive as usize].cylinder;

        // Build ST0
        let st0 = if st1 != 0 || st2 != 0 {
            ST0_IC_ABNORMAL | (self.head << 2) | drive
        } else {
            ST0_IC_NORMAL | (self.head << 2) | drive
        };

        self.result_buffer.clear();
        self.result_buffer.push(st0);
        self.result_buffer.push(st1);
        self.result_buffer.push(st2);
        self.result_buffer.push(cylinder);
        self.result_buffer.push(self.head);
        self.result_buffer.push(self.sector);
        self.result_buffer.push(self.sector_size);

        self.result_index = 0;
        self.phase = FdcPhase::Result;
        self.irq_pending = true;
    }

    /// Read a result byte
    fn read_result_byte(&mut self) -> u8 {
        if self.result_index < self.result_buffer.len() {
            let byte = self.result_buffer[self.result_index];
            self.result_index += 1;

            if self.result_index >= self.result_buffer.len() {
                // All result bytes read, return to idle
                self.phase = FdcPhase::Idle;
                self.current_command = FdcCommand::None;
            }

            byte
        } else {
            0xFF
        }
    }

    /// Complete a DMA transfer (called when terminal count is reached)
    pub fn complete_transfer(&mut self) {
        // Set up result phase after DMA completes
        self.dma_pending = false;

        let drive = self.dor & DOR_DRIVE_SEL_MASK;
        self.setup_read_write_result(drive, 0, 0);
    }
}

impl Default for Fdc {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// IoDevice Implementation
// =============================================================================

impl IoDevice for Fdc {
    fn port_range(&self) -> RangeInclusive<u16> {
        FDC_PORT_BASE..=FDC_PORT_END
    }

    fn read_u8(&mut self, port: u16) -> u8 {
        match port {
            FDC_MSR => self.build_msr(),

            FDC_DATA => {
                if self.phase == FdcPhase::Result {
                    self.read_result_byte()
                } else {
                    0xFF
                }
            }

            FDC_DIR => {
                // Digital Input Register
                // Bit 7: Disk change (1 = disk changed/removed)
                let drive = (self.dor & DOR_DRIVE_SEL_MASK) as usize;
                if self.drives[drive].disk_changed {
                    0x80
                } else {
                    0x00
                }
            }

            _ => 0xFF,
        }
    }

    fn write_u8(&mut self, port: u16, value: u8) {
        match port {
            FDC_DOR => {
                let old_dor = self.dor;
                self.dor = value;

                // Check for reset transition (bit 2: 0->1 = exit reset)
                if (old_dor & DOR_RESET) == 0 && (value & DOR_RESET) != 0 {
                    // Exiting reset - complete reset sequence and generate interrupts
                    self.complete_reset();
                } else if (old_dor & DOR_RESET) != 0 && (value & DOR_RESET) == 0 {
                    // Entering reset - clear state
                    self.enter_reset();
                }

                // Update motor states
                self.drives[0].motor_on = (value & DOR_MOTOR_A) != 0;
                self.drives[1].motor_on = (value & DOR_MOTOR_B) != 0;
                self.drives[2].motor_on = (value & DOR_MOTOR_C) != 0;
                self.drives[3].motor_on = (value & DOR_MOTOR_D) != 0;
            }

            FDC_DATA => {
                if self.phase == FdcPhase::Idle || self.phase == FdcPhase::Command {
                    self.write_command_byte(value);
                }
            }

            FDC_DIR => {
                // Configuration Control Register (write)
                // Bits 1-0: Data rate select
                self.ccr = value & 0x03;
            }

            _ => {}
        }
    }

    fn tick(&mut self, _cycles: u16, pic: &mut Pic) {
        // Handle IRQ6 signaling
        if self.irq_pending && (self.dor & DOR_DMA_ENABLE) != 0 {
            pic.set_irq_level(6, true);
        } else {
            pic.set_irq_level(6, false);
        }
    }
}

// =============================================================================
// DmaCapable Implementation
// =============================================================================

impl DmaCapable for Fdc {
    fn dma_dreq(&self) -> bool {
        self.dma_pending && self.transfer_index < self.transfer_buffer.len()
    }

    fn dma_read_byte(&mut self) -> Option<u8> {
        // For Read Data: provide byte from disk to memory
        if self.dma_pending && self.transfer_index < self.transfer_buffer.len() {
            let byte = self.transfer_buffer[self.transfer_index];
            self.transfer_index += 1;
            Some(byte)
        } else {
            None
        }
    }

    fn dma_write_byte(&mut self, value: u8) {
        // For Write Data: receive byte from memory
        if self.dma_pending {
            self.transfer_buffer.push(value);
        }
    }

    fn dma_terminal_count(&mut self) {
        self.complete_transfer();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fdc_new() {
        let fdc = Fdc::new();
        assert_eq!(fdc.phase, FdcPhase::Idle);
        assert_eq!(fdc.dor, 0);
        assert!(!fdc.irq_pending);
    }

    #[test]
    fn test_msr_in_reset() {
        let fdc = Fdc::new();
        // Controller starts in reset (DOR bit 2 = 0)
        assert_eq!(fdc.dor & DOR_RESET, 0);
        assert_eq!(fdc.build_msr(), 0);
    }

    #[test]
    fn test_msr_after_reset_release() {
        let mut fdc = Fdc::new();
        // Release reset
        fdc.write_u8(FDC_DOR, DOR_RESET);
        // MSR should show RQM (ready)
        assert_eq!(fdc.build_msr() & MSR_RQM, MSR_RQM);
    }

    #[test]
    fn test_dor_motor_control() {
        let mut fdc = Fdc::new();
        fdc.write_u8(FDC_DOR, DOR_RESET | DOR_MOTOR_A | DOR_MOTOR_B);

        assert!(fdc.drives[0].motor_on);
        assert!(fdc.drives[1].motor_on);
        assert!(!fdc.drives[2].motor_on);
        assert!(!fdc.drives[3].motor_on);
    }

    #[test]
    fn test_reset_generates_interrupts() {
        let mut fdc = Fdc::new();
        // Start in reset
        assert_eq!(fdc.dor & DOR_RESET, 0);

        // Exit reset
        fdc.write_u8(FDC_DOR, DOR_RESET | DOR_DMA_ENABLE);

        // Should have 4 pending interrupts (one per drive)
        assert_eq!(fdc.pending_interrupts.len(), 4);
        assert!(fdc.irq_pending);
    }

    #[test]
    fn test_specify_command() {
        let mut fdc = Fdc::new();
        fdc.write_u8(FDC_DOR, DOR_RESET);

        // Send Specify command: SRT=8, HUT=0, HLT=1, ND=0
        fdc.write_u8(FDC_DATA, CMD_SPECIFY);
        fdc.write_u8(FDC_DATA, 0x80); // SRT=8, HUT=0
        fdc.write_u8(FDC_DATA, 0x02); // HLT=1, ND=0

        assert_eq!(fdc.phase, FdcPhase::Idle);
        assert_eq!(fdc.step_rate_time, 8);
        assert_eq!(fdc.head_unload_time, 0);
        assert_eq!(fdc.head_load_time, 1);
        assert!(!fdc.non_dma_mode);
    }

    #[test]
    fn test_recalibrate_command() {
        let mut fdc = Fdc::new();
        fdc.write_u8(FDC_DOR, DOR_RESET | DOR_DMA_ENABLE);

        // Clear reset interrupts
        fdc.pending_interrupts.clear();
        fdc.irq_pending = false;

        // Set drive 0 to cylinder 10
        fdc.drives[0].cylinder = 10;

        // Send Recalibrate command for drive 0
        fdc.write_u8(FDC_DATA, CMD_RECALIBRATE);
        fdc.write_u8(FDC_DATA, 0x00); // Drive 0

        // Head should be at track 0
        assert_eq!(fdc.drives[0].cylinder, 0);

        // Should have generated an interrupt
        assert!(fdc.irq_pending);
        assert_eq!(fdc.pending_interrupts.len(), 1);

        // Verify interrupt status
        let (st0, cyl) = fdc.pending_interrupts[0];
        assert_eq!(st0 & ST0_SE, ST0_SE); // Seek End
        assert_eq!(st0 & ST0_DS_MASK, 0); // Drive 0
        assert_eq!(cyl, 0); // Cylinder 0
    }

    #[test]
    fn test_seek_command() {
        let mut fdc = Fdc::new();
        fdc.write_u8(FDC_DOR, DOR_RESET | DOR_DMA_ENABLE);

        // Clear reset interrupts
        fdc.pending_interrupts.clear();
        fdc.irq_pending = false;

        // Send Seek command for drive 0 to cylinder 20
        fdc.write_u8(FDC_DATA, CMD_SEEK);
        fdc.write_u8(FDC_DATA, 0x00); // Drive 0
        fdc.write_u8(FDC_DATA, 20); // Cylinder 20

        // Head should be at track 20
        assert_eq!(fdc.drives[0].cylinder, 20);
        assert!(fdc.irq_pending);
    }

    #[test]
    fn test_sense_interrupt_status() {
        let mut fdc = Fdc::new();
        fdc.write_u8(FDC_DOR, DOR_RESET | DOR_DMA_ENABLE);

        // Should have 4 pending interrupts from reset
        assert_eq!(fdc.pending_interrupts.len(), 4);

        // Read all 4 with Sense Interrupt Status
        for i in 0..4 {
            fdc.write_u8(FDC_DATA, CMD_SENSE_INTERRUPT);
            assert_eq!(fdc.phase, FdcPhase::Result);

            let st0 = fdc.read_u8(FDC_DATA);
            let _cyl = fdc.read_u8(FDC_DATA);

            // Should be Ready Changed status for each drive
            assert_eq!(st0 & ST0_IC_MASK, ST0_IC_READY_CHANGE);
            assert_eq!(st0 & ST0_DS_MASK, i);
        }

        // No more pending interrupts
        assert!(!fdc.irq_pending);
        assert!(fdc.pending_interrupts.is_empty());
    }

    #[test]
    fn test_sense_interrupt_no_pending() {
        let mut fdc = Fdc::new();
        fdc.write_u8(FDC_DOR, DOR_RESET);
        fdc.pending_interrupts.clear();

        // Sense Interrupt with no pending interrupt
        fdc.write_u8(FDC_DATA, CMD_SENSE_INTERRUPT);

        // Should return invalid command status
        let st0 = fdc.read_u8(FDC_DATA);
        assert_eq!(st0, ST0_IC_INVALID);
    }

    #[test]
    fn test_read_data_no_disk() {
        let mut fdc = Fdc::new();
        fdc.write_u8(FDC_DOR, DOR_RESET | DOR_DMA_ENABLE);
        fdc.pending_interrupts.clear();

        // Send Read Data command (0xE6 = MT+MF+SK+READ)
        fdc.write_u8(FDC_DATA, 0xE6);
        fdc.write_u8(FDC_DATA, 0x00); // Drive 0, Head 0
        fdc.write_u8(FDC_DATA, 0); // Cylinder
        fdc.write_u8(FDC_DATA, 0); // Head
        fdc.write_u8(FDC_DATA, 1); // Sector
        fdc.write_u8(FDC_DATA, 2); // Sector size (512)
        fdc.write_u8(FDC_DATA, 9); // EOT
        fdc.write_u8(FDC_DATA, 0x1B); // GPL
        fdc.write_u8(FDC_DATA, 0xFF); // DTL

        // Should be in result phase with error
        assert_eq!(fdc.phase, FdcPhase::Result);

        // Read result bytes
        let st0 = fdc.read_u8(FDC_DATA);
        let st1 = fdc.read_u8(FDC_DATA);
        let _st2 = fdc.read_u8(FDC_DATA);

        // Should report abnormal termination with No Data
        assert_eq!(st0 & ST0_IC_MASK, ST0_IC_ABNORMAL);
        assert_eq!(st1 & ST1_ND, ST1_ND);
    }

    #[test]
    fn test_invalid_command() {
        let mut fdc = Fdc::new();
        fdc.write_u8(FDC_DOR, DOR_RESET);

        // Send invalid command
        fdc.write_u8(FDC_DATA, 0x1F); // Invalid opcode

        // Should return invalid status
        let st0 = fdc.read_u8(FDC_DATA);
        assert_eq!(st0, ST0_IC_INVALID);
    }

    #[test]
    fn test_dma_capable_no_transfer() {
        let fdc = Fdc::new();
        assert!(!fdc.dma_dreq());
    }

    #[test]
    fn test_dir_disk_changed() {
        let mut fdc = Fdc::new();
        fdc.write_u8(FDC_DOR, DOR_RESET);

        // Drives start with disk_changed = true
        let dir = fdc.read_u8(FDC_DIR);
        assert_eq!(dir & 0x80, 0x80);

        // Clear disk changed flag
        fdc.drives[0].disk_changed = false;
        let dir = fdc.read_u8(FDC_DIR);
        assert_eq!(dir & 0x80, 0x00);
    }
}
