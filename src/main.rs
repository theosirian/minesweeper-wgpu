use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

enum Pipeline {
    Simple,
    Color,
}

struct State {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    hue: f64,
    sat: f64,
    val: f64,
    clear_color: wgpu::Color,

    which_pipeline: self::Pipeline,
    simple_pipeline: wgpu::RenderPipeline,
    color_pipeline: wgpu::RenderPipeline,

    size: winit::dpi::PhysicalSize<u32>,
}

impl State {
    fn create_pipeline(
        vs_src: &str,
        fs_src: &str,
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> wgpu::RenderPipeline {
        let vs_spirv = glsl_to_spirv::compile(vs_src, glsl_to_spirv::ShaderType::Vertex).unwrap();
        let fs_spirv = glsl_to_spirv::compile(fs_src, glsl_to_spirv::ShaderType::Fragment).unwrap();

        let vs_data = wgpu::read_spirv(vs_spirv).unwrap();
        let fs_data = wgpu::read_spirv(fs_spirv).unwrap();

        let vs_module = device.create_shader_module(&vs_data);
        let fs_module = device.create_shader_module(&fs_data);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        render_pipeline
    }

    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let surface = wgpu::Surface::create(window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: Default::default(),
            })
            .await;

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let simple_pipeline = Self::create_pipeline(
            include_str!("simple.vert"),
            include_str!("simple.frag"),
            &device,
            &sc_desc,
        );

        let color_pipeline = Self::create_pipeline(
            include_str!("color.vert"),
            include_str!("color.frag"),
            &device,
            &sc_desc,
        );

        Self {
            surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,

            which_pipeline: self::Pipeline::Simple,
            simple_pipeline,
            color_pipeline,

            hue: 0.0,
            sat: 0.5,
            val: 0.5,

            clear_color: wgpu::Color {
                r: 0.3,
                g: 0.2,
                b: 0.1,
                a: 1.0,
            },
        }
    }

    async fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        ..
                    },
                ..
            } => {
                self.which_pipeline = match self.which_pipeline {
                    self::Pipeline::Simple => self::Pipeline::Color,
                    self::Pipeline::Color => self::Pipeline::Simple,
                };
                false
            }

            WindowEvent::CursorEntered { .. } => {
                self.clear_color = wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                };

                false
            }

            WindowEvent::CursorLeft { .. } => {
                self.clear_color = wgpu::Color {
                    r: 0.3,
                    g: 0.2,
                    b: 0.1,
                    a: 1.0,
                };

                false
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.sat = position.x / self.size.width as f64;
                self.val = position.y / self.size.height as f64;

                true
            }

            _ => true,
        }
    }

    async fn update(&mut self) {
        self.hue = self.hue + 1.0 % 360.0;
    }

    async fn render(&mut self) {
        let frame = self
            .swap_chain
            .get_next_texture()
            .expect("Timeout getting texture");

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let color: palette::Srgb<f64> = palette::Hsv::new(self.hue, self.sat, self.val).into();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color {
                    r: color.red,
                    g: color.green,
                    b: color.blue,
                    a: 1.0,
                },
            }],
            depth_stencil_attachment: None,
        });

        match self.which_pipeline {
            self::Pipeline::Simple => render_pass.set_pipeline(&self.simple_pipeline),
            self::Pipeline::Color => render_pass.set_pipeline(&self.color_pipeline),
        };

        render_pass.draw(0..3, 0..1);

        drop(render_pass);

        self.queue.submit(&[encoder.finish()]);
    }
}

fn main() {
    use futures::executor::block_on;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Q),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        },

                        WindowEvent::Resized(physical_size) => {
                            block_on(state.resize(*physical_size));
                        }

                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            block_on(state.resize(**new_inner_size));
                        }

                        _ => {}
                    }
                }
            }

            Event::RedrawRequested(_) => {
                block_on(state.update());
                block_on(state.render());
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
