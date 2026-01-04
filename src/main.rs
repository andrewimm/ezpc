//! EZPC - IBM PC Emulator
//!
//! Main entry point for the emulator application.

use ezpc::emulator::EmulatorState;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// Application state for winit event loop
struct App {
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    emulator: Option<EmulatorState>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            surface: None,
            emulator: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window
        let window_attrs = Window::default_attributes()
            .with_title("EZPC - IBM PC Emulator")
            .with_inner_size(winit::dpi::PhysicalSize::new(720, 350));

        let window = Arc::new(
            event_loop
                .create_window(window_attrs)
                .expect("Failed to create window"),
        );

        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        // Request adapter
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
        }))
        .expect("Failed to find suitable adapter");

        // Request device and queue
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Main Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ))
        .expect("Failed to create device");

        // Get surface capabilities and configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: 720,
            height: 350,
            present_mode: wgpu::PresentMode::Fifo, // VSync
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // Create emulator state
        let emulator = EmulatorState::new(device, queue, surface_format);

        // Store state
        self.window = Some(window);
        self.surface = Some(surface);
        self.emulator = Some(emulator);

        // Request initial redraw
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let (Some(emulator), Some(surface), Some(window)) =
                    (&mut self.emulator, &self.surface, &self.window)
                {
                    // Update emulator state (frame timing)
                    emulator.update();

                    // Get surface texture
                    match surface.get_current_texture() {
                        Ok(surface_texture) => {
                            // Render frame
                            emulator.render(&surface_texture);

                            // Present frame
                            surface_texture.present();

                            // Request next frame
                            window.request_redraw();
                        }
                        Err(wgpu::SurfaceError::Lost) => {
                            // Surface was lost, skip this frame
                            window.request_redraw();
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            eprintln!("Out of memory!");
                            event_loop.exit();
                        }
                        Err(e) => {
                            eprintln!("Surface error: {:?}", e);
                            window.request_redraw();
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() {
    // Create event loop
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    // Create and run app
    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .expect("Failed to run event loop");
}
