use futures_lite::future::block_on;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub struct Window<'a> {
    pub renderer: Renderer,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: &'a winit::window::Window,
}

impl<'a> Window<'a> {
    pub fn new(window: &'a winit::window::Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        Self {
            renderer: Renderer::new(&instance, surface, size),
            size,
            window,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.renderer.surface_configuration.width = new_size.width;
            self.renderer.surface_configuration.height = new_size.height;
            self.renderer
                .surface
                .configure(&self.renderer.device, &self.renderer.surface_configuration);
        }
    }
}

pub struct Renderer {
    surface: wgpu::Surface,
    surface_configuration: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipelines: HashMap<TypeId, Arc<wgpu::RenderPipeline>>,
}

impl Renderer {
    pub fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface,
        size: winit::dpi::PhysicalSize<u32>,
    ) -> Self {
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: adapter.features(),
                limits: adapter.limits(),
                label: None,
            },
            None,
        ))
        .unwrap();

        let capabilities = surface.get_capabilities(&adapter);
        let surface_format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(capabilities.formats[0]);

        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_configuration);

        Self {
            surface,
            surface_configuration,
            pipelines: HashMap::new(),
            device,
            queue,
        }
    }

    pub fn get_pipeline<T: Pipeline>(&mut self) -> &Arc<wgpu::RenderPipeline> {
        let pipeline = self
            .pipelines
            .entry(Any::type_id(&PhantomData::<Self>))
            .or_insert_with(|| {
                Arc::new(T::create(&self.device, self.surface_configuration.format))
            });

        pipeline
    }

    pub fn create_vertex_buffer<T: Vertex>(&self, vertices: &[T]) -> VertexBuffer<T> {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        VertexBuffer {
            buffer,
            vertices: vertices.to_vec(),
        }
    }

    pub fn create_index_buffer(&self, indices: &[u16]) -> IndexBuffer {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });

        IndexBuffer {
            buffer,
            indices: indices.to_vec(),
        }
    }

    pub fn update_index_buffer<T: Pipeline, U>(
        &mut self,
        index_buffer: &mut IndexBuffer,
        indices: U,
    ) where
        Vec<u16>: From<U>,
    {
        index_buffer.indices = indices.into();
        self.queue.write_buffer(
            &index_buffer.buffer,
            0,
            bytemuck::cast_slice(&index_buffer.indices),
        );
    }

    pub fn update_vertex_buffer<T: Vertex, U>(
        &mut self,
        vertex_buffer: &mut VertexBuffer<T>,
        vertices: U,
    ) where
        Vec<T>: From<U>,
    {
        vertex_buffer.vertices = vertices.into();
        self.queue.write_buffer(
            &vertex_buffer.buffer,
            0,
            bytemuck::cast_slice(&vertex_buffer.vertices),
        );
    }

    pub fn encoder(&mut self) -> Result<Encoder, wgpu::SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        Ok(Encoder {
            encoder: Some(encoder),
            view: Arc::new(view),
            renderer: self,
            surface_texture: Some(surface_texture),
        })
    }

    pub fn with_encoder(&mut self, f: impl FnOnce(Encoder)) -> Result<(), wgpu::SurfaceError> {
        f(self.encoder()?);
        Ok(())
    }
}

pub trait Pipeline {
    fn create(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline;
}

pub trait Vertex: bytemuck::Pod {
    type Pipeline: Pipeline;

    fn buffer_layout() -> wgpu::VertexBufferLayout<'static>;
}

pub struct Encoder<'a> {
    encoder: Option<wgpu::CommandEncoder>,
    surface_texture: Option<wgpu::SurfaceTexture>,
    view: Arc<wgpu::TextureView>,
    renderer: &'a mut Renderer,
}

impl<'a> Drop for Encoder<'a> {
    fn drop(&mut self) {
        self.renderer
            .queue
            .submit([self.encoder.take().unwrap().finish()]);

        self.surface_texture.take().unwrap().present();
    }
}

impl<'a> Encoder<'a> {
    pub fn view(&self) -> Arc<wgpu::TextureView> {
        Arc::clone(&self.view)
    }

    pub fn render_pass<'b>(
        &'b mut self,
        desc: &wgpu::RenderPassDescriptor<'b, '_>,
    ) -> RenderPass<()> {
        let encoder = self.encoder.as_mut().unwrap();
        RenderPass {
            render_pass: encoder.begin_render_pass(desc),
            renderer: self.renderer,
            _phantom: PhantomData,
        }
    }
}

pub struct RenderPass<'a, T> {
    render_pass: wgpu::RenderPass<'a>,
    renderer: &'a mut Renderer,
    _phantom: PhantomData<T>,
}

impl<'a, T> RenderPass<'a, T> {
    pub fn step<P: Pipeline>(mut self) -> RenderPass<'a, P> {
        let pipeline = Arc::as_ptr(self.renderer.get_pipeline::<P>());
        self.render_pass.set_pipeline(unsafe { &*pipeline });

        RenderPass {
            render_pass: self.render_pass,
            renderer: self.renderer,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Pipeline> RenderPass<'a, T> {
    pub fn draw_buffers<V: Vertex<Pipeline = T>>(
        &mut self,
        vertex_buffer: &'a VertexBuffer<V>,
        index_buffer: &'a IndexBuffer,
    ) {
        self.render_pass
            .set_vertex_buffer(0, vertex_buffer.buffer.slice(..));
        self.render_pass
            .set_index_buffer(index_buffer.buffer.slice(..), wgpu::IndexFormat::Uint16);
        #[allow(clippy::cast_possible_truncation)]
        self.render_pass
            .draw_indexed(0..index_buffer.indices.len() as u32, 0, 0..1);
    }
}

pub struct VertexBuffer<T: Vertex> {
    buffer: wgpu::Buffer,
    vertices: Vec<T>,
}

impl<T: Vertex> VertexBuffer<T> {
    pub fn vertices(&self) -> &[T] {
        self.vertices.as_ref()
    }
}

pub struct IndexBuffer {
    buffer: wgpu::Buffer,
    indices: Vec<u16>,
}

impl IndexBuffer {
    pub fn indices(&self) -> &[u16] {
        self.indices.as_ref()
    }
}
