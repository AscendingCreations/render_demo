use crate::{
    DrawOrder, GpuRenderer, Index, MapRenderer, MapTextures, MapVertex,
    OrderedIndex, Vec2, Vec3,
};
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
    /// image is for modifying the Buffer R = Texture location, G = Texture layer, B = None, A = Alpha
    /// This is a special texture file for use to render each tile in the shader.
    pub image: ImageBuffer<image::Rgba<u32>, Vec<u32>>,
    /// set to know the image array ID within the shader.
    pub layer: Option<u32>,
    /// vertex array in bytes. Does not need to get changed exept on map switch and location change.
    pub lowerstore_id: Index,
    /// vertex array in bytes for fringe layers.
    pub upperstore_id: Index,
    /// the draw order of the maps. created when update is called.
    pub order: DrawOrder,
    /// Count of how many Filled Tiles Exist. this is to optimize out empty maps in rendering.
    pub filled_tiles: [u8; MapLayers::Count as usize],
    pub tilesize: u32,
    /// if the image changed we need to reupload it to the texture.
    /// we can also use this to deturmine if we want to unload the map from the
    /// GPU or not. set to false if its been updated and a layer is_some().
    /// if false and added to a map texture array and layer is_some() it will upload
    /// the data into the texture. if true skips updating and uploading if layer is_some().
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
                hw: [32.0 * self.tilesize as f32, 32.0 * self.tilesize as f32],
                layer: self.layer.unwrap() as i32,
                tilesize: self.tilesize,
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

        self.order =
            DrawOrder::new(false, &Vec3::new(self.pos.x, self.pos.y, 1.0), 1);
        self.changed = false;
    }

    /// Sets the map rendering objects layer so it's texture data can be written to the MapTexture.
    /// If you are using a ton of preloaded maps You only need to run this when you need to render it.
    /// You can only store so many of these in the layer. so it will return None if all layers are in use.
    pub fn init_texture_layer(&mut self, map_texture: &mut MapRenderer) {
        self.layer = map_texture.get_unused_id();
    }

    /// Used to check if the map has a texture layer or not. Can help decide if we want to load it into the GPU or not.
    pub fn has_layer(&self) -> bool {
        self.layer.is_some()
    }

    pub fn get_tile(&mut self, x: u32, y: u32) -> (u32, u32, u32, u32) {
        if x >= 32 || y >= 256 {
            return (0, 0, 0, 0);
        }
        let pixel = self.image.get_pixel(x, y);
        let image::Rgba(data) = *pixel;
        (data[0], data[1], data[2], data[3])
    }

    pub fn new(renderer: &mut GpuRenderer, tilesize: u32) -> Self {
        let image = ImageBuffer::new(32, 256);

        Self {
            pos: Vec2::default(),
            layer: None,
            lowerstore_id: renderer.new_buffer(),
            upperstore_id: renderer.new_buffer(),
            filled_tiles: [0; MapLayers::Count as usize],
            image,
            order: DrawOrder::default(),
            tilesize,
            img_changed: true,
            changed: true,
        }
    }

    // this sets the tile's Id within the texture,
    //layer within the texture array and Alpha for its transparency.
    // This allows us to loop through the tiles Shader side efficiently.
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
    ) -> Option<(OrderedIndex, OrderedIndex)> {
        if let Some(layer) = self.layer {
            // if pos or tex_pos or color changed.
            if self.img_changed {
                map_textures.update(renderer, layer, self.image.as_raw());
                self.img_changed = false;
            }

            if self.changed {
                self.create_quad(renderer);
            }

            Some((
                OrderedIndex::new(self.order, self.lowerstore_id, 0),
                OrderedIndex::new(self.order, self.upperstore_id, 0),
            ))
        } else {
            None
        }
    }
}
