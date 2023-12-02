use super::{buffer, Renderer};
use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::Arc;
use wgpu::BufferUsages;

impl Renderer {
    pub fn get_bind_group_layout<B: BindGroupLayout + 'static>(
        &mut self,
    ) -> &Arc<wgpu::BindGroupLayout> {
        self.bind_group_layouts
            .entry(TypeId::of::<B>())
            .or_insert_with(|| {
                let layout =
                    self.device
                        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: None,
                            entries: B::layout_entries().as_ref(),
                        });
                Arc::new(layout)
            })
    }

    pub fn create_bind_group<'a, T: BindGroupLayout + 'static>(
        &mut self,
        entries: impl AsRef<[wgpu::BindGroupEntry<'a>]>,
    ) -> BindGroup<T> {
        let layout = Arc::clone(self.get_bind_group_layout::<T>());
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: entries.as_ref(),
        });

        BindGroup {
            bind_group,
            _phantom: PhantomData,
        }
    }
}

pub trait BindGroupLayout {
    fn layout_entries() -> impl AsRef<[wgpu::BindGroupLayoutEntry]>;

    fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: Self::layout_entries().as_ref(),
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

pub struct Single<T: bytemuck::Pod> {
    buffer: buffer::Single<T>,
    bind_group: BindGroup<Self>,
}

impl<T: bytemuck::Pod> AsRef<wgpu::BindGroup> for Single<T> {
    fn as_ref(&self) -> &wgpu::BindGroup {
        self.bind_group.as_ref()
    }
}

impl<T: bytemuck::Pod> BindGroupLayout for Single<T> {
    fn layout_entries() -> impl AsRef<[wgpu::BindGroupLayoutEntry]> {
        [wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }]
    }
}

impl<T: bytemuck::Pod> Single<T> {
    pub fn new(renderer: &mut Renderer, item: T) -> Self {
        let buffer = buffer::Single::create(
            renderer.device.as_ref(),
            item,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let bind_group = renderer.create_bind_group::<Self>([wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.buffer().as_entire_binding(),
        }]);

        Self { buffer, bind_group }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, item: T) {
        self.buffer.update(queue, item);
    }
}
