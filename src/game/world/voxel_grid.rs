//TODO refactor this entire module
use std::{borrow::Cow, convert::TryInto};

use crate::renderer::RenderContext;

pub struct VoxelGrid {
    width: usize,
    height: usize,
    length: usize,
    data: Vec<f32>,
    texture: Option<wgpu::Texture>,
}

impl VoxelGrid {
    pub fn from_string(text_data: String) -> VoxelGrid {
        let mut lines = text_data.split("\n").map(|x| x.replace("\r", ""));

        let dimensions = lines.next().expect("failed to parse scene: unexpected EOF");

        let mut dimensions = dimensions.split("x").map(|x| {
            x.trim()
                .parse::<usize>()
                .expect("failed to parse scene: expected int in dimension data got str")
        });

        let width = dimensions
            .next()
            .expect("failed to parse scene dimensions: not enough dimensions provided");
        let height = dimensions
            .next()
            .expect("failed to parse scene dimensions: not enough dimensions provided");
        let length = dimensions
            .next()
            .expect("failed to parse scene dimensions: not enough dimensions provided");

        let mut voxel_grid = VoxelGrid {
            width,
            height,
            length,
            data: vec![0.0; width * height * length * 4],
            texture: None,
        };

        for (line_number, line) in lines.enumerate() {
            let values: String = line.chars().filter(|x| *x != ' ').collect();
            let values = values.split(",").map(|x| {
                x.parse::<usize>()
                    .expect("failed to parse scene: expected int in dimension data got str")
            });
            let voxel_data: [usize; 6] = values.collect::<Vec<_>>().try_into().expect(
                format!(
                    "failed to parse scene: not enough data on line {}",
                    line_number + 1
                )
                .as_str(),
            );

            let pos: [usize; 3] = (&voxel_data[0..3]).try_into().unwrap();
            let color: [u8; 3] = voxel_data[3..6]
                .iter()
                .map(|x| *x as u8)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            let color: [u8; 4] = [color[0], color[1], color[2], 255];

            voxel_grid.set_data(pos, color);
        }

        voxel_grid
    }

    fn set_data(&mut self, pos: [usize; 3], color: [u8; 4]) {
        let p = Self::grid_position(pos, (self.width, self.height, self.length));

        let slice = &mut self.data[p..p + 4];
        slice.iter_mut().enumerate().for_each(|(i, v)| {
            *v = (color[i] as f32) / 255.0;
        });
    }

    fn grid_position(pos: [usize; 3], size: (usize, usize, usize)) -> usize {
        let (x, z, y) = (pos[0], pos[1], pos[2]);
        let (width, height, length) = size;
        ((x + width * y) + z * width * length) * 4
    }

    fn set_voxel(&mut self, pos: [usize; 3], color: [u8; 4], render_context: &RenderContext) {
        self.set_data(pos, color);
        self.write_texture_data(render_context);
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn length(&self) -> usize {
        // This should really get a refactor, along with the shader to use a more typical video game coordinate system instead of a math based one
        self.length
    }

    pub fn gen_texture(&mut self, render_context: &RenderContext) {
        let texture_size = Some(wgpu::Extent3d {
            width: self.width as u32,
            height: self.length as u32,
            depth_or_array_layers: self.height as u32,
        });

        self.texture = Some(
            render_context
                .device
                .create_texture(&wgpu::TextureDescriptor {
                    size: texture_size.unwrap(),
                    mip_level_count: self.get_mip_levels(),
                    label: Some("scene_texture"),
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D3,
                    format: wgpu::TextureFormat::Rgba32Float, //TODO convert data to float
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                }),
        );

        self.write_texture_data(render_context);
    }

    pub fn as_texture(&self) -> &wgpu::Texture {
        self.texture.as_ref().unwrap()
    }

    pub fn write_texture_data(&self, render_context: &RenderContext) {
        let texture = match &self.texture {
            Some(texture) => texture,
            None => return,
        };

        let mut data = Vec::new();

        data.push(self.data.clone());

        for level in 1..self.get_mip_levels() as usize {
            let div_factor = (2_usize).pow(level as u32);
            let (width, height, length) = (
                self.width / div_factor,
                self.height / div_factor,
                self.length / div_factor,
            );
            let mut level_data = vec![0.0; width * height * length * 4];

            for x in 0..width {
                for z in 0..height {
                    for y in 0..length {
                        let pos = [x, z, y];
                        let mut values: Vec<Vec<_>> = Vec::new();

                        for dx in 0..1 {
                            for dz in 0..1 {
                                for dy in 0..1 {
                                    let mut dpos = pos.map(|x| x * 2);
                                    dpos = [dpos[0] + dx, dpos[1] + dy, dpos[2] + dz];

                                    let p = Self::grid_position(dpos, (width, height, length));

                                    values.push(
                                        data[level - 1][p..p + 4].to_vec().try_into().unwrap(),
                                    );
                                }
                            }
                        }

                        // let values: Vec<_> = values.iter().filter(|x| x[3] != 0.0).collect();

                        let average: [f32; 4] = values
                            .iter()
                            .fold([0.0; 4], |sum, val| {
                                [
                                    sum[0] + val[0],
                                    sum[1] + val[1],
                                    sum[2] + val[2],
                                    sum[3] + val[3],
                                ]
                            })
                            .map(|x| x / values.len() as f32);

                        let p = Self::grid_position(pos, (width, height, length));

                        let slice = &mut level_data[p..p + 4];
                        slice.iter_mut().enumerate().for_each(|(i, v)| {
                            *v = (average[i] as f32) / 255.0;
                        });
                    }
                }
            }
            data.push(level_data);
        }

        unsafe {
            //TODO rework away unsafe
            for (mipmap_level, mipmap_data) in data.iter().enumerate() {
                let div_factor = (2_usize).pow(mipmap_level as u32);
                let (width, height, length) = (
                    self.width / div_factor,
                    self.height / div_factor,
                    self.length / div_factor,
                );

                println!("{}, {}x{}x{}", mipmap_level, width, height, length);

                let texture_size = wgpu::Extent3d {
                    width: width as u32,
                    height: length as u32,
                    depth_or_array_layers: height as u32,
                };

                render_context.queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: texture,
                        mip_level: mipmap_level as u32,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    std::slice::from_raw_parts(
                        mipmap_data.as_ptr() as *const u8,
                        mipmap_data.len() * 4,
                    ),
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: std::num::NonZeroU32::new(16 * width as u32),
                        rows_per_image: std::num::NonZeroU32::new(length as u32),
                    },
                    texture_size,
                );
            }
        }
    }

    pub fn get_mip_levels(&self) -> u32 {
        // (((self.width).min((self.height).min(self.length)) as f32).log2()).floor() as u32
        1
    }
}
