#![warn(clippy::pedantic)]

use image::RgbaImage;
use pipeline::Vertex;
use renderer::texture::Texture;
use renderer::{bind_group, buffer, Renderer, Window};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowBuilder;

mod pipeline;
mod renderer;

#[cfg(feature = "egl")]
#[link(name = "EGL")]
#[link(name = "GLESv2")]
extern "C" {}

#[derive(Debug, bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C)]
struct Globals {
    size: [f32; 2],
    window_size: [f32; 2],
}

struct Step {
    x: f32,
    y: f32,
}

impl Step {
    const WIDTH: f32 = 0.1;

    fn get_buffers(&self, texture: &Texture, device: &wgpu::Device) -> ([Vertex; 4], [u16; 6]) {
        let texture: &RgbaImage = texture.as_ref();
        let (width, height) = texture.dimensions();
        let half_width = Self::WIDTH / 2.;
        let half_height = half_width * height as f32 / width as f32;

        (
            [
                Vertex {
                    position: [self.x - half_width, self.y - half_height],
                    tex_coords: [0., 1.],
                },
                Vertex {
                    position: [self.x + half_width, self.y - half_height],
                    tex_coords: [1., 1.],
                },
                Vertex {
                    position: [self.x + half_width, self.y + half_height],
                    tex_coords: [1., 0.],
                },
                Vertex {
                    position: [self.x - half_width, self.y + half_height],
                    tex_coords: [0., 0.],
                },
            ],
            [0, 1, 2, 0, 2, 3],
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("WGPU fun")
        .build(&event_loop)?;
    let window_id = window.id();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let (mut window, device, queue) = Window::new(&instance, &window);

    let mut globals = bind_group::Single::new(
        &mut window.renderer,
        Globals {
            window_size: [window.size.width as f32, window.size.height as f32],
            size: [1., 1.],
        },
    );

    let step_texture = Texture::new(
        &image::load_from_memory(include_bytes!("../assets/step.png")).unwrap(),
        &mut window.renderer,
    );
    let mut step = Step { x: 0.5, y: 0.5 };
    let (vertices, indices) = step.get_buffers(&step_texture, &device);
    let (mut vertex, mut index) = (
        buffer::Vertex::create(device.as_ref(), &vertices),
        buffer::Index::create(device.as_ref(), &indices),
    );

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
                (Key::Named(NamedKey::ArrowLeft), ElementState::Pressed) => step.x -= 0.01,
                (Key::Named(NamedKey::ArrowRight), ElementState::Pressed) => step.x += 0.01,
                _ => {}
            },
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(size) => {
                globals.update(
                    &queue,
                    Globals {
                        window_size: [size.width as f32, size.height as f32],
                        size: [1., 1.],
                    },
                );
                window.resize(size);
            }
            WindowEvent::RedrawRequested => {
                let (vertices, indices) = step.get_buffers(&step_texture, &device);
                vertex.update(queue.as_ref(), &vertices);
                index.update(queue.as_ref(), &indices);

                let result = window.renderer.with_encoder(|mut encoder| {
                    let view = encoder.view();
                    let mut render_pass = encoder
                        .render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
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
                            occlusion_query_set: None,
                            timestamp_writes: None,
                        })
                        .step::<pipeline::Pipeline>();

                    render_pass
                        .render_pass
                        .set_bind_group(0, globals.as_ref(), &[]);

                    render_pass
                        .render_pass
                        .set_bind_group(1, step_texture.as_ref(), &[]);

                    render_pass.draw_indexed(&vertex, &index);
                });

                if let Err(err) = result {
                    match err {
                        wgpu::SurfaceError::Lost => {
                            window.resize(window.size);
                        }
                        wgpu::SurfaceError::OutOfMemory => elwt.exit(),
                        _ => eprintln!("{err:?}"),
                    }
                }
            }
            _ => {}
        },
        _ => {}
    })?;

    Ok(())
}
