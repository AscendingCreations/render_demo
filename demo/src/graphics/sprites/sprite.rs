use crate::graphics::{allocation::Allocation, Rgba, SpriteVertex};

pub struct Sprite {
    pub pos: [u32; 3],
    pub hw: [u32; 2],
    pub uv: [u32; 4],
    pub layer: u32,
    pub color: Rgba,
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
            layer: 0,
            color: Rgba::new(1.0, 1.0, 1.0, 1.0),
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
            layer: texture.layer as u32,
            color: Rgba::new(1.0, 1.0, 1.0, 1.0),
            texture: Some(texture),
            bytes: Vec::new(),
            changed: true,
        }
    }

    pub fn create_quad(&mut self) {
        let min_x = self.pos[0] as f32;
        let min_y = self.pos[1] as f32;
        let max_x = self.pos[0].saturating_add(self.hw[0]) as f32;
        let max_y = self.pos[1].saturating_add(self.hw[1]) as f32;

        let (width, height) = if let Some(allocation) = &self.texture {
            let (h, w) = allocation.size();
            (h, w)
        } else {
            (1, 1)
        };

        let (x, y) = if let Some(allocation) = &self.texture {
            let (x, y) = allocation.position();
            (x, y)
        } else {
            (0, 0)
        };

        let width = if width > self.uv[2] {
            self.uv[2]
        } else {
            width
        };

        let height = if height > self.uv[3] {
            self.uv[3]
        } else {
            height
        };

        let uv_x = self.uv[0].saturating_add(x) as f32 / 2048.0;
        let uv_y = self.uv[1].saturating_add(y) as f32 / 2048.0;
        let uv_h = self.uv[0].saturating_add(x).saturating_add(width) as f32 / 2048.0;
        let uv_w = self.uv[1].saturating_add(y).saturating_add(height) as f32 / 2048.0;

        let z = self.pos[2] as f32 / 100.0;

        let buffer = vec![
            SpriteVertex {
                position: [-0.5, -0.5, z],
                tex_coord: [uv_w, uv_h, self.layer as f32],
                color: self.color.as_slice(),
            },
            SpriteVertex {
                position: [0.5, -0.5, z],
                tex_coord: [uv_x, uv_h, self.layer as f32],
                color: self.color.as_slice(),
            },
            SpriteVertex {
                position: [0.5, 0.5, z],
                tex_coord: [uv_x, uv_y, self.layer as f32],
                color: self.color.as_slice(),
            },
            SpriteVertex {
                position: [-0.5, 0.5, z],
                tex_coord: [uv_w, uv_y, self.layer as f32],
                color: self.color.as_slice(),
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
