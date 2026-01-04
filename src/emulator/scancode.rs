//! IBM PC Keyboard Scancode Set 1 Conversion
//!
//! Converts winit keyboard events to IBM PC XT scancode set 1.
//!
//! In scancode set 1:
//! - Make codes are sent when a key is pressed
//! - Break codes are sent when a key is released (make code | 0x80)

use winit::keyboard::{KeyCode, PhysicalKey};

/// Convert a winit physical key to IBM PC Scancode Set 1
///
/// Returns the make code (press) scancode. For break codes (release),
/// OR the result with 0x80.
pub fn physical_key_to_scancode(key: PhysicalKey) -> Option<u8> {
    match key {
        PhysicalKey::Code(code) => keycode_to_scancode(code),
        _ => None,
    }
}

/// Convert a winit KeyCode to IBM PC Scancode Set 1 make code
fn keycode_to_scancode(code: KeyCode) -> Option<u8> {
    use KeyCode::*;

    Some(match code {
        // Numbers row
        Digit1 => 0x02,
        Digit2 => 0x03,
        Digit3 => 0x04,
        Digit4 => 0x05,
        Digit5 => 0x06,
        Digit6 => 0x07,
        Digit7 => 0x08,
        Digit8 => 0x09,
        Digit9 => 0x0A,
        Digit0 => 0x0B,

        // Top row
        KeyQ => 0x10,
        KeyW => 0x11,
        KeyE => 0x12,
        KeyR => 0x13,
        KeyT => 0x14,
        KeyY => 0x15,
        KeyU => 0x16,
        KeyI => 0x17,
        KeyO => 0x18,
        KeyP => 0x19,

        // Home row
        KeyA => 0x1E,
        KeyS => 0x1F,
        KeyD => 0x20,
        KeyF => 0x21,
        KeyG => 0x22,
        KeyH => 0x23,
        KeyJ => 0x24,
        KeyK => 0x25,
        KeyL => 0x26,

        // Bottom row
        KeyZ => 0x2C,
        KeyX => 0x2D,
        KeyC => 0x2E,
        KeyV => 0x2F,
        KeyB => 0x30,
        KeyN => 0x31,
        KeyM => 0x32,

        // Special keys
        Escape => 0x01,
        Backspace => 0x0E,
        Tab => 0x0F,
        Enter => 0x1C,
        Space => 0x39,

        // Modifiers
        ControlLeft => 0x1D,
        ShiftLeft => 0x2A,
        ShiftRight => 0x36,
        AltLeft => 0x38,
        CapsLock => 0x3A,

        // Function keys
        F1 => 0x3B,
        F2 => 0x3C,
        F3 => 0x3D,
        F4 => 0x3E,
        F5 => 0x3F,
        F6 => 0x40,
        F7 => 0x41,
        F8 => 0x42,
        F9 => 0x43,
        F10 => 0x44,

        // Arrow keys
        ArrowUp => 0x48,
        ArrowLeft => 0x4B,
        ArrowRight => 0x4D,
        ArrowDown => 0x50,

        // Punctuation
        Minus => 0x0C,        // -_
        Equal => 0x0D,        // =+
        BracketLeft => 0x1A,  // [{
        BracketRight => 0x1B, // ]}
        Semicolon => 0x27,    // ;:
        Quote => 0x28,        // '"
        Backquote => 0x29,    // `~
        Backslash => 0x2B,    // \|
        Comma => 0x33,        // ,<
        Period => 0x34,       // .>
        Slash => 0x35,        // /?

        // Numpad
        NumLock => 0x45,
        Numpad0 => 0x52,
        Numpad1 => 0x4F,
        Numpad2 => 0x50,
        Numpad3 => 0x51,
        Numpad4 => 0x4B,
        Numpad5 => 0x4C,
        Numpad6 => 0x4D,
        Numpad7 => 0x47,
        Numpad8 => 0x48,
        Numpad9 => 0x49,
        NumpadMultiply => 0x37,
        NumpadSubtract => 0x4A,
        NumpadAdd => 0x4E,
        NumpadDecimal => 0x53,

        // Keys we don't support yet
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_letters() {
        assert_eq!(keycode_to_scancode(KeyCode::KeyA), Some(0x1E));
        assert_eq!(keycode_to_scancode(KeyCode::KeyZ), Some(0x2C));
        assert_eq!(keycode_to_scancode(KeyCode::KeyM), Some(0x32));
    }

    #[test]
    fn test_numbers() {
        assert_eq!(keycode_to_scancode(KeyCode::Digit1), Some(0x02));
        assert_eq!(keycode_to_scancode(KeyCode::Digit0), Some(0x0B));
    }

    #[test]
    fn test_special_keys() {
        assert_eq!(keycode_to_scancode(KeyCode::Escape), Some(0x01));
        assert_eq!(keycode_to_scancode(KeyCode::Enter), Some(0x1C));
        assert_eq!(keycode_to_scancode(KeyCode::Space), Some(0x39));
    }

    #[test]
    fn test_function_keys() {
        assert_eq!(keycode_to_scancode(KeyCode::F1), Some(0x3B));
        assert_eq!(keycode_to_scancode(KeyCode::F10), Some(0x44));
    }

    #[test]
    fn test_arrow_keys() {
        assert_eq!(keycode_to_scancode(KeyCode::ArrowUp), Some(0x48));
        assert_eq!(keycode_to_scancode(KeyCode::ArrowDown), Some(0x50));
        assert_eq!(keycode_to_scancode(KeyCode::ArrowLeft), Some(0x4B));
        assert_eq!(keycode_to_scancode(KeyCode::ArrowRight), Some(0x4D));
    }

    #[test]
    fn test_modifiers() {
        assert_eq!(keycode_to_scancode(KeyCode::ShiftLeft), Some(0x2A));
        assert_eq!(keycode_to_scancode(KeyCode::ControlLeft), Some(0x1D));
        assert_eq!(keycode_to_scancode(KeyCode::AltLeft), Some(0x38));
    }
}
