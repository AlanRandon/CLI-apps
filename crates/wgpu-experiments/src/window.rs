use futures_lite::future::block_on;
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Range;
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

pub struct PipelineData {
    pipeline: wgpu::RenderPipeline,
}

impl PipelineData {
    pub fn new(device: &wgpu::Device, desc: &wgpu::RenderPipelineDescriptor) -> Self {
        Self {
            pipeline: device.create_render_pipeline(desc),
        }
    }
}

pub struct Renderer {
    surface: wgpu::Surface,
    surface_configuration: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    pipelines: HashMap<TypeId, Arc<PipelineData>>,
    bind_group_layouts: HashMap<TypeId, Arc<wgpu::BindGroupLayout>>,
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
            device,
            queue,
            pipelines: HashMap::new(),
            bind_group_layouts: HashMap::new(),
        }
    }

    pub fn get_pipeline<T: Pipeline + 'static>(&mut self) -> &Arc<PipelineData> {
        self.pipelines.entry(TypeId::of::<T>()).or_insert_with(|| {
            let pipeline = T::create(&self.device, self.surface_configuration.format);
            Arc::new(pipeline)
        })
    }

    pub fn get_bind_group_layout<B: BindGroupLayout + 'static>(
        &mut self,
    ) -> &Arc<wgpu::BindGroupLayout>
    where
        [(); B::ENTRIES]: Sized,
    {
        self.bind_group_layouts
            .entry(TypeId::of::<B>())
            .or_insert_with(|| {
                let layout =
                    self.device
                        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: None,
                            entries: &B::layout_entries(),
                        });
                Arc::new(layout)
            })
    }

    pub fn create_bind_group<T: BindGroupLayout + 'static>(
        &mut self,
        entries: [wgpu::BindGroupEntry; T::ENTRIES],
    ) -> BindGroup<T>
    where
        T::Pipeline: 'static,
    {
        let layout = Arc::clone(self.get_bind_group_layout::<T>());
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &entries,
        });

        BindGroup {
            bind_group,
            _phantom: PhantomData,
        }
    }

    pub fn create_vertex_buffer<T: VertexLayout>(&self, vertices: &[T]) -> VertexBuffer<T> {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vertices),
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
                contents: bytemuck::cast_slice(indices),
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

    pub fn update_vertex_buffer<T: VertexLayout, U>(
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
    fn create(device: &wgpu::Device, format: wgpu::TextureFormat) -> PipelineData;
}

pub trait BindGroupLayout {
    type Pipeline: Pipeline;
    const ENTRIES: usize;

    fn layout_entries() -> [wgpu::BindGroupLayoutEntry; Self::ENTRIES];

    fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
    where
        [(); Self::ENTRIES]: Sized,
    {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &Self::layout_entries(),
        })
    }
}

pub struct BindGroup<T: BindGroupLayout + ?Sized> {
    bind_group: wgpu::BindGroup,
    _phantom: PhantomData<T>,
}

impl<T: BindGroupLayout + ?Sized> AsRef<wgpu::BindGroup> for BindGroup<T> {
    fn as_ref(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

pub trait VertexLayout: bytemuck::Pod {
    type Pipeline: Pipeline;

    fn layout() -> wgpu::VertexBufferLayout<'static>;
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
    pub render_pass: wgpu::RenderPass<'a>,
    renderer: &'a mut Renderer,
    _phantom: PhantomData<T>,
}

impl<'a, T> RenderPass<'a, T> {
    pub fn step<P: Pipeline + 'static>(mut self) -> RenderPass<'a, P> {
        let pipeline = self.renderer.get_pipeline::<P>();
        {
            let pipeline = Arc::as_ptr(pipeline);
            self.render_pass
                .set_pipeline(&unsafe { &*pipeline }.pipeline);
        }

        RenderPass {
            render_pass: self.render_pass,
            renderer: self.renderer,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Pipeline> RenderPass<'a, T> {
    pub fn draw<V: VertexLayout<Pipeline = T>>(&mut self, vertex_buffer: &'a VertexBuffer<V>) {
        self.render_pass
            .set_vertex_buffer(0, vertex_buffer.buffer.slice(..));
        #[allow(clippy::cast_possible_truncation)]
        self.render_pass
            .draw(0..vertex_buffer.vertices.len() as u32, 0..1);
    }

    pub fn draw_indexed<V: VertexLayout<Pipeline = T>>(
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

    pub fn draw_instanced<V: VertexLayout<Pipeline = T>>(
        &mut self,
        vertex_buffer: &'a VertexBuffer<V>,
        index_buffer: &'a IndexBuffer,
        instances: Range<u32>,
    ) {
        self.render_pass
            .set_vertex_buffer(0, vertex_buffer.buffer.slice(..));
        self.render_pass
            .set_index_buffer(index_buffer.buffer.slice(..), wgpu::IndexFormat::Uint16);
        #[allow(clippy::cast_possible_truncation)]
        self.render_pass
            .draw_indexed(0..index_buffer.indices.len() as u32, 0, instances);
    }
}

pub struct VertexBuffer<T: VertexLayout> {
    buffer: wgpu::Buffer,
    vertices: Vec<T>,
}

impl<T: VertexLayout> VertexBuffer<T> {
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
