//! Monochrome Display Adapter (MDA) emulation
//!
//! The MDA provides 80x25 text mode with:
//! - 4KB video RAM at 0xB0000-0xB0FFF
//! - 9x14 pixel characters
//! - 720x350 display resolution
//! - Monochrome green phosphor output

/// MDA (Monochrome Display Adapter)
pub struct Mda {
    /// Video RAM (4KB for 80x25 text mode, 2 bytes per cell)
    /// Each cell: byte 0 = character code, byte 1 = attribute
    vram: [u8; 4096],

    /// Cycle accumulator for periodic updates
    cycle_count: u64,

    /// Update threshold - regenerate framebuffer every N cycles
    /// (MDA refreshes at ~50-70Hz, we'll use 60Hz aligned with frame rate)
    update_threshold: u64,

    /// Font ROM data (256 characters × 14 rows × 1 byte)
    font_rom: [u8; 256 * 14],

    /// Dirty flag - set when VRAM is written
    dirty: bool,
}

impl Mda {
    /// Create a new MDA with blank VRAM and embedded font
    pub fn new() -> Self {
        let mut vram = [0; 4096];

        // HACK: Write "HELLO" to first 5 character cells for testing
        // TODO: Remove this once we verify MDA is working
        let hello = b"HELLO";
        for (i, &ch) in hello.iter().enumerate() {
            vram[i * 2] = ch; // Character code
            vram[i * 2 + 1] = 0x07; // Attribute: white on black
        }

        Self {
            vram,
            cycle_count: 0,
            // 60 Hz refresh at 4.77 MHz ~= 79,500 cycles per frame
            update_threshold: 79_500,
            font_rom: Self::load_font_rom(),
            dirty: false,
        }
    }

    /// Load the IBM MDA character ROM font
    ///
    /// The ROM file is 8KB (4KB used) organized in two 2KB banks:
    /// - Bank 0 (0x0000-0x07FF): Top 8 scan lines (0-7) for all 256 characters
    ///   - 0x0000-0x00FF: Scan line 0 for all chars
    ///   - 0x0100-0x01FF: Scan line 1 for all chars
    ///   - ... through scan line 7
    /// - Bank 1 (0x0800-0x0FFF): Bottom 8 scan lines (8-15) for all 256 characters
    ///   - 0x0800-0x08FF: Scan line 8 for all chars
    ///   - 0x0900-0x09FF: Scan line 9 for all chars
    ///   - ... through scan line 15
    ///
    /// MDA uses only the first 14 scan lines (0-13).
    fn load_font_rom() -> [u8; 256 * 14] {
        // Include the actual MDA character ROM at compile time
        const ROM_DATA: &[u8] = include_bytes!("../../roms/MDA_CHAR.bin");

        let mut font = [0u8; 256 * 14];

        // Extract from two-bank format and reorganize to character-major
        for char_idx in 0..256 {
            for scan_line in 0..14 {
                let rom_offset = if scan_line < 8 {
                    // First bank: lines 0-7
                    scan_line * 256 + char_idx
                } else {
                    // Second bank: lines 8-13 (at offset 0x0800)
                    0x0800 + (scan_line - 8) * 256 + char_idx
                };

                // Our font is organized: char 0's all scanlines, char 1's all scanlines, etc.
                let font_offset = char_idx * 14 + scan_line;

                font[font_offset] = ROM_DATA[rom_offset];
            }
        }

        font
    }

    /// Read from video RAM
    #[inline(always)]
    pub fn read_vram(&self, offset: u16) -> u8 {
        self.vram[offset as usize & 0xFFF]
    }

    /// Write to video RAM
    #[inline(always)]
    pub fn write_vram(&mut self, offset: u16, value: u8) {
        self.vram[offset as usize & 0xFFF] = value;
        self.dirty = true;
    }

    /// Update based on CPU cycles
    pub fn tick(&mut self, cycles: u16) {
        self.cycle_count += cycles as u64;

        // TODO: Implement periodic framebuffer regeneration
        // For now, just accumulate cycles
    }

    /// Read from MDA I/O port
    pub fn read_u8(&mut self, port: u16) -> u8 {
        match port {
            0x3BA => {
                // Status register
                // Bit 0: Horizontal retrace (not implemented)
                // Bit 3: Vertical retrace (toggle for now)
                if (self.cycle_count >> 10) & 1 == 0 {
                    0x08
                } else {
                    0x00
                }
            }
            _ => {
                // Other ports not implemented yet
                0xFF
            }
        }
    }

    /// Write to MDA I/O port
    pub fn write_u8(&mut self, port: u16, value: u8) {
        match port {
            0x3B8 => {
                // Mode control register
                // Not implemented yet
                let _ = value;
            }
            0x3B4 => {
                // CRTC index register
                // Not implemented yet
                let _ = value;
            }
            0x3B5 => {
                // CRTC data register
                // Not implemented yet
                let _ = value;
            }
            _ => {
                // Other ports ignored
            }
        }
    }

    /// Render the text mode display to an RGBA framebuffer
    ///
    /// Converts the 80x25 text cells into 720x350 pixels (9x14 per character)
    pub fn render_to_framebuffer(&self, framebuffer: &mut [u8]) {
        for row in 0..25 {
            for col in 0..80 {
                let cell_idx = row * 80 + col;
                let char_code = self.vram[cell_idx * 2];
                let attribute = self.vram[cell_idx * 2 + 1];

                // Render 9x14 character to framebuffer
                self.render_char(framebuffer, col, row, char_code, attribute);
            }
        }
    }

    /// Render a single character to the framebuffer
    fn render_char(
        &self,
        framebuffer: &mut [u8],
        col: usize,
        row: usize,
        char_code: u8,
        attribute: u8,
    ) {
        // Extract foreground intensity from attribute byte
        // Bit 3 = high intensity, bits 0-2 = color (7 = white for monochrome)
        let fg_intensity = if (attribute & 0x08) != 0 { 0xFF } else { 0xAA };
        let bg_intensity = 0x00; // Black background

        // Render each scan line of the character
        for scan_line in 0..14 {
            let font_byte = self.font_rom[char_code as usize * 14 + scan_line];
            let y = row * 14 + scan_line;

            // Render each pixel (9 pixels wide, 8 from font + 1 blank)
            for bit in 0..9 {
                let x = col * 9 + bit;
                let pixel_on = if bit < 8 {
                    (font_byte >> (7 - bit)) & 1 != 0
                } else {
                    false // 9th column usually blank
                };

                let color = if pixel_on { fg_intensity } else { bg_intensity };
                let idx = (y * 720 + x) * 4;
                framebuffer[idx + 0] = color; // R
                framebuffer[idx + 1] = color; // G
                framebuffer[idx + 2] = color; // B
                framebuffer[idx + 3] = 0xFF; // A
            }
        }
    }
}
