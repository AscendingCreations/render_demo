use crate::graphics::MapVertex;
use image::{self, ImageBuffer};

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum MapLayers {
    Ground,
    Mask,
    //Mask 2 is the Z layer spacer for bridges.
    Mask2,
    Anim1,
    Anim2,
    Anim3,
    //always above player. \/
    Fringe,
    Fringe2,
    Count,
}

impl MapLayers {
    pub fn indexed_layerz(layer: u32) -> f32 {
        match layer {
            0 => 8.0,
            1 => 7.0,
            2 => 6.0,
            3 => 5.0,
            4 => 4.0,
            5 => 3.0,
            6 => 2.0,
            _ => 1.0,
        }
    }

    pub fn layerz(layer: MapLayers) -> f32 {
        //for use with Player Z map done shader side.
        match layer {
            MapLayers::Ground => 8.0,
            MapLayers::Mask => 7.0,
            MapLayers::Mask2 => 6.0,
            MapLayers::Anim1 => 5.0,
            MapLayers::Anim2 => 4.0,
            MapLayers::Anim3 => 3.0,
            MapLayers::Fringe => 2.0,
            MapLayers::Fringe2 | MapLayers::Count => 1.0,
        }
    }
}

pub struct Map {
    pub id: u32,       //Order ID so we can find the map.
    pub map_id: u32,   //so we know what map it is linked too currently.
    pub pos: [i32; 2], //its render position. used to build the Vertex Array for this map.
    //image is for modifying the Buffer R = Texture location, G = Texture layer, B = Hue, A = Alpha
    pub image: ImageBuffer<image::Rgba<u32>, Vec<u32>>,
    //Texture Data to hold image in GPU
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    //set to know the image array ID within the shader.
    pub array_id: u32,
    //vertex array in bytes. Does not need to get changed exept on map switch and location change.
    pub bytes: Vec<u8>,
    //if the image changed
    pub img_changed: bool,
    //if the location or map array id changed.
    pub changed: bool,
}

impl Map {
    pub fn new(device: &wgpu::Device) -> Self {
        let image = ImageBuffer::new(32, 256);
        let extent = wgpu::Extent3d {
            width: 32,
            height: 256,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Map Texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Map Texture"),
            format: Some(wgpu::TextureFormat::Rgba32Uint),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: std::num::NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: std::num::NonZeroU32::new(1),
        });

        Self {
            id: 0,
            map_id: 0,
            pos: [0; 2],
            array_id: 0,
            bytes: Vec::new(),
            image,
            texture,
            texture_view,
            img_changed: true,
            changed: true,
        }
    }

    pub fn create_quad(&mut self) {
        let (x, y, w, h) = (
            self.pos[0] as f32,
            self.pos[1] as f32,
            self.pos[0].saturating_add(512) as f32,
            self.pos[1].saturating_add(512) as f32,
        );

        let mut buffer = Vec::new();

        for i in 0..8 {
            let z = MapLayers::indexed_layerz(i);
            let mut vertices = vec![
                MapVertex {
                    position: [x, y, z],
                    tex_coord: [0.0, 512.0, self.array_id as f32],
                },
                MapVertex {
                    position: [w, y, z],
                    tex_coord: [512.0, 512.0, self.array_id as f32],
                },
                MapVertex {
                    position: [w, h, z],
                    tex_coord: [512.0, 0.0, self.array_id as f32],
                },
                MapVertex {
                    position: [x, h, z],
                    tex_coord: [0.0, 0.0, self.array_id as f32],
                },
            ];

            buffer.append(&mut vertices);
        }

        self.bytes = bytemuck::cast_slice(&buffer).to_vec();
    }

    pub fn set_tile(&mut self, x: u32, y: u32, id: u32, layer: u32, hue: u32, alpha: u32) {
        if x >= 32 || y >= 256 {
            return;
        }

        let pixel = self.image.get_pixel_mut(x, y);
        *pixel = image::Rgba([id, layer, hue, alpha]);
        self.img_changed = true;
    }

    pub fn get_tile(&mut self, x: u32, y: u32) -> (u32, u32, u32, u32) {
        if x >= 32 || y >= 256 {
            return (0, 0, 0, 0);
        }
        let pixel = self.image.get_pixel(x, y);
        let image::Rgba(data) = *pixel;
        (data[0], data[1], data[2], data[3])
    }

    //used to check and update the vertex array.
    pub fn update(&mut self, queue: &wgpu::Queue) {
        //if pos or tex_pos or color changed.
        if self.img_changed {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                &bytemuck::cast_slice(self.image.as_raw()).to_vec(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(512),
                    rows_per_image: std::num::NonZeroU32::new(256),
                },
                wgpu::Extent3d {
                    width: 32,
                    height: 256,
                    depth_or_array_layers: 1,
                },
            );

            self.img_changed = false;
        }

        if self.changed {
            self.create_quad();
            self.changed = false;
        }
    }
}
