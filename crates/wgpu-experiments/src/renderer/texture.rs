use super::bind_group::{self, BindGroupLayout};
use super::Renderer;
use image::RgbaImage;

pub struct Texture {
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group: bind_group::BindGroup<Self>,
    image: RgbaImage,
}

impl Texture {
    pub fn new(image: &image::DynamicImage, renderer: &mut Renderer) -> Self {
        let image = image.to_rgba8();
        let (width, height) = image.dimensions();

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let diffuse_texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: None,
            view_formats: &[],
        });

        renderer.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let diffuse_sampler = renderer.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = renderer.create_bind_group::<Self>([
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
            },
        ]);

        Self {
            view: diffuse_texture_view,
            sampler: diffuse_sampler,
            bind_group,
            image,
        }
    }
}

impl BindGroupLayout for Texture {
    fn layout_entries() -> impl AsRef<[wgpu::BindGroupLayoutEntry]> {
        [
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ]
    }
}

impl AsRef<wgpu::BindGroup> for Texture {
    fn as_ref(&self) -> &wgpu::BindGroup {
        self.bind_group.as_ref()
    }
}

impl AsRef<RgbaImage> for Texture {
    fn as_ref(&self) -> &RgbaImage {
        &self.image
    }
}
