use crate::graphics::{allocation::Allocation, SpriteVertex};
use std::cmp;

//rendering data for all sprites.
//not to be confused with Actual NPC or Player data.
pub struct Sprite {
    pub pos: [i32; 3],
    pub hw: [u32; 2],
    pub uv: [u32; 4],
    pub color: [u32; 4],
    //Texture area location in Atlas.
    pub texture: Option<Allocation>,
    pub bytes: Vec<u8>,
    //if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            pos: [0; 3],
            hw: [0; 2],
            uv: [0; 4],
            color: [0, 0, 100, 100],
            texture: None,
            bytes: Vec::new(),
            changed: true,
        }
    }
}

impl Sprite {
    pub fn new(texture: Allocation) -> Self {
        Self {
            pos: [0; 3],
            hw: [0; 2],
            uv: [0; 4],
            color: [0, 0, 100, 100],
            texture: Some(texture),
            bytes: Vec::new(),
            changed: true,
        }
    }

    pub fn create_quad(&mut self) {
        let (x, y, w, h) = (
            self.pos[0] as f32,
            self.pos[1] as f32,
            self.pos[0].saturating_add(self.hw[0] as i32) as f32,
            self.pos[1].saturating_add(self.hw[1] as i32) as f32,
        );

        let allocation = match &self.texture {
            Some(allocation) => allocation,
            None => return,
        };

        let (u, v, width, height) = allocation.rect();
        let (width, height) = (cmp::min(self.uv[2], width), cmp::min(self.uv[3], height));

        let (u1, v1, u2, v2) = (
            self.uv[0].saturating_add(u) as f32,
            self.uv[1].saturating_add(v) as f32,
            self.uv[0].saturating_add(u).saturating_add(width) as f32,
            self.uv[1].saturating_add(v).saturating_add(height) as f32,
        );

        let buffer = vec![
            SpriteVertex {
                position: [x, y, self.pos[2] as f32],
                tex_coord: [u1, v2, allocation.layer as f32],
                color: self.color,
            },
            SpriteVertex {
                position: [w, y, self.pos[2] as f32],
                tex_coord: [u2, v2, allocation.layer as f32],
                color: self.color,
            },
            SpriteVertex {
                position: [w, h, self.pos[2] as f32],
                tex_coord: [u2, v1, allocation.layer as f32],
                color: self.color,
            },
            SpriteVertex {
                position: [x, h, self.pos[2] as f32],
                tex_coord: [u1, v1, allocation.layer as f32],
                color: self.color,
            },
        ];

        self.bytes = bytemuck::cast_slice(&buffer).to_vec();
        self.changed = false;
    }

    //used to check and update the vertex array.
    pub fn update(&mut self) {
        //if pos or tex_pos or color changed.
        if self.changed {
            self.create_quad();
        }
    }
}
