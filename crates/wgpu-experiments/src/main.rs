#![warn(clippy::pedantic)]

use window::{Pipeline, RenderPassExt, VertexBufferLayout, Window};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowBuilder;

mod window;

#[cfg(feature = "egl")]
#[link(name = "EGL")]
#[link(name = "GLESv2")]
extern "C" {}

struct MainPipeline;

impl Pipeline for MainPipeline {
    type Vertex = Vertex;

    fn create(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Self::Vertex::buffer_layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            depth_stencil: None,
        })
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];
}

impl VertexBufferLayout for Vertex {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("WGPU fun")
        .build(&event_loop)?;
    let window_id = window.id();
    let mut window = Window::new(&window);

    let mut vertex_buffer = window.create_vertex_buffer::<MainPipeline>(
        &[[0.1, 0.1], [-0.1, 0.1], [-0.1, -0.1], [0.1, -0.1]].map(|position| Vertex { position }),
    );
    let index_buffer = window.create_index_buffer::<MainPipeline>(&[0u16, 1, 2, 0, 2, 3]);

    event_loop.run(move |event, elwt| match event {
        winit::event::Event::WindowEvent {
            window_id: id,
            event,
        } if window_id == id => match event {
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    logical_key, state, ..
                },
                ..
            } => match (logical_key, state) {
                (Key::Named(NamedKey::Escape), ElementState::Pressed) => elwt.exit(),
                (Key::Named(NamedKey::Space), ElementState::Pressed) => {
                    let vertices = vertex_buffer
                        .vertices()
                        .iter()
                        .map(|Vertex { position }| Vertex {
                            position: position.map(|x| x + 0.01),
                        })
                        .collect::<Vec<_>>();

                    window.update_vertex_buffer(&mut vertex_buffer, vertices);
                    window.window.request_redraw();
                }
                _ => {}
            },
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(size) => window.resize(size),
            WindowEvent::RedrawRequested => {
                let pipeline = window.get_pipeline::<MainPipeline>();
                let render = window.render(|view, encoder| {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.5,
                                    b: 0.5,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                    render_pass.set_pipeline(&pipeline);
                    render_pass.draw_buffers_instanced(&vertex_buffer, &index_buffer, 0..10);
                });
                if let Err(err) = render {
                    match err {
                        wgpu::SurfaceError::Lost => window.resize(window.size),
                        wgpu::SurfaceError::OutOfMemory => elwt.exit(),
                        err => eprintln!("{err:?}"),
                    }
                }
            }
            _ => {}
        },
        _ => {}
    })?;

    Ok(())
}
