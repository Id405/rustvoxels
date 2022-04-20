use std::collections::HashMap;

use wgpu::TextureViewDescriptor;
use winit::dpi::PhysicalSize;

use super::RenderContext;

#[derive(Clone, Copy)]
pub struct TextureInfo {
    pub size: (u32, u32, u32),
    pub mip_levels: u32,
}

pub enum TextureType {
    TextureSwapChain(wgpu::Texture, wgpu::Texture),
    SingleTexture(wgpu::Texture),
    DescriptorTexture(wgpu::Texture, TextureInfo), // Own only, don't mess with it at all
    Buffer(wgpu::Buffer),
}

pub struct TextureAtlas {
    textures: HashMap<String, TextureType>,
    sampler: wgpu::Sampler,
}

impl TextureAtlas {
    pub fn new(context: &RenderContext) -> Self {
        Self {
            textures: HashMap::new(),
            sampler: context
                .device
                .create_sampler(&wgpu::SamplerDescriptor::default()),
        }
    }

    pub fn register<S>(&mut self, name: S, context: &RenderContext)
    where
        S: Into<String>,
    {
        let name: String = name.into();

        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            size: Self::texture_size(context),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some(name.as_str()),
        });

        self.textures
            .insert(name, TextureType::SingleTexture(texture));
    }

    pub fn register_from_image<S>(&mut self, name: S, bytes: &[u8], context: &RenderContext)
    where
        S: Into<String>,
    {
        let name: String = name.into();

        let image = image::load_from_memory(bytes).unwrap();
        let rgba = image.as_rgba8().unwrap();

        use image::GenericImageView;
        let dimensions = image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(name.as_str()),
        });

        context.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            rgba,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            texture_size,
        );

        self.textures.insert(
            name,
            TextureType::DescriptorTexture(
                texture,
                TextureInfo {
                    size: (dimensions.0, dimensions.1, 1),
                    mip_levels: 1,
                },
            ),
        );
    }

    pub fn register_swapchain<S>(&mut self, name: S, context: &RenderContext)
    where
        S: Into<String>,
    {
        let name: String = name.into();

        let textures = (
            context.device.create_texture(&wgpu::TextureDescriptor {
                size: Self::texture_size(context),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some(format!("{} swap even", name.as_str()).as_str()),
            }),
            context.device.create_texture(&wgpu::TextureDescriptor {
                size: Self::texture_size(context),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some(format!("{} swap even", name.as_str()).as_str()),
            }),
        );

        self.textures.insert(
            name.into(),
            TextureType::TextureSwapChain(textures.0, textures.1),
        );
    }

    pub fn register_from_descriptor<S>(
        &mut self,
        name: S,
        descriptor: wgpu::TextureDescriptor,
        context: &RenderContext,
    ) where
        S: Into<String>,
    {
        let name: String = name.into();

        let texture = context.device.create_texture(&descriptor);

        self.textures.insert(
            name,
            TextureType::DescriptorTexture(texture, TextureInfo {
                size: (descriptor.size.width, descriptor.size.height, descriptor.size.depth_or_array_layers),
                mip_levels: descriptor.mip_level_count,
            }),
        );
    }

    pub fn register_buffer<S>(
        &mut self,
        name: S,
        descriptor: wgpu::BufferDescriptor,
        context: &RenderContext,
    ) where
        S: Into<String>,
    {
        let name: String = name.into();

        let buffer = context.device.create_buffer(&descriptor);

        self.textures.insert(name, TextureType::Buffer(buffer));
    }

    pub fn resize(&mut self, context: &RenderContext) {
        let keys = self.textures.keys().map(|x| x.clone()).collect::<Vec<_>>();
        for name in keys {
            match self.textures.get(&name).unwrap() {
                TextureType::TextureSwapChain(_, _) => self.register_swapchain(name, context),
                TextureType::SingleTexture(_) => self.register(name, context),
                TextureType::DescriptorTexture(_, _) => (),
                TextureType::Buffer(_) => (),
            }
        }
    }

    pub fn get<S>(&self, name: S, context: &RenderContext) -> Option<&wgpu::Texture>
    where
        S: Into<String>,
    {
        match self.textures.get(&name.into()) {
            Some(texture_type) => {
                match texture_type {
                    TextureType::TextureSwapChain(texture1, texture2) => {
                        match context.frame_count % 2 == 0 {
                            // TODO ensure most recent texture is returned
                            true => Some(&texture1),
                            false => Some(&texture2),
                        }
                    }
                    TextureType::SingleTexture(texture) => Some(&texture),
                    TextureType::DescriptorTexture(texture, _) => Some(&texture),
                    TextureType::Buffer(_) => None,
                }
            }
            None => None,
        }
    }

    pub fn get_view<S>(&self, name: S, context: &RenderContext) -> Option<wgpu::TextureView>
    where
        S: Into<String>,
    {
        match self.textures.get(&name.into()) {
            Some(texture_type) => {
                match texture_type {
                    TextureType::TextureSwapChain(texture1, texture2) => {
                        match context.frame_count % 2 == 0 {
                            // TODO ensure most recent texture is returned
                            true => Some(texture1.create_view(&TextureViewDescriptor::default())),
                            false => Some(texture2.create_view(&TextureViewDescriptor::default())),
                        }
                    }
                    TextureType::SingleTexture(texture) => {
                        Some(texture.create_view(&TextureViewDescriptor::default()))
                    }
                    TextureType::DescriptorTexture(texture, _) => {
                        Some(texture.create_view(&TextureViewDescriptor::default()))
                    }
                    TextureType::Buffer(_) => None,
                }
            }
            None => None,
        }
    }

    pub fn get_view_swapchain<S>(
        &self,
        name: S,
        context: &RenderContext,
    ) -> Option<(wgpu::TextureView, wgpu::TextureView)>
    where
        S: Into<String>,
    {
        if let Some(TextureType::TextureSwapChain(texture1, texture2)) =
            self.textures.get(&name.into())
        {
            match context.frame_count % 2 == 0 {
                true => {
                    return Some((
                        texture1.create_view(&TextureViewDescriptor::default()),
                        texture2.create_view(&TextureViewDescriptor::default()),
                    ));
                }
                false => {
                    return Some((
                        texture2.create_view(&TextureViewDescriptor::default()),
                        texture1.create_view(&TextureViewDescriptor::default()),
                    ));
                }
            }
        }

        None
    }

    pub fn get_view_descriptor<S>(
        &self,
        name: S,
        descriptor: &TextureViewDescriptor,
        context: &RenderContext,
    ) -> Option<wgpu::TextureView>
    where
        S: Into<String>,
    {
        match self.textures.get(&name.into()) {
            Some(texture_type) => {
                match texture_type {
                    TextureType::TextureSwapChain(texture1, texture2) => {
                        match context.frame_count % 2 == 0 {
                            // TODO ensure most recent texture is returned
                            true => Some(texture1.create_view(descriptor)),
                            false => Some(texture2.create_view(descriptor)),
                        }
                    }
                    TextureType::SingleTexture(texture) => Some(texture.create_view(descriptor)),
                    TextureType::DescriptorTexture(texture, _) => {
                        Some(texture.create_view(descriptor))
                    }
                    TextureType::Buffer(_) => None,
                }
            }
            None => None,
        }
    }

    pub fn get_info<S>(&self, name: S, context: &RenderContext) -> Option<TextureInfo>
    where
        S: Into<String>,
    {
        match self.textures.get(&name.into()) {
            Some(texture_type) => match texture_type {
                TextureType::TextureSwapChain(_, _) | TextureType::SingleTexture(_) => {
                    let size = context.window.inner_size();
                    Some(TextureInfo {
                        size: (size.width, size.height, 1),
                        mip_levels: 1,
                    })
                }
                TextureType::DescriptorTexture(_, info) => Some(info.clone()),
                TextureType::Buffer(_) => None,
            },
            None => None,
        }
    }

    pub fn get_buffer<S>(&self, name: S, context: &RenderContext) -> Option<&wgpu::Buffer>
    where
        S: Into<String>,
    {
        if let Some(TextureType::Buffer(buffer)) = self.textures.get(&name.into()) {
            return Some(buffer);
        }

        None
    }

    pub fn get_default_sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    fn texture_size(context: &RenderContext) -> wgpu::Extent3d {
        let size = context.window.inner_size();

        wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        }
    }
}
