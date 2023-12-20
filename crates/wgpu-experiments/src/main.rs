#![warn(clippy::pedantic)]

use glam::{vec2, Vec2};
use image::RgbaImage;
use pipeline::Vertex;
use rand::prelude::*;
use renderer::texture::Texture;
use renderer::{bind_group, buffer, Encoder, Renderer, Window};
use std::sync::Arc;
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
struct ShaderGlobals {
    size: [f32; 2],
    window_size: [f32; 2],
}

#[derive(Clone)]
struct Step {
    position: Vec2,
    texture: Arc<Texture>,
}

impl buffer::ToVertex<Vertex> for Step {
    fn to_vertex<'a>(&'a self) -> impl AsRef<[Vertex]>
    where
        Vertex: 'a,
    {
        let texture: &RgbaImage = self.texture.as_ref().as_ref();
        let (width, height) = texture.dimensions();
        let half_width = Self::WIDTH / 2.;
        let half_height = half_width * height as f32 / width as f32;

        let Vec2 { x, y } = self.position;

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

impl Step {
    const WIDTH: f32 = 0.1;
}

struct Ball {
    position: Vec2,
    texture: Texture,
}

impl buffer::ToVertex<Vertex> for Ball {
    fn to_vertex<'a>(&'a self) -> impl AsRef<[Vertex]>
    where
        Vertex: 'a,
    {
        let texture: &RgbaImage = self.texture.as_ref();
        let (width, height) = texture.dimensions();
        let half_width = Self::WIDTH / 2.;
        let half_height = half_width * height as f32 / width as f32;

        let Vec2 { x, y } = self.position;

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

impl Ball {
    const WIDTH: f32 = 0.1;
}

struct Textures {
    step: Arc<Texture>,
    rectangle_index: buffer::Index,
}

#[derive(Debug, Clone, Copy)]
enum Event {
    Tick,
    Resize(winit::dpi::PhysicalSize<u32>),
    Left,
    Right,
}

struct State {
    textures: Textures,
    steps: Vec<buffer::Vertex<Vertex, Step>>,
    ball: buffer::Vertex<Vertex, Ball>,
    globals: bind_group::Single<ShaderGlobals>,
    ticks_since_step: f64,
}

impl State {
    const SIZE: [f32; 2] = [1., 1.];

    fn new(renderer: &mut Renderer) -> Self {
        const RECTANGLE_INDEX: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let rectangle_index = buffer::Index::create(&renderer.device, &RECTANGLE_INDEX);

        let globals = bind_group::Single::new(
            renderer,
            ShaderGlobals {
                size: Self::SIZE,
                window_size: [100., 100.],
            },
        );

        let step_texture = Arc::new(Texture::new(
            &image::load_from_memory(include_bytes!("../assets/step.png")).unwrap(),
            renderer,
        ));

        let ball_texture = Texture::new(
            &image::load_from_memory(include_bytes!("../assets/ball.png")).unwrap(),
            renderer,
        );

        Self {
            textures: Textures {
                rectangle_index,
                step: step_texture,
            },
            steps: Vec::new(),
            ball: buffer::Vertex::create(
                &renderer.device,
                Ball {
                    position: vec2(0.5, 0.5),
                    texture: ball_texture,
                },
            ),
            globals,
            ticks_since_step: 0.,
        }
    }

    fn handle_event(&mut self, event: Event, device: &wgpu::Device, queue: &wgpu::Queue) {
        match event {
            Event::Resize(size) => {
                self.globals.data_mut().window_size = [size.width as f32, size.height as f32];
                self.globals.update(queue);
            }
            Event::Tick => {
                if rand::distributions::Bernoulli::new((self.ticks_since_step - 3.).clamp(0., 1.))
                    .unwrap()
                    .sample(&mut thread_rng())
                {
                    self.ticks_since_step = 0.;

                    self.steps.push(buffer::Vertex::create(
                        device,
                        Step {
                            position: vec2(rand::random(), 0.),
                            texture: Arc::clone(&self.textures.step),
                        },
                    ));
                } else {
                    self.ticks_since_step += 1.;
                }

                self.steps.retain_mut(|step| {
                    let y = &mut step.data_mut().position.y;
                    *y += 0.01;
                    *y < 1.
                });
            }
            Event::Left => self.ball.data_mut().position.x -= 0.01,
            Event::Right => self.ball.data_mut().position.x += 0.01,
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
            .set_bind_group(1, self.textures.step.as_ref().as_ref(), &[]);

        for step in &self.steps {
            render_pass.draw_indexed(step, &self.textures.rectangle_index);
        }

        render_pass
            .render_pass
            .set_bind_group(1, self.ball.data().texture.as_ref(), &[]);

        render_pass.draw_indexed(&self.ball, &self.textures.rectangle_index);
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

    std::thread::spawn({
        let proxy = event_loop.create_proxy();
        move || loop {
            let _ = proxy.send_event(Event::Tick);
            std::thread::sleep(Duration::from_millis(50));
        }
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
                event:
                    KeyEvent {
                        logical_key,
                        state: key_state,
                        ..
                    },
                ..
            } => match (logical_key, key_state) {
                (Key::Named(NamedKey::Escape), ElementState::Pressed) => elwt.exit(),
                (Key::Named(NamedKey::ArrowLeft), ElementState::Pressed) => {
                    state.handle_event(Event::Left, &device, &queue);
                }
                (Key::Named(NamedKey::ArrowRight), ElementState::Pressed) => {
                    state.handle_event(Event::Right, &device, &queue);
                }
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
