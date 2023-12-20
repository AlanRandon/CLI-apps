use self::buffer::{Index, ToVertex, Vertex};
use futures_lite::future::block_on;
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Range;
use std::sync::Arc;

pub mod bind_group;
pub mod buffer;
pub mod texture;

pub struct Window<'a> {
    pub renderer: Renderer,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: &'a winit::window::Window,
}

impl<'a> Window<'a> {
    pub fn new(
        instance: &'a wgpu::Instance,
        window: &'a winit::window::Window,
    ) -> (Self, Arc<wgpu::Device>, Arc<wgpu::Queue>) {
        let size = window.inner_size();

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let (renderer, device, queue) = Renderer::new(surface, size, instance);

        let window = Self {
            renderer,
            size,
            window,
        };

        (window, device, queue)
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
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pipelines: HashMap<TypeId, Arc<PipelineData>>,
    bind_group_layouts: HashMap<TypeId, Arc<wgpu::BindGroupLayout>>,
}

impl Renderer {
    pub fn new(
        surface: wgpu::Surface,
        size: winit::dpi::PhysicalSize<u32>,
        instance: &wgpu::Instance,
    ) -> (Self, Arc<wgpu::Device>, Arc<wgpu::Queue>) {
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
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

        let renderer = Self {
            surface,
            surface_configuration,
            device: Arc::new(device),
            queue: Arc::new(queue),
            pipelines: HashMap::new(),
            bind_group_layouts: HashMap::new(),
        };

        let device = Arc::clone(&renderer.device);
        let queue = Arc::clone(&renderer.queue);

        (renderer, device, queue)
    }

    pub fn get_pipeline<T: Pipeline + 'static>(&mut self) -> &Arc<PipelineData> {
        self.pipelines.entry(TypeId::of::<T>()).or_insert_with(|| {
            let pipeline = T::create(&self.device, self.surface_configuration.format);
            Arc::new(pipeline)
        })
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
    pub fn draw<V: VertexLayout<Pipeline = T>, U>(&mut self, vertex_buffer: &'a Vertex<V, U>)
    where
        V: bytemuck::Pod,
        U: ToVertex<V>,
    {
        let len = vertex_buffer
            .update(self.renderer.queue.as_ref())
            .as_ref()
            .len();

        self.render_pass
            .set_vertex_buffer(0, vertex_buffer.buffer().slice(..));
        #[allow(clippy::cast_possible_truncation)]
        self.render_pass.draw(0..len as u32, 0..1);
    }

    pub fn draw_indexed<V: VertexLayout<Pipeline = T>, U>(
        &mut self,
        vertex_buffer: &'a Vertex<V, U>,
        index_buffer: &'a Index,
    ) where
        V: bytemuck::Pod,
        U: ToVertex<V>,
    {
        vertex_buffer.update(self.renderer.queue.as_ref());

        self.render_pass
            .set_vertex_buffer(0, vertex_buffer.buffer().slice(..));
        self.render_pass
            .set_index_buffer(index_buffer.buffer().slice(..), wgpu::IndexFormat::Uint16);
        #[allow(clippy::cast_possible_truncation)]
        self.render_pass
            .draw_indexed(0..index_buffer.indices().len() as u32, 0, 0..1);
    }

    pub fn draw_instanced<V: VertexLayout<Pipeline = T>, U>(
        &mut self,
        vertex_buffer: &'a Vertex<V, U>,
        index_buffer: &'a Index,
        instances: Range<u32>,
    ) where
        V: bytemuck::Pod,
        U: ToVertex<V>,
    {
        vertex_buffer.update(self.renderer.queue.as_ref());

        self.render_pass
            .set_vertex_buffer(0, vertex_buffer.buffer().slice(..));
        self.render_pass
            .set_index_buffer(index_buffer.buffer().slice(..), wgpu::IndexFormat::Uint16);
        #[allow(clippy::cast_possible_truncation)]
        self.render_pass
            .draw_indexed(0..index_buffer.indices().len() as u32, 0, instances);
    }
}
