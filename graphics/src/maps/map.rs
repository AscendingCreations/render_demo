use crate::{
    DrawOrder, GpuRenderer, Index, MapVertex, OrderedIndex, Vec2, Vec3,
};
use cosmic_text::Color;

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
            0 => 10.0,
            1 => 9.0,
            2 => 8.0,
            3 => 7.0,
            4 => 6.0,
            5 => 5.0,
            6 => 3.0,
            _ => 2.0,
        }
    }

    pub fn layerz(layer: MapLayers) -> f32 {
        // for use with Player Z map done shader side.
        match layer {
            MapLayers::Ground => 10.0,
            MapLayers::Mask => 9.0,
            MapLayers::Mask2 => 8.0,
            MapLayers::Anim1 => 7.0,
            MapLayers::Anim2 => 6.0,
            MapLayers::Anim3 => 5.0,
            MapLayers::Fringe => 3.0,
            MapLayers::Fringe2 | MapLayers::Count => 2.0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct TileData {
    pub texture_id: u32,
    pub texture_layer: u8,
    pub color: Color,
}

impl Default for TileData {
    fn default() -> Self {
        Self {
            texture_id: 0,
            texture_layer: 0,
            color: Color::rgba(255, 255, 255, 255),
        }
    }
}

pub struct Map {
    /// X, Y, GroupID for loaded map.
    /// Add this to the higher up Map struct.
    /// pub world_pos: Vec3,
    /// its render position. within the screen.
    pub pos: Vec2,
    // tiles per layer.
    pub tiles: [TileData; 8192],
    /// vertex array in bytes. Does not need to get changed exept on map switch and location change.
    pub lowerstore_id: Index,
    /// vertex array in bytes for fringe layers.
    pub upperstore_id: Index,
    /// the draw order of the maps. created when update is called.
    pub order: DrawOrder,
    /// count if any Filled Tiles Exist. this is to optimize out empty maps in rendering.
    pub filled_tiles: [u16; MapLayers::Count as usize],
    // The size of the Tile to render. for spacing tiles out upon
    // vertex creation. Default will be 20.
    pub tilesize: u32,
    // Used to deturmine if the map can be rendered or if its just a preload.
    pub can_render: bool,
    /// if the position or a tile gets changed.
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

            for x in 0..32 {
                for y in 0..32 {
                    let tile =
                        &self.tiles[(x + (y * 32) + (i * 1024)) as usize];

                    let map_vertex = MapVertex {
                        position: [
                            self.pos.x + (x * self.tilesize) as f32,
                            self.pos.y + (y * self.tilesize) as f32,
                            z,
                        ],
                        tilesize: self.tilesize as f32,
                        texture_id: tile.texture_id as f32,
                        texture_layer: tile.texture_layer as f32,
                        color: tile.color.0,
                    };

                    if i >= 6 {
                        upperbuffer.push(map_vertex);
                    } else {
                        lowerbuffer.push(map_vertex);
                    }
                }
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

    pub fn new(renderer: &mut GpuRenderer, tilesize: u32) -> Self {
        Self {
            tiles: [TileData::default(); 8192],
            pos: Vec2::default(),
            lowerstore_id: renderer.new_buffer(),
            upperstore_id: renderer.new_buffer(),
            filled_tiles: [0; MapLayers::Count as usize],
            order: DrawOrder::default(),
            tilesize,
            can_render: false,
            changed: true,
        }
    }

    pub fn get_tile(&self, pos: (u32, u32, u32)) -> TileData {
        assert!(
            pos.0 < 32 || pos.1 < 32 || pos.2 < 8,
            "pos is invalid. X < 32, y < 256, z < 8"
        );

        self.tiles[(pos.0 + (pos.1 * 32) + (pos.2 * 1024)) as usize]
    }

    // this sets the tile's Id within the texture,
    //layer within the texture array and Alpha for its transparency.
    // This allows us to loop through the tiles Shader side efficiently.
    pub fn set_tile(&mut self, pos: (u32, u32, u32), tile: TileData) {
        if pos.0 >= 32 || pos.1 >= 32 || pos.2 >= 8 {
            return;
        }
        let tilepos = (pos.0 + (pos.1 * 32) + (pos.2 * 1024)) as usize;
        let current_tile = self.tiles[tilepos];

        if (current_tile.texture_id > 0 || current_tile.color.a() > 0)
            && (tile.color.a() == 0 || tile.texture_id == 0)
        {
            self.filled_tiles[pos.2 as usize] =
                self.filled_tiles[pos.2 as usize].saturating_sub(1);
        } else if tile.color.a() > 0 || tile.texture_id > 0 {
            self.filled_tiles[pos.2 as usize] =
                self.filled_tiles[pos.2 as usize].saturating_add(1);
        }

        self.tiles[tilepos] = tile;
        self.changed = true;
    }

    /// used to check and update the vertex array or Texture witht he image buffer.
    pub fn update(
        &mut self,
        renderer: &mut GpuRenderer,
    ) -> Option<(OrderedIndex, OrderedIndex)> {
        if self.can_render {
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
