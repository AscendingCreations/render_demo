pub(crate) use crate::graphics::{
    allocation::Allocation, BufferLayout, BufferPass, SpriteVertex,
};
use std::cmp;

/// rendering data for all sprites.
/// not to be confused with Actual NPC or Player data.
pub struct Sprite {
    pub pos: [i32; 3],
    pub hw: [u16; 2],
    pub uv: [u16; 4],
    pub color: [u32; 4],
    pub frames: u16,
    /// in millsecs 1000 = 1sec
    pub switch_time: u32,
    pub animate: bool,
    /// Texture area location in Atlas.
    pub texture: Option<Allocation>,
    pub bytes: Vec<u8>,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            pos: [0; 3],
            hw: [0; 2],
            uv: [0; 4],
            frames: 0,
            switch_time: 0,
            animate: false,
            color: [0, 0, 100, 100],
            texture: None,
            bytes: Vec::new(),
            changed: true,
        }
    }
}

impl Sprite {
    pub fn create_quad(&mut self) {
        let (x, y, w, h) = (
            self.pos[0] as f32,
            self.pos[1] as f32,
            self.pos[0].saturating_add((self.hw[0] - 1) as i32) as f32,
            self.pos[1].saturating_add((self.hw[1] - 1) as i32) as f32,
        );

        let allocation = match &self.texture {
            Some(allocation) => allocation,
            None => return,
        };

        let (u, v, width, height) = allocation.rect();
        let (u, v, width, height) = (
            u as u16,
            v as u16,
            cmp::min(self.uv[2], width as u16),
            cmp::min(self.uv[3], height as u16),
        );

        let (u1, v1, u2, v2) = (
            self.uv[0].saturating_add(u),
            self.uv[1].saturating_add(v),
            self.uv[0].saturating_add(u).saturating_add(width),
            self.uv[1].saturating_add(v).saturating_add(height),
        );

        let animate = u16::from(self.animate);

        let buffer = vec![
            SpriteVertex {
                position: [x, y, self.pos[2] as f32],
                tex_coord: [u1, v2],
                rg: [self.color[0], self.color[1]],
                ba: [self.color[2] as u16, self.color[3] as u16],
                frames: [self.frames, animate],
                tex_hw: [width, height],
                time: self.switch_time,
                layer: allocation.layer as i32,
            },
            SpriteVertex {
                position: [w, y, self.pos[2] as f32],
                tex_coord: [u2, v2],
                rg: [self.color[0], self.color[1]],
                ba: [self.color[2] as u16, self.color[3] as u16],
                frames: [self.frames, animate],
                tex_hw: [width, height],
                time: self.switch_time,
                layer: allocation.layer as i32,
            },
            SpriteVertex {
                position: [w, h, self.pos[2] as f32],
                tex_coord: [u2, v1],
                rg: [self.color[0], self.color[1]],
                ba: [self.color[2] as u16, self.color[3] as u16],
                frames: [self.frames, animate],
                tex_hw: [width, height],
                time: self.switch_time,
                layer: allocation.layer as i32,
            },
            SpriteVertex {
                position: [x, h, self.pos[2] as f32],
                tex_coord: [u1, v1],
                rg: [self.color[0], self.color[1]],
                ba: [self.color[2] as u16, self.color[3] as u16],
                frames: [self.frames, animate],
                tex_hw: [width, height],
                time: self.switch_time,
                layer: allocation.layer as i32,
            },
        ];

        self.bytes = bytemuck::cast_slice(&buffer).to_vec();
        self.changed = false;
    }

    pub fn new(texture: Allocation) -> Self {
        Self {
            pos: [0; 3],
            hw: [0; 2],
            uv: [0; 4],
            frames: 0,
            switch_time: 0,
            animate: false,
            color: [0, 0, 100, 100],
            texture: Some(texture),
            bytes: Vec::new(),
            changed: true,
        }
    }

    /// used to check and update the vertex array.
    pub fn update(&mut self) -> bool {
        // if pos or tex_pos or color changed.
        if self.changed {
            self.create_quad();
            true
        } else {
            false
        }
    }
}
