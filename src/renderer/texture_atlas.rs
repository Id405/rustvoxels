use std::collections::HashMap;

use wgpu::TextureViewDescriptor;
use winit::dpi::PhysicalSize;

use super::RenderContext;

#[derive(Debug)]
pub enum TextureType {
    TextureSwapChain(wgpu::Texture, wgpu::Texture),
    SingleTexture(wgpu::Texture),
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

        self.textures
            .insert(name, TextureType::SingleTexture(texture));
    }

    pub fn resize(&mut self, context: &RenderContext) {
        let keys = self.textures.keys().map(|x| x.clone()).collect::<Vec<_>>();
        for name in keys {
            match self.textures.get(&name).unwrap() {
                TextureType::TextureSwapChain(_, _) => self.register_swapchain(name, context),
                TextureType::SingleTexture(_) => self.register(name, context),
            }
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
