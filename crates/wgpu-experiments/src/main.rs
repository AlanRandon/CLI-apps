#![warn(clippy::pedantic)]

use image::RgbaImage;
use pipeline::Vertex;
use renderer::texture::Texture;
use renderer::{bind_group, buffer, Encoder, Renderer, Window};
use std::time::Duration;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::EventLoopBuilder;
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

#[derive(Debug, Clone, Copy)]
struct Step {
    x: f32,
    y: f32,
}

impl Step {
    const WIDTH: f32 = 0.1;
    const INDEX: [u16; 6] = [0, 1, 2, 0, 2, 3];

    fn vertex(&self, texture: &Texture) -> [Vertex; 4] {
        let texture: &RgbaImage = texture.as_ref();
        let (width, height) = texture.dimensions();
        let half_width = Self::WIDTH / 2.;
        let half_height = half_width * height as f32 / width as f32;

        let x = self.x;
        let y = self.y;

        [
            Vertex {
                position: [x - half_width, y - half_height],
                tex_coords: [0., 1.],
            },
            Vertex {
                position: [x + half_width, y - half_height],
                tex_coords: [1., 1.],
            },
            Vertex {
                position: [x + half_width, y + half_height],
                tex_coords: [1., 0.],
            },
            Vertex {
                position: [x - half_width, y + half_height],
                tex_coords: [0., 0.],
            },
        ]
    }
}

#[derive(Debug, Clone, Copy)]
enum Event {
    Tick,
    Resize(winit::dpi::PhysicalSize<u32>),
}

struct State {
    step_texture: Texture,
    step_index: buffer::Index,
    steps: Vec<(Step, buffer::Vertex<Vertex>)>,
    globals: bind_group::Single<Globals>,
}

impl State {
    const SIZE: [f32; 2] = [1., 1.];

    fn new(renderer: &mut Renderer) -> Self {
        let step_texture = Texture::new(
            &image::load_from_memory(include_bytes!("../assets/step.png")).unwrap(),
            renderer,
        );

        let step_index = buffer::Index::create(&renderer.device, &Step::INDEX);

        let globals = bind_group::Single::new(
            renderer,
            Globals {
                size: Self::SIZE,
                window_size: [100., 100.],
            },
        );

        Self {
            step_index,
            steps: (0..10)
                .map(|_| {
                    let step = Step {
                        x: rand::random(),
                        y: 0.,
                    };

                    (
                        step,
                        buffer::Vertex::create(&renderer.device, &step.vertex(&step_texture)),
                    )
                })
                .collect(),
            globals,
            step_texture,
        }
    }

    fn handle_event(&mut self, event: Event, device: &wgpu::Device, queue: &wgpu::Queue) {
        match event {
            Event::Resize(size) => self.globals.update(
                queue,
                Globals {
                    size: Self::SIZE,
                    window_size: [size.width as f32, size.height as f32],
                },
            ),
            Event::Tick => {
                for step in &mut self.steps {
                    step.0.y += 0.01;
                    step.1.update(queue, step.0.vertex(&self.step_texture));
                }
            }
        }
    }

    fn render(&self, encoder: &mut Encoder) {
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
            .set_bind_group(0, self.globals.as_ref(), &[]);

        render_pass
            .render_pass
            .set_bind_group(1, self.step_texture.as_ref(), &[]);

        for step in &self.steps {
            render_pass.draw_indexed(&step.1, &self.step_index);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let event_loop = EventLoopBuilder::with_user_event().build().unwrap();
    let window = WindowBuilder::new()
        .with_title("WGPU fun")
        .build(&event_loop)?;
    let window_id = window.id();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let (mut window, device, queue) = Window::new(&instance, &window);

    let mut state = State::new(&mut window.renderer);
    state.handle_event(Event::Resize(window.size), &device, &queue);

    let proxy = event_loop.create_proxy();

    std::thread::spawn(move || loop {
        let _ = proxy.send_event(Event::Tick);
        std::thread::sleep(Duration::from_millis(50));
    });

    event_loop.run(move |event, elwt| match event {
        winit::event::Event::UserEvent(event) => {
            state.handle_event(event, &device, &queue);
            window.window.request_redraw();
        }
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
                _ => {}
            },
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(size) => {
                state.handle_event(Event::Resize(size), &device, &queue);
                window.resize(size);
            }
            WindowEvent::RedrawRequested => {
                let result = window
                    .renderer
                    .with_encoder(|mut encoder| state.render(&mut encoder));

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
