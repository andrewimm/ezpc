//! EZPC - IBM PC Emulator
//!
//! Main entry point for the emulator application.

use ezpc::components::floppy::FloppyDisk;
use ezpc::emulator::scancode::physical_key_to_scancode;
use ezpc::emulator::EmulatorState;
use std::path::Path;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// Application state for winit event loop
struct App {
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    emulator: Option<EmulatorState>,
    rom_data: Option<Vec<u8>>,
    gdb_socket_path: Option<String>,
    floppy_a: Option<FloppyDisk>,
    floppy_b: Option<FloppyDisk>,
}

impl App {
    fn new(
        rom_data: Option<Vec<u8>>,
        gdb_socket_path: Option<String>,
        floppy_a: Option<FloppyDisk>,
        floppy_b: Option<FloppyDisk>,
    ) -> Self {
        Self {
            window: None,
            surface: None,
            emulator: None,
            rom_data,
            gdb_socket_path,
            floppy_a,
            floppy_b,
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

        // Create emulator state with ROM data, GDB socket, and floppy disks
        let emulator = EmulatorState::with_floppies(
            device,
            queue,
            surface_format,
            self.rom_data.take(),
            self.gdb_socket_path.as_deref(),
            self.floppy_a.take(),
            self.floppy_b.take(),
        );

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
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                // Convert winit key to IBM PC scancode
                if let Some(make_code) = physical_key_to_scancode(key_event.physical_key) {
                    let scancode = match key_event.state {
                        ElementState::Pressed => make_code,         // Make code
                        ElementState::Released => make_code | 0x80, // Break code
                    };

                    // Push scancode to emulator queue
                    if let Some(emulator) = &self.emulator {
                        let queue = emulator.scancode_queue();
                        queue.write().unwrap().push_back(scancode);
                    }
                }
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
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();

    let mut gdb_socket_path: Option<String> = None;
    let mut rom_path: Option<String> = None;
    let mut floppy_a_path: Option<String> = None;
    let mut floppy_b_path: Option<String> = None;
    let mut writable = false;

    // Simple argument parser
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--gdb" => {
                // Next argument is the socket path
                if i + 1 < args.len() {
                    gdb_socket_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --gdb requires a socket path argument");
                    eprintln!("Usage: {} [OPTIONS] [ROM_FILE]", args[0]);
                    std::process::exit(1);
                }
            }
            "-a" | "--floppy-a" => {
                if i + 1 < args.len() {
                    floppy_a_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: {} requires a disk image path", args[i]);
                    std::process::exit(1);
                }
            }
            "-b" | "--floppy-b" => {
                if i + 1 < args.len() {
                    floppy_b_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: {} requires a disk image path", args[i]);
                    std::process::exit(1);
                }
            }
            "-w" | "--writable" => {
                writable = true;
                i += 1;
            }
            "--help" | "-h" => {
                println!("EZPC - IBM PC Emulator");
                println!();
                println!("Usage: {} [OPTIONS] [ROM_FILE]", args[0]);
                println!();
                println!("Options:");
                println!("  -a, --floppy-a <PATH>  Disk image for drive A: (first floppy)");
                println!("  -b, --floppy-b <PATH>  Disk image for drive B: (second floppy)");
                println!(
                    "  -w, --writable         Allow writes to disk images (default: read-only)"
                );
                println!("  --gdb <socket-path>    Enable GDB remote debugging on Unix socket");
                println!("  --help, -h             Show this help message");
                println!();
                println!("Supported disk formats: raw sector images (.img)");
                println!("  160KB (40x1x8), 180KB (40x1x9), 320KB (40x2x8), 360KB (40x2x9)");
                println!("  720KB (80x2x9), 1.2MB (80x2x15), 1.44MB (80x2x18)");
                println!();
                println!("Examples:");
                println!("  {} bios.rom", args[0]);
                println!("  {} -a dos.img bios.rom", args[0]);
                println!("  {} -a boot.img -w --gdb /tmp/ezpc.sock bios.rom", args[0]);
                std::process::exit(0);
            }
            arg if arg.starts_with('-') => {
                eprintln!("Error: Unknown option '{}'", arg);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
            arg => {
                // Assume it's the ROM file path
                rom_path = Some(arg.to_string());
                i += 1;
            }
        }
    }

    // Load ROM file if provided
    let rom_data = if let Some(path) = rom_path {
        match std::fs::read(&path) {
            Ok(data) => {
                println!("Loaded ROM: {} ({} bytes)", path, data.len());
                Some(data)
            }
            Err(e) => {
                eprintln!("Failed to load ROM file '{}': {}", path, e);
                std::process::exit(1);
            }
        }
    } else {
        println!("No ROM file provided, starting with empty ROM");
        None
    };

    // Load floppy disk images
    let floppy_a = if let Some(ref path) = floppy_a_path {
        match FloppyDisk::from_file(Path::new(path)) {
            Ok(mut disk) => {
                let geometry = disk.geometry();
                println!(
                    "Drive A: {} ({}x{}x{}, {} bytes)",
                    path,
                    geometry.cylinders,
                    geometry.heads,
                    geometry.sectors_per_track,
                    geometry.total_size()
                );
                if writable {
                    disk.set_write_protected(false);
                    println!("  (writable)");
                } else {
                    println!("  (read-only)");
                }
                Some(disk)
            }
            Err(e) => {
                eprintln!("Failed to load floppy A '{}': {}", path, e);
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    let floppy_b = if let Some(ref path) = floppy_b_path {
        match FloppyDisk::from_file(Path::new(path)) {
            Ok(mut disk) => {
                let geometry = disk.geometry();
                println!(
                    "Drive B: {} ({}x{}x{}, {} bytes)",
                    path,
                    geometry.cylinders,
                    geometry.heads,
                    geometry.sectors_per_track,
                    geometry.total_size()
                );
                if writable {
                    disk.set_write_protected(false);
                    println!("  (writable)");
                } else {
                    println!("  (read-only)");
                }
                Some(disk)
            }
            Err(e) => {
                eprintln!("Failed to load floppy B '{}': {}", path, e);
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    // Print GDB info if enabled
    if let Some(ref socket) = gdb_socket_path {
        println!("GDB remote debugging enabled on: {}", socket);
        println!("Connect with: gdb -ex 'target remote {}'", socket);
    }

    // Create event loop
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    // Create and run app
    let mut app = App::new(rom_data, gdb_socket_path, floppy_a, floppy_b);
    event_loop
        .run_app(&mut app)
        .expect("Failed to run event loop");
}
