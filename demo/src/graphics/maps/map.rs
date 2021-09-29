use crate::graphics::{allocation::Allocation, TileVertex};
use image::ImageBuffer;
use std::cmp;

#[allow(dead_code)]
#[derive(Copy, Clone, Display)]
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

#[derive(Copy, Clone)]
pub struct Map {
    //image is for modifng the Buffer R = Texture location, G = Texture layer, B = Hue, A = Alpha
    pub image: ImageBuffer,
    //Texture Data to hold image in GPU
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    //if anything got updated we need to update the texture data
    pub changed: bool,
}

impl Default for Map {
    fn default() -> Self {
        let image = ImageBuffer::new(96, 768);
        let extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Map Texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            image,
            texture,
            texture_view,
            changed: true,
        }
    }
}

impl Map {
    pub fn new() -> Self {
        Map::default()
    }

    pub fn set_tile(&mut self, x: u32, y: u32, id: u32, layer: u32, hue: f32, alpha: f32) {
        let pixel = self.image.get_pixel_mut(x, y);
        *pixel = Image::Rgba([id as f32, layer as f32, hue, alpha]);
        self.changed = true;
    }

    pub fn get_tile(&mut self, x: u32, y: u32) -> (u32, u32, f32, f32) {
        let pixel = self.image.get_pixel(x, y);
        let image::Rgba(data) = *pixel;
        (data[0] as u32, data[1] as u32, data[2] as f32, data[3] as f32)
    }

    //used to check and update the vertex array.
    pub fn update(&mut self, queue: &wgpu::Queue) {
        //if pos or tex_pos or color changed.
        if self.changed {
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &self.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &image.into_raw(),
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: std::num::NonZeroU32::new(16 * 96),
                        rows_per_image: std::num::NonZeroU32::new(768),
                    },
                    wgpu::Extent3d {
                        96,
                        768,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
    }
}
