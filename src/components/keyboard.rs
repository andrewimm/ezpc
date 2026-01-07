//! IBM PC Keyboard - Simple scancode buffer
//!
//! The Keyboard is a simple data structure that buffers scancodes from the GUI.
//! It does not implement IoDevice - all I/O is handled by the PPI.

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

/// IBM PC Keyboard
///
/// Simple scancode buffer that receives input from the GUI.
/// The PPI pulls scancodes from this buffer during its tick().
pub struct Keyboard {
    /// Shared queue of keyboard scancodes from GUI
    scancode_queue: Arc<RwLock<VecDeque<u8>>>,
}

impl Keyboard {
    /// Create a new keyboard with the given scancode queue
    ///
    /// # Arguments
    /// * `scancode_queue` - Shared queue for receiving scancodes from GUI
    pub fn new(scancode_queue: Arc<RwLock<VecDeque<u8>>>) -> Self {
        Self { scancode_queue }
    }

    /// Pop the next scancode from the queue
    ///
    /// Returns the next available scancode, or None if queue is empty.
    pub fn pop_scancode(&mut self) -> Option<u8> {
        self.scancode_queue.write().ok()?.pop_front()
    }

    /// Check if there are scancodes available
    pub fn has_scancode(&self) -> bool {
        self.scancode_queue
            .read()
            .map(|q| !q.is_empty())
            .unwrap_or(false)
    }

    /// Reset the keyboard
    ///
    /// Clears the queue and pushes the 0xAA self-test success code.
    /// Called by PPI when keyboard reset sequence completes.
    pub fn reset(&mut self) {
        if let Ok(mut queue) = self.scancode_queue.write() {
            queue.clear();
            queue.push_back(0xAA); // Self-test passed
        }
    }

    /// Get the keyboard scancode queue for GUI integration
    pub fn scancode_queue(&self) -> Arc<RwLock<VecDeque<u8>>> {
        self.scancode_queue.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_new() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let kbd = Keyboard::new(queue);
        assert!(!kbd.has_scancode());
    }

    #[test]
    fn test_keyboard_pop_scancode() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue.clone());

        // Queue is empty
        assert_eq!(kbd.pop_scancode(), None);

        // Add scancodes
        queue.write().unwrap().push_back(0x1E);
        queue.write().unwrap().push_back(0x9E);

        // Pop in order
        assert_eq!(kbd.pop_scancode(), Some(0x1E));
        assert_eq!(kbd.pop_scancode(), Some(0x9E));
        assert_eq!(kbd.pop_scancode(), None);
    }

    #[test]
    fn test_keyboard_has_scancode() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let kbd = Keyboard::new(queue.clone());

        assert!(!kbd.has_scancode());

        queue.write().unwrap().push_back(0x1E);
        assert!(kbd.has_scancode());
    }

    #[test]
    fn test_keyboard_reset() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let mut kbd = Keyboard::new(queue.clone());

        // Add some scancodes
        queue.write().unwrap().push_back(0x1E);
        queue.write().unwrap().push_back(0x30);
        queue.write().unwrap().push_back(0x2E);

        // Reset clears queue and adds 0xAA
        kbd.reset();

        // Should only have 0xAA
        assert_eq!(kbd.pop_scancode(), Some(0xAA));
        assert_eq!(kbd.pop_scancode(), None);
    }

    #[test]
    fn test_keyboard_scancode_queue() {
        let queue = Arc::new(RwLock::new(VecDeque::new()));
        let kbd = Keyboard::new(queue.clone());

        // Should return the same queue
        let returned_queue = kbd.scancode_queue();
        returned_queue.write().unwrap().push_back(0x1E);

        assert_eq!(queue.read().unwrap().front(), Some(&0x1E));
    }
}
