use bytemuck::{Pod, Zeroable};
//TODO refactor this entire module
use rayon::prelude::*;
use std::{borrow::Cow, convert::TryInto, io::Write, mem, num::NonZeroU32};
use wgpu::{util::DeviceExt, BufferDescriptor};

use crate::renderer::{RenderContext, ShaderBundle};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TimestampData {
    start: u64,
    end: u64,
}

fn pipeline_statistics_offset(mip_pass_count: usize) -> wgpu::BufferAddress {
    ((mem::size_of::<TimestampData>() * mip_pass_count) as wgpu::BufferAddress)
        .max(wgpu::QUERY_RESOLVE_BUFFER_ALIGNMENT)
}

pub struct VoxelGrid {
    width: usize,
    height: usize,
    length: usize,
    data: Vec<f32>,
    changed_voxels: Vec<[usize; 3]>,
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
            changed_voxels: Vec::new(),
            texture: None,
        };

        for (line_number, line) in lines.enumerate() {
            let values: String = line.chars().filter(|x| *x != ' ').collect();
            let values = values.split(",").map(|x| {
                x.parse::<usize>()
                    .expect("failed to parse scene: expected int in dimension data got str")
            });
            let voxel_data: Vec<usize> = values.collect::<Vec<_>>().try_into().expect(
                format!(
                    "failed to parse scene: not enough data on line {}",
                    line_number + 1
                )
                .as_str(),
            );

            let pos: [usize; 3] = (&voxel_data[0..3]).try_into().unwrap();
            let mut color: [u8; 3] = [125, 125, 125];
            if voxel_data.len() == 6 {
                color = voxel_data[3..6]
                    .iter()
                    .map(|x| *x as u8)
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
            }
            let color: [u8; 4] = [color[0], color[1], color[2], 255];

            voxel_grid.set_data(pos, color);
        }

        voxel_grid
    }

    fn set_data(&mut self, pos: [usize; 3], color: [u8; 4]) {
        self.changed_voxels.push(pos);
        let p = Self::grid_position(pos, (self.width, self.height, self.length));

        let slice = &mut self.data[p..p + 4];
        slice.iter_mut().enumerate().for_each(|(i, v)| {
            *v = (color[i] as f32) / 255.0;
        });
    }

    fn grid_position(pos: [usize; 3], size: (usize, usize, usize)) -> usize {
        let (x, z, y) = (pos[0], pos[1], pos[2]);
        let (width, height, length) = size;
        (x + (width * y) + (z * width * length)) * 4
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
                    format: wgpu::TextureFormat::Rgba32Float,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::STORAGE_BINDING,
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

        let texture_size = wgpu::Extent3d {
            width: self.width as u32,
            height: self.length as u32,
            depth_or_array_layers: self.height as u32,
        };

        unsafe {
            // TODO rework away unsafe
            render_context.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                std::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.data.len() * 4),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(16 * self.width as u32),
                    rows_per_image: std::num::NonZeroU32::new(self.length as u32),
                },
                texture_size,
            );
        }

        println!("generating mipmap levels with compute shader...");

        let compute_bind_group_layout =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D3,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: wgpu::TextureFormat::Rgba32Float,
                                view_dimension: wgpu::TextureViewDimension::D3,
                            },
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            count: None,
                        },
                    ],
                    label: Some("Mipmap Compute Shader Bind Group layout decriptor"),
                });

        let mip_passes = self.get_mip_levels() - 1;

        let compute_pipeline_layout =
            render_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Mipmap Compute Shader Pipeline Layout Descriptor"),
                    bind_group_layouts: &[&compute_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let timestamp = render_context
            .device
            .create_query_set(&wgpu::QuerySetDescriptor {
                label: None,
                count: (mip_passes) * 2,
                ty: wgpu::QueryType::Timestamp,
            });

        let timestamp_period = render_context.queue.get_timestamp_period();

        let pipeline_statistics =
            render_context
                .device
                .create_query_set(&wgpu::QuerySetDescriptor {
                    label: None,
                    count: mip_passes,
                    ty: wgpu::QueryType::PipelineStatistics(
                        wgpu::PipelineStatisticsTypes::COMPUTE_SHADER_INVOCATIONS,
                    ),
                });

        let data_buffer = render_context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("query buffer"),
                size: pipeline_statistics_offset(mip_passes as usize)
                    + 8 * (mip_passes) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });

        let compute_shader;

        unsafe {
            compute_shader = ShaderBundle::compute_from_path("mipmap")
                .create_compute_shader_module_spirv(render_context);
        }

        let compute_pipeline =
            render_context
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Compute pipeline"),
                    layout: Some(&compute_pipeline_layout),
                    module: &compute_shader,
                    entry_point: "main",
                });

        let mut encoder = render_context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let views = (0..self.get_mip_levels())
            .map(|mip| {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("mip"),
                    format: None,
                    dimension: None,
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: mip,
                    mip_level_count: NonZeroU32::new(1),
                    base_array_layer: 0,
                    array_layer_count: None,
                })
            })
            .collect::<Vec<_>>();

        for level in 1..self.get_mip_levels() as usize {
            let div_factor = (2_usize).pow(level as u32);
            let (width, height, length) = (
                self.width / div_factor,
                self.height / div_factor,
                self.length / div_factor,
            );

            let texture_view = views.get(level - 1).unwrap();

            let destination_texture = views.get(level).unwrap();

            let bind_group = render_context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("Mipmap Compute Shader Bind Group level {}", level)),
                    layout: &compute_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&destination_texture),
                        },
                    ],
                });

            {
                let mut compute_pass =
                    encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
                println!("{}", (level - 1) * 2);
                compute_pass.write_timestamp(&timestamp, (level - 1) as u32 * 2);
                compute_pass
                    .begin_pipeline_statistics_query(&pipeline_statistics, (level - 1) as u32);

                compute_pass.set_pipeline(&compute_pipeline);
                compute_pass.set_bind_group(0, &bind_group, &[]);
                println!("{}", (level - 1) * 2 + 1);
                compute_pass.dispatch(
                    ((width as u32) / 8).max(1),
                    ((length as u32) / 8).max(1),
                    ((height as u32) / 8).max(1),
                );

                compute_pass.write_timestamp(&timestamp, (level - 1) as u32 * 2 + 1);
                compute_pass.end_pipeline_statistics_query();
            }
        }

        encoder.resolve_query_set(&timestamp, 0..mip_passes * 2, &data_buffer, 0);
        encoder.resolve_query_set(
            &pipeline_statistics,
            0..mip_passes,
            &data_buffer,
            pipeline_statistics_offset(mip_passes as usize * 2),
        );

        render_context.queue.submit(Some(encoder.finish()));

        let _ = data_buffer.slice(..).map_async(wgpu::MapMode::Read);

        render_context.device.poll(wgpu::Maintain::Wait);

        let timestamp_view = data_buffer
            .slice(..pipeline_statistics_offset(mip_passes as usize))
            .get_mapped_range();
        let pipeline_stats_view = data_buffer
            .slice(pipeline_statistics_offset(mip_passes as usize)..)
            .get_mapped_range();

        let timestamp_data: Vec<TimestampData> = bytemuck::pod_collect_to_vec(&timestamp_view);

        let pipeline_stats_data: Vec<u64> = bytemuck::pod_collect_to_vec(&pipeline_stats_view);

        let mut total_mip_cost = 0.0;

        // Iterate over the data
        for (idx, (timestamp, pipeline)) in timestamp_data
            .iter()
            .zip(pipeline_stats_data.iter())
            .enumerate()
        {
            // Figure out the timestamp differences and multiply by the period to get nanoseconds
            let nanoseconds = (timestamp.end - timestamp.start) as f32 * timestamp_period;
            // Nanoseconds is a bit small, so lets use microseconds.
            let microseconds = nanoseconds / 1000.0;

            total_mip_cost += microseconds / 1000.0;

            // Print the data!
            println!(
                "Generating mip level {} took {:.3} μs and called the compute shader {} times",
                idx + 1,
                microseconds,
                pipeline
            );
        }

        println!(
            "Generating mipmaps took {:.3} ms in total",
            total_mip_cost
        )
    }

    pub fn get_mip_levels(&self) -> u32 {
        ((((self.width).min((self.height).min(self.length)) as f32).log2()).floor() - 0.0).max(0.0)
            as u32
            - 1
    }
}
