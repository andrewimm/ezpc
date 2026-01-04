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

        // TODO: Step CPU execution with cycle budget
        // For now, just maintain frame timing

        // Sleep if we're under the frame budget
        if elapsed < self.target_frame_duration {
            std::thread::sleep(self.target_frame_duration - elapsed);
        }

        self.last_frame_time = Instant::now();
    }

    /// Render current frame to surface
    pub fn render(&self, surface_texture: &wgpu::SurfaceTexture) {
        self.renderer.render(surface_texture);
    }
}
