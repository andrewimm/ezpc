//! GPU-accelerated framebuffer rendering
//!
//! This module provides the FramebufferRenderer which handles all wgpu
//! rendering operations for the emulator's display output.

/// WGSL shader for fullscreen quad rendering
const SHADER_SOURCE: &str = r#"
// Vertex shader - fullscreen triangle
@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(idx) / 2) * 4.0 - 1.0;
    let y = f32(i32(idx) % 2) * 4.0 - 1.0;
    return vec4<f32>(x, y, 0.0, 1.0);
}

// Fragment shader - sample framebuffer texture
@group(0) @binding(0) var fb_texture: texture_2d<f32>;
@group(0) @binding(1) var fb_sampler: sampler;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy / vec2<f32>(720.0, 350.0);
    return textureSample(fb_texture, fb_sampler, uv);
}
"#;

/// GPU-accelerated framebuffer renderer
pub struct FramebufferRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    framebuffer_texture: wgpu::Texture,
    framebuffer_bind_group: wgpu::BindGroup,
    framebuffer_data: Vec<u8>,
    width: u32,
    height: u32,
}

impl FramebufferRenderer {
    /// Create a new framebuffer renderer
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let width = 720;
        let height = 350;

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Framebuffer Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        // Create framebuffer texture
        let framebuffer_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Framebuffer Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let framebuffer_view =
            framebuffer_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create sampler with nearest-neighbor filtering
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Framebuffer Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Framebuffer Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create bind group
        let framebuffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Framebuffer Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&framebuffer_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Create render pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Framebuffer Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Framebuffer Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Allocate framebuffer data (RGBA8)
        let buffer_size = (width * height * 4) as usize;
        let framebuffer_data = vec![0u8; buffer_size];

        let mut renderer = Self {
            device,
            queue,
            render_pipeline,
            framebuffer_texture,
            framebuffer_bind_group,
            framebuffer_data,
            width,
            height,
        };

        // Initialize with checkerboard pattern
        renderer.init_checkerboard();

        renderer
    }

    /// Initialize framebuffer with checkerboard pattern
    fn init_checkerboard(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let is_white = ((x / 8) + (y / 8)) % 2 == 0;
                let color = if is_white { 0xFF } else { 0x80 };

                let idx = ((y * self.width + x) * 4) as usize;
                self.framebuffer_data[idx + 0] = color; // R
                self.framebuffer_data[idx + 1] = color; // G
                self.framebuffer_data[idx + 2] = color; // B
                self.framebuffer_data[idx + 3] = 0xFF; // A
            }
        }

        self.update_texture();
    }

    /// Update GPU texture from CPU buffer
    fn update_texture(&self) {
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.framebuffer_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.framebuffer_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.width * 4),
                rows_per_image: Some(self.height),
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Get mutable reference to framebuffer data for graphics card integration
    pub fn framebuffer_mut(&mut self) -> &mut [u8] {
        &mut self.framebuffer_data
    }

    /// Render framebuffer to surface texture
    pub fn render(&self, surface_texture: &wgpu::SurfaceTexture) {
        // Update GPU texture from CPU buffer
        self.update_texture();

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Framebuffer Render Encoder"),
            });

        // Get surface texture view
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Begin render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Framebuffer Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.framebuffer_bind_group, &[]);
            render_pass.draw(0..3, 0..1); // Draw fullscreen triangle
        }

        // Submit command buffer
        self.queue.submit(Some(encoder.finish()));
    }
}
