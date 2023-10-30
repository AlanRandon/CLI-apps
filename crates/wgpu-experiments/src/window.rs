use futures_lite::future::block_on;
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Range;
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub struct Window<'a> {
    surface: wgpu::Surface,
    surface_configuration: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipelines: HashMap<TypeId, Arc<wgpu::RenderPipeline>>,
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
            size,
            device,
            queue,
            window,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_configuration.width = new_size.width;
            self.surface_configuration.height = new_size.height;
            self.surface
                .configure(&self.device, &self.surface_configuration);
        }
    }

    pub fn get_pipeline<T: Pipeline + 'static>(&mut self) -> Arc<wgpu::RenderPipeline> {
        let pipeline = self.pipelines.entry(TypeId::of::<T>()).or_insert_with(|| {
            Arc::new(T::create(&self.device, self.surface_configuration.format))
        });
        Arc::clone(pipeline)
    }

    pub fn create_vertex_buffer<T: Pipeline>(&self, vertices: &'a [T::Vertex]) -> VertexBuffer<T> {
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
            _phantom: PhantomData,
        }
    }

    pub fn create_index_buffer<T: Pipeline>(&self, indices: &'a [u16]) -> IndexBuffer<T> {
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
            _phantom: PhantomData,
        }
    }

    pub fn update_index_buffer<T: Pipeline, U>(
        &mut self,
        index_buffer: &mut IndexBuffer<T>,
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

    pub fn update_vertex_buffer<T: Pipeline, U>(
        &mut self,
        vertex_buffer: &mut VertexBuffer<T>,
        vertices: U,
    ) where
        Vec<T::Vertex>: From<U>,
    {
        vertex_buffer.vertices = vertices.into();
        self.queue.write_buffer(
            &vertex_buffer.buffer,
            0,
            bytemuck::cast_slice(&vertex_buffer.vertices),
        );
    }

    pub fn render(
        &self,
        render: impl FnOnce(&wgpu::TextureView, &mut wgpu::CommandEncoder),
    ) -> Result<(), wgpu::SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        render(&view, &mut encoder);

        self.queue.submit([encoder.finish()]);
        surface_texture.present();
        Ok(())
    }
}

pub struct VertexBuffer<T: Pipeline> {
    buffer: wgpu::Buffer,
    vertices: Vec<T::Vertex>,
    _phantom: PhantomData<T>,
}

impl<T: Pipeline> VertexBuffer<T> {
    pub fn vertices(&self) -> &[T::Vertex] {
        self.vertices.as_ref()
    }
}

pub struct IndexBuffer<T: Pipeline> {
    buffer: wgpu::Buffer,
    indices: Vec<u16>,
    _phantom: PhantomData<T>,
}

impl<T: Pipeline> IndexBuffer<T> {
    pub fn indices(&self) -> &[u16] {
        self.indices.as_ref()
    }
}

pub trait Pipeline {
    type Vertex: VertexBufferLayout + bytemuck::Pod;

    fn create(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline;
}

pub trait VertexBufferLayout {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static>;
}

pub trait RenderPassExt<'a> {
    fn draw_buffers_instanced<T: Pipeline>(
        &mut self,
        vertex_buffer: &'a VertexBuffer<T>,
        index_buffer: &'a IndexBuffer<T>,
        instances: Range<u32>,
    );

    fn draw_buffers<T: Pipeline>(
        &mut self,
        vertex_buffer: &'a VertexBuffer<T>,
        index_buffer: &'a IndexBuffer<T>,
    ) {
        self.draw_buffers_instanced(vertex_buffer, index_buffer, 0..1);
    }
}

impl<'a> RenderPassExt<'a> for wgpu::RenderPass<'a> {
    fn draw_buffers_instanced<T: Pipeline>(
        &mut self,
        vertex_buffer: &'a VertexBuffer<T>,
        index_buffer: &'a IndexBuffer<T>,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, vertex_buffer.buffer.slice(..));
        self.set_index_buffer(index_buffer.buffer.slice(..), wgpu::IndexFormat::Uint16);
        #[allow(clippy::cast_possible_truncation)]
        self.draw_indexed(0..index_buffer.indices.len() as u32, 0, instances);
    }
}
