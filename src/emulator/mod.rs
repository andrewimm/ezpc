//! Emulator state and coordination
//!
//! This module manages the overall emulator state, including CPU, memory,
//! and rendering components.

use crate::cpu::Cpu;
use crate::memory::MemoryBus;
use std::time::{Duration, Instant};

pub mod graphics;
use graphics::FramebufferRenderer;

/// Main emulator state
pub struct EmulatorState {
    cpu: Cpu,
    memory: MemoryBus,
    renderer: FramebufferRenderer,
    last_frame_time: Instant,
    target_frame_duration: Duration,
}

impl EmulatorState {
    /// Create a new emulator state
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            cpu: Cpu::new(),
            memory: MemoryBus::new(),
            renderer: FramebufferRenderer::new(device, queue, surface_format),
            last_frame_time: Instant::now(),
            target_frame_duration: Duration::from_micros(16667), // 60 FPS (~16.67ms)
        }
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
