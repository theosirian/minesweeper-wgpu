use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

enum Toggle {
    A,
    B,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[wgpu::VertexAttributeDescriptor {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float3,
            }],
        }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

// #[repr(C)]
// #[derive(Copy, Clone, Debug)]
// struct Color {
//     color: [f32; 3],
// }

// impl Color {
//     fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
//         use std::mem;
//         wgpu::VertexBufferDescriptor {
//             stride: mem::size_of::<Color>() as wgpu::BufferAddress,
//             step_mode: wgpu::InputStepMode::Vertex,
//             attributes: &[wgpu::VertexAttributeDescriptor {
//                 offset: 0,
//                 shader_location: 0,
//                 format: wgpu::VertexFormat::Float3,
//             }],
//         }
//     }
// }

// unsafe impl bytemuck::Pod for Color {}
// unsafe impl bytemuck::Zeroable for Color {}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
    },
    Vertex {
        position: [-0.76935, 1.059, 0.0],
    },
    Vertex {
        position: [-0.4755, 0.1545, 0.0],
    },
    Vertex {
        position: [-1.24485, -0.4045, 0.0],
    },
    Vertex {
        position: [-0.29385, -0.4045, 0.0],
    },
    Vertex {
        position: [0.0, -1.309, 0.0],
    },
    Vertex {
        position: [0.29385, -0.4045, 0.0],
    },
    Vertex {
        position: [1.24485, -0.4045, 0.0],
    },
    Vertex {
        position: [0.4755, 0.1545, 0.0],
    },
    Vertex {
        position: [0.76935, 1.059, 0.0],
    },
];

const PENTAGON_INDICES: &[u16] = &[
    0, 2, 8, //
    2, 4, 8, //
    4, 6, 8, //
];

const STAR_INDICES: &[u16] = &[
    0, 2, 8, //
    2, 4, 8, //
    4, 6, 8, //
    0, 1, 2, //
    2, 3, 4, //
    4, 5, 6, //
    6, 7, 8, //
    0, 8, 9, //
];

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

    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,

    pentagon_indices: wgpu::Buffer,
    pentagon_len: u32,

    star_indices: wgpu::Buffer,
    star_len: u32,

    // color_buffer: wgpu::Buffer,
    toggle: self::Toggle,

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
                vertex_buffers: &[self::Vertex::desc()],
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

        let render_pipeline = Self::create_pipeline(
            include_str!("simple.vert"),
            include_str!("simple.frag"),
            &device,
            &sc_desc,
        );

        let vertex_buffer = device
            .create_buffer_with_data(bytemuck::cast_slice(VERTICES), wgpu::BufferUsage::VERTEX);

        let pentagon_indices = device.create_buffer_with_data(
            bytemuck::cast_slice(PENTAGON_INDICES),
            wgpu::BufferUsage::INDEX,
        );
        let pentagon_len = PENTAGON_INDICES.len() as u32;

        let star_indices = device
            .create_buffer_with_data(bytemuck::cast_slice(STAR_INDICES), wgpu::BufferUsage::INDEX);
        let star_len = STAR_INDICES.len() as u32;

        // let color_buffer = device.create_buffer_with_data(
        //     bytemuck::cast_slice(&[Color {
        //         color: [1.0, 1.0, 1.0],
        //     }]),
        //     wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::WRITE_ALL,
        // );

        Self {
            surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,

            render_pipeline,

            hue: 0.0,
            sat: 0.5,
            val: 0.5,

            vertex_buffer,

            pentagon_indices,
            pentagon_len,

            star_indices,
            star_len,

            // color_buffer,
            toggle: self::Toggle::A,

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
                self.toggle = match self.toggle {
                    self::Toggle::A => self::Toggle::B,
                    self::Toggle::B => self::Toggle::A,
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

        // let data = self
        //     .color_buffer
        //     .map_write(0, std::mem::size_of::<Color>() as wgpu::BufferAddress);

        // self.device.poll(wgpu::Maintain::Wait);

        // if let Ok(mut data) = data.await {
        //     let color: palette::Srgb<f32> = palette::Hsv::new(
        //         ((self.hue + 180.0) % 360.0) as f32,
        //         self.sat as f32,
        //         self.val as f32,
        //     )
        //     .into();

        //     let new_color = Color {
        //         // color: [color.red, color.green, color.blue],
        //         color: [0.0, 0.0, 0.0],
        //     };

        //     data.as_slice()
        //         .copy_from_slice(bytemuck::cast_slice(&[new_color]));
        // } else {
        //     println!("Something went wrong");
        // }

        // self.color_buffer.unmap();

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

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        // render_pass.set_vertex_buffer(1, &self.color_buffer, 0, 0);

        match self.toggle {
            self::Toggle::A => {
                render_pass.set_index_buffer(&self.pentagon_indices, 0, 0);
                render_pass.draw_indexed(0..self.pentagon_len, 0, 0..1);
            }
            self::Toggle::B => {
                render_pass.set_index_buffer(&self.star_indices, 0, 0);
                render_pass.draw_indexed(0..self.star_len, 0, 0..1);
            }
        };

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
