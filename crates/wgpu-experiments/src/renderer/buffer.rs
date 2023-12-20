use std::marker::PhantomData;
use wgpu::util::DeviceExt;
use wgpu::BufferUsages;

pub struct Vertex<V, T> {
    many: Many<V, T>,
}

impl<V, T> Vertex<V, T> {
    pub fn create(device: &wgpu::Device, data: T) -> Self
    where
        V: bytemuck::Pod,
        T: ToVertex<V>,
    {
        Self {
            many: Many::create(device, data, BufferUsages::VERTEX | BufferUsages::COPY_DST),
        }
    }

    pub fn update(&self, queue: &wgpu::Queue) -> impl AsRef<[V]> + '_
    where
        V: bytemuck::Pod,
        T: ToVertex<V>,
    {
        self.many.update(queue)
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        self.many.buffer()
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.many.data
    }

    pub fn data(&self) -> &T {
        &self.many.data
    }
}

pub struct Index {
    many: Many<u16, Vec<u16>>,
}

impl Index {
    pub fn indices(&self) -> &[u16] {
        self.many.data.as_ref()
    }

    pub fn create(device: &wgpu::Device, indices: &[u16]) -> Self {
        Self {
            many: Many::create(device, indices.to_vec(), BufferUsages::INDEX),
        }
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

    pub fn update(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.data]));
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

pub struct Many<V, T> {
    buffer: wgpu::Buffer,
    data: T,
    _phantom: PhantomData<V>,
}

impl<V, T> Many<V, T> {
    pub fn create(device: &wgpu::Device, data: T, usage: wgpu::BufferUsages) -> Self
    where
        V: bytemuck::Pod,
        T: ToVertex<V>,
    {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(data.to_vertex().as_ref()),
            usage,
        });

        Self {
            buffer,
            data,
            _phantom: PhantomData,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue) -> impl AsRef<[V]> + '_
    where
        V: bytemuck::Pod,
        T: ToVertex<V>,
    {
        let vertex = self.data.to_vertex();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(vertex.as_ref()));
        vertex
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

pub trait ToVertex<V> {
    fn to_vertex<'a>(&'a self) -> impl AsRef<[V]>
    where
        V: 'a;
}

impl<V, T: AsRef<[V]>> ToVertex<V> for T {
    fn to_vertex<'a>(&'a self) -> impl AsRef<[V]>
    where
        V: 'a,
    {
        self.as_ref()
    }
}
