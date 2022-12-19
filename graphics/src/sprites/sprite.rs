use crate::{Allocation, Color, SpriteVertex};
use std::cmp;

/// rendering data for all sprites.
/// not to be confused with Actual NPC or Player data.
pub struct Sprite {
    pub pos: [i32; 3],
    pub hw: [u16; 2],
    // used for static offsets or animation Start positions
    pub uv: [u16; 4],
    /// Color dah  number / 255.
    pub color: Color,
    // frames, frames_per_row: this will cycle thru
    // frames per rox at the uv start.
    pub frames: [u16; 2],
    /// in millsecs 1000 = 1sec
    pub switch_time: u32,
    /// turn on animation if set.
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
            frames: [0; 2],
            switch_time: 0,
            animate: false,
            color: Color::rgba(255, 255, 255, 255),
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
            self.hw[0] as f32,
            self.hw[1] as f32,
        );

        let allocation = match &self.texture {
            Some(allocation) => allocation,
            None => return,
        };

        let (u, v, width, height) = allocation.rect();
        let (u, v, width, height) = (
            self.uv[0].saturating_add(u as u16),
            self.uv[1].saturating_add(v as u16),
            cmp::min(self.uv[2], width as u16),
            cmp::min(self.uv[3], height as u16),
        );

        let animate = u32::from(self.animate);

        let instance = vec![SpriteVertex {
            position: [x, y, self.pos[2] as f32],
            hw: [w, h],
            tex_data: [u, v, width, height],
            color: self.color.0,
            frames: self.frames,
            animate,
            time: self.switch_time,
            layer: allocation.layer as i32,
        }];

        self.bytes = bytemuck::cast_slice(&instance).to_vec();
        self.changed = false;
    }

    pub fn new(texture: Allocation) -> Self {
        Self {
            texture: Some(texture),
            ..Default::default()
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
