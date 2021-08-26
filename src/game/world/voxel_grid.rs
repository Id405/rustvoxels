use std::convert::TryInto;

pub struct VoxelGrid {
    width: usize,
    height: usize,
    length: usize,
    pub data: Vec<u8>,
    texture: Option<wgpu::Texture>,
    texture_size: Option<wgpu::Extent3d>,
}

impl VoxelGrid {
    pub fn from_string(text_data: String) -> VoxelGrid {
        let mut lines = text_data.split("\n").map( |x| x.replace("\r", ""));

        let mut dimensions = lines
            .next()
            .expect("failed to parse scene: unexpected EOF");
        
        let mut dimensions = dimensions
            .split("x")
            .map(|x| {
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
            data: vec![0; width * height * length * 4],
            texture: None,
            texture_size: None,
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
        let (x, z, y) = (pos[0], pos[1], pos[2]);
        let (width, _height, length) = (self.width, self.height, self.length);

        let p = ((x + width * y) + z * width * length) * 4;

        let slice = &mut self.data[p..p + 4];
        slice.iter_mut().enumerate().for_each(|(i, v)| {
            *v = color[i];
        });
    }

    fn set_voxel(
        &mut self,
        pos: [usize; 3],
        color: [u8; 4],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.set_data(pos, color);
        self.write_texture_data(device, queue);
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

    pub fn gen_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.texture_size = Some(wgpu::Extent3d {
            width: self.width as u32,
            height: self.length as u32,
            depth_or_array_layers: self.height as u32,
        });

        self.texture = Some(device.create_texture(&wgpu::TextureDescriptor {
            size: self.texture_size.unwrap(),
            mip_level_count: 4, // (((self.width).min((self.height).min(self.length)) as f32).log2()).floor() as u32,
            label: Some("scene_texture"),
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba8UnormSrgb, //TODO convert data to float
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        }));

        self.write_texture_data(device, queue);
    }

    pub fn as_texture(&self) -> &wgpu::Texture {
        self.texture.as_ref().unwrap()
    }

    fn write_texture_data(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // Thinking out loud, device and queue should public resources
        let texture = match &self.texture {
            Some(texture) => texture,
            None => return,
        };

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: texture,
                mip_level: 4, // use math to determine this once data is float
                origin: wgpu::Origin3d::ZERO,
            },
            &self.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * self.width as u32),
                rows_per_image: std::num::NonZeroU32::new(self.length as u32),
            },
            self.texture_size.unwrap(),
        )
    }
}
