use crate::rendering::draw_commands::{DrawCommand, DrawCommandList};
use crate::game_math::Vec2;
use sdl2::video::Window;
use std::num::NonZero;
use wgpu::util::DeviceExt;

/// Simple 2D renderer with a libGDX-like API
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    window_width: u32,
    window_height: u32,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    cube_pos: [f32; 2],
    window_size: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

impl Renderer {
    /// Create a new renderer from an SDL2 window
    pub fn new(window: &Window, width: u32, height: u32) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("Initializing renderer...");
        
        // Initialize wgpu
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(window)?)?
        };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            },
        ))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f: &wgpu::TextureFormat| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        // Prefer VSync (Fifo) for smooth frame limiting, fallback to first available
        let present_mode = surface_caps
            .present_modes
            .iter()
            .copied()
            .find(|&mode| mode == wgpu::PresentMode::Fifo)
            .unwrap_or(surface_caps.present_modes[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
        });

        // Create uniform buffer and bind group
        let uniform_buffer_size = std::mem::size_of::<Uniforms>() as wgpu::BufferAddress;
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: uniform_buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZero::new(std::mem::size_of::<Uniforms>() as u64),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create render pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Disable culling for 2D rendering
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        // Create vertex buffer for cube (will be reused)
        let cube_vertices = Renderer::create_cube_vertices(25.0); // Default size
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(&cube_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        log::info!("Renderer initialized successfully");

        Ok(Self {
            device,
            queue,
            surface,
            config,
            render_pipeline,
            vertex_buffer,
            uniform_buffer,
            bind_group,
            window_width: width,
            window_height: height,
        })
    }

    /// Begin a new frame. Returns a Frame that must be used for all draw calls.
    pub fn begin(&mut self) -> Result<Frame, Box<dyn std::error::Error>> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        Ok(Frame {
            renderer: self,
            view,
            encoder: Some(encoder),
            output: Some(output),
            render_pass_started: false,
        })
    }
}

/// A frame handle for drawing. Created by Renderer::begin() and consumed by Frame::end()
pub struct Frame<'a> {
    renderer: &'a mut Renderer,
    view: wgpu::TextureView,
    encoder: Option<wgpu::CommandEncoder>,
    output: Option<wgpu::SurfaceTexture>,
    render_pass_started: bool,
}

impl<'a> Frame<'a> {
    /// Draw a list of draw commands
    pub fn draw_commands(&mut self, commands: &DrawCommandList) {
        for command in commands.iter() {
            match command {
                DrawCommand::Cube { position, size } => {
                    self.draw_cube_internal(*position, *size);
                }
            }
        }
    }

    /// Draw a cube (square) at the specified position and size
    /// 
    /// # Arguments
    /// * `position` - Position (top-left corner) in pixels
    /// * `size` - Size of the cube in pixels
    pub fn draw_cube(&mut self, position: Vec2, size: f32) {
        self.draw_cube_internal(position, size);
    }

    fn draw_cube_internal(&mut self, position: Vec2, size: f32) {
        let encoder = self.encoder.as_mut().expect("Encoder should exist");
        
        // Start render pass on first draw call (clear screen once)
        if !self.render_pass_started {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.2,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            
            // Set up pipeline once
            render_pass.set_pipeline(&self.renderer.render_pipeline);
            render_pass.set_bind_group(0, &self.renderer.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.renderer.vertex_buffer.slice(..));
            
            drop(render_pass);
            self.render_pass_started = true;
        }
        
        // Update uniform buffer with cube position
        let center = position + Vec2::new(size / 2.0, size / 2.0);
        let uniform_data = Uniforms {
            cube_pos: [center.x, center.y],
            window_size: [self.renderer.window_width as f32, self.renderer.window_height as f32],
        };
        
        self.renderer.queue.write_buffer(&self.renderer.uniform_buffer, 0, bytemuck::cast_slice(&[uniform_data]));

        // Draw the cube
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Don't clear on subsequent draws
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.renderer.render_pipeline);
            render_pass.set_bind_group(0, &self.renderer.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.renderer.vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }
    }

    /// End the frame and present it to the screen. Consumes the Frame.
    pub fn end(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let encoder = self.encoder.take().expect("Encoder should exist");
        let output = self.output.take().expect("Output should exist");
        
        self.renderer.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

impl Renderer {
    fn create_cube_vertices(size: f32) -> [Vertex; 6] {
        [
            // Triangle 1
            Vertex {
                position: [-size, -size],
                color: [1.0, 0.0, 0.0], // Red
            },
            Vertex {
                position: [size, -size],
                color: [0.0, 1.0, 0.0], // Green
            },
            Vertex {
                position: [size, size],
                color: [0.0, 0.0, 1.0], // Blue
            },
            // Triangle 2
            Vertex {
                position: [-size, -size],
                color: [1.0, 0.0, 0.0], // Red
            },
            Vertex {
                position: [size, size],
                color: [0.0, 0.0, 1.0], // Blue
            },
            Vertex {
                position: [-size, size],
                color: [1.0, 1.0, 0.0], // Yellow
            },
        ]
    }
}
