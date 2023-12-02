use super::VertexLayout;
use wgpu::util::DeviceExt;
use wgpu::BufferUsages;

pub struct Vertex<T: VertexLayout> {
    many: Many<T>,
}

impl<T: VertexLayout> Vertex<T> {
    pub fn vertices(&self) -> &[T] {
        self.many.data.as_ref()
    }

    pub fn create(device: &wgpu::Device, indices: &[T]) -> Self {
        Self {
            many: Many::create(
                device,
                indices,
                BufferUsages::VERTEX | BufferUsages::COPY_DST,
            ),
        }
    }

    pub fn update<U>(&mut self, queue: &wgpu::Queue, indices: U)
    where
        Vec<T>: From<U>,
    {
        self.many.update(queue, indices)
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        self.many.buffer()
    }
}

pub struct Index {
    many: Many<u16>,
}

impl Index {
    pub fn indices(&self) -> &[u16] {
        self.many.data.as_ref()
    }

    pub fn create(device: &wgpu::Device, indices: &[u16]) -> Self {
        Self {
            many: Many::create(
                device,
                indices,
                BufferUsages::INDEX | BufferUsages::COPY_DST,
            ),
        }
    }

    pub fn update<U>(&mut self, queue: &wgpu::Queue, indices: U)
    where
        Vec<u16>: From<U>,
    {
        self.many.update(queue, indices)
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        self.many.buffer()
    }
}

pub struct Single<T: bytemuck::Pod> {
    buffer: wgpu::Buffer,
    data: T,
}

impl<T: bytemuck::Pod> Single<T> {
    pub fn create(device: &wgpu::Device, data: T, usage: wgpu::BufferUsages) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[data]),
            usage,
        });

        Self { buffer, data }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, data: T) {
        self.data = data;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

pub struct Many<T: bytemuck::Pod> {
    buffer: wgpu::Buffer,
    data: Vec<T>,
}

impl<T: bytemuck::Pod> Many<T> {
    pub fn create<U>(device: &wgpu::Device, data: U, usage: wgpu::BufferUsages) -> Self
    where
        Vec<T>: From<U>,
    {
        let data = Vec::from(data);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&data),
            usage,
        });

        Self { buffer, data }
    }

    pub fn update<U>(&mut self, queue: &wgpu::Queue, data: U)
    where
        Vec<T>: From<U>,
    {
        self.data = data.into();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data));
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
