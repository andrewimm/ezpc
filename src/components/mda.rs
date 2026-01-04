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
        Self {
            vram: [0; 4096],
            cycle_count: 0,
            // 60 Hz refresh at 4.77 MHz ~= 79,500 cycles per frame
            update_threshold: 79_500,
            font_rom: Self::create_placeholder_font(),
            dirty: false,
        }
    }

    /// Create a placeholder font (all zeros for now)
    /// TODO: Replace with actual IBM MDA font ROM
    fn create_placeholder_font() -> [u8; 256 * 14] {
        let mut font = [0u8; 256 * 14];

        // For now, just create a simple pattern for ASCII printable characters
        // This will be replaced with actual MDA font data later
        for c in 32..127 {
            let base = c * 14;
            // Simple cross pattern for visibility
            font[base + 0] = 0b00011000;
            font[base + 1] = 0b00011000;
            font[base + 2] = 0b00011000;
            font[base + 3] = 0b11111111;
            font[base + 4] = 0b11111111;
            font[base + 5] = 0b00011000;
            font[base + 6] = 0b00011000;
            font[base + 7] = 0b00011000;
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
