//! Emulator state and coordination
//!
//! This module manages the overall emulator state, including CPU, memory,
//! and rendering components.

use crate::components::keyboard::Keyboard;
use crate::cpu::Cpu;
use crate::memory::MemoryBus;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub mod graphics;
pub mod scancode;

use graphics::FramebufferRenderer;

/// Main emulator state
pub struct EmulatorState {
    cpu: Cpu,
    memory: MemoryBus,
    renderer: FramebufferRenderer,
    last_frame_time: Instant,
    target_frame_duration: Duration,
    /// Keyboard scancode queue (shared with windowing system)
    scancode_queue: Arc<RwLock<VecDeque<u8>>>,
}

impl EmulatorState {
    /// Create a new emulator state
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        rom_data: Option<Vec<u8>>,
    ) -> Self {
        let mut memory = MemoryBus::new();

        // Load ROM if provided
        if let Some(rom) = rom_data {
            memory.load_rom(&rom);
        }

        // Create keyboard queue and register keyboard controller
        let scancode_queue = Arc::new(RwLock::new(VecDeque::new()));
        let keyboard = Keyboard::new(scancode_queue.clone());
        memory.register_io_device(Box::new(keyboard));

        // Create and reset CPU to initialize reset vector (CS=0xF000, IP=0xFFF0)
        let mut cpu = Cpu::new();
        cpu.reset();

        Self {
            cpu,
            memory,
            renderer: FramebufferRenderer::new(device, queue, surface_format),
            last_frame_time: Instant::now(),
            target_frame_duration: Duration::from_micros(16667), // 60 FPS (~16.67ms)
            scancode_queue,
        }
    }

    /// Get a reference to the keyboard scancode queue
    ///
    /// The windowing system can use this to push scancodes when keys are pressed.
    pub fn scancode_queue(&self) -> Arc<RwLock<VecDeque<u8>>> {
        self.scancode_queue.clone()
    }

    /// Update emulator state for one frame
    pub fn update(&mut self) {
        let elapsed = self.last_frame_time.elapsed();

        // Step CPU and update peripherals with cycle counts
        // For now, execute a fixed number of instructions per frame
        const INSTRUCTIONS_PER_FRAME: usize = 1000; // Placeholder

        for _ in 0..INSTRUCTIONS_PER_FRAME {
            let cycles = self.cpu.step(&mut self.memory);
            self.memory.tick(cycles);
        }

        // Sleep if we're under the frame budget
        if elapsed < self.target_frame_duration {
            std::thread::sleep(self.target_frame_duration - elapsed);
        }

        self.last_frame_time = Instant::now();
    }

    /// Render current frame to surface
    pub fn render(&mut self, surface_texture: &wgpu::SurfaceTexture) {
        // Get mutable access to framebuffer
        let framebuffer = self.renderer.framebuffer_mut();

        // Let MDA render its text mode to the framebuffer
        self.memory.mda().render_to_framebuffer(framebuffer);

        // Render framebuffer to surface
        self.renderer.render(surface_texture);
    }
}
