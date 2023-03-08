use crate::{GpuRenderer, Index, MapTextures, MapVertex, Vec2};
use image::{self, ImageBuffer};

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum MapLayers {
    Ground,
    Mask,
    /// Mask 2 is the Z layer spacer for bridges.
    Mask2,
    Anim1,
    Anim2,
    Anim3,
    /// always above player. \/
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
        // for use with Player Z map done shader side.
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
    /// X, Y, GroupID for loaded map.
    /// Add this to the higher up Map struct.
    ///pub world_pos: Vec3,
    /// its render position. within the screen.
    pub pos: Vec2,
    /// image is for modifying the Buffer R = Texture location, G = Texture layer, B = Hue, A = Alpha
    pub image: ImageBuffer<image::Rgba<u32>, Vec<u32>>,
    /// set to know the image array ID within the shader.
    pub layer: u32,
    /// vertex array in bytes. Does not need to get changed exept on map switch and location change.
    pub lowerstore_id: Index,
    /// vertex array in bytes for fringe layers.
    pub upperstore_id: Index,
    /// Count of how many Filled Tiles Exist. this is to optimize out empty maps in rendering.
    pub filled_tiles: [u8; MapLayers::Count as usize],
    /// if the image changed we need to reupload it to the texture.
    pub img_changed: bool,
    /// if the location or map array id changed. to rebuild the vertex buffer.
    pub changed: bool,
}

impl Map {
    pub fn create_quad(&mut self, renderer: &mut GpuRenderer) {
        let mut lowerbuffer = Vec::new();
        let mut upperbuffer = Vec::new();

        for i in 0..8 {
            let z = MapLayers::indexed_layerz(i);

            if self.filled_tiles[i as usize] == 0 {
                continue;
            }

            let map_vertex = MapVertex {
                position: [self.pos.x, self.pos.y, z], //2,3
                hw: [512.0, 512.0],
                layer: self.layer as i32,
            };

            if i >= 6 {
                upperbuffer.push(map_vertex);
            } else {
                lowerbuffer.push(map_vertex);
            }
        }

        if let Some(store) = renderer.get_buffer_mut(&self.lowerstore_id) {
            store.store = bytemuck::cast_slice(&lowerbuffer).to_vec();
            store.changed = true;
        }

        if let Some(store) = renderer.get_buffer_mut(&self.upperstore_id) {
            store.store = bytemuck::cast_slice(&upperbuffer).to_vec();
            store.changed = true;
        }

        self.changed = false;
    }

    pub fn get_tile(&mut self, x: u32, y: u32) -> (u32, u32, u32, u32) {
        if x >= 32 || y >= 256 {
            return (0, 0, 0, 0);
        }
        let pixel = self.image.get_pixel(x, y);
        let image::Rgba(data) = *pixel;
        (data[0], data[1], data[2], data[3])
    }

    pub fn new(renderer: &mut GpuRenderer) -> Self {
        let image = ImageBuffer::new(32, 256);

        Self {
            pos: Vec2::default(),
            layer: 0,
            lowerstore_id: renderer.new_buffer(),
            upperstore_id: renderer.new_buffer(),
            filled_tiles: [0; MapLayers::Count as usize],
            image,
            img_changed: true,
            changed: true,
        }
    }

    pub fn set_tile(
        &mut self,
        pos: (u32, u32, u32),
        id: u32,
        layer: u32,
        alpha: u32,
    ) {
        if pos.0 >= 32 || pos.1 >= 32 || pos.2 >= 8 {
            return;
        }

        let pixel = self.image.get_pixel_mut(pos.0, pos.1 + (pos.2 * 32));
        *pixel = image::Rgba([id, layer, 0, alpha]);

        if alpha == 0 {
            self.filled_tiles[pos.2 as usize] =
                self.filled_tiles[pos.2 as usize].saturating_sub(1);
        } else {
            self.filled_tiles[pos.2 as usize] =
                self.filled_tiles[pos.2 as usize].saturating_add(1);
        }

        self.img_changed = true;
    }

    /// used to check and update the vertex array or Texture witht he image buffer.
    pub fn update(
        &mut self,
        renderer: &mut GpuRenderer,
        map_textures: &mut MapTextures,
    ) -> (Index, Index) {
        // if pos or tex_pos or color changed.
        if self.img_changed {
            map_textures.update(renderer, self.layer, self.image.as_raw());
            self.img_changed = false;
        }

        if self.changed {
            self.create_quad(renderer);
        }

        (self.lowerstore_id, self.upperstore_id)
    }
}
