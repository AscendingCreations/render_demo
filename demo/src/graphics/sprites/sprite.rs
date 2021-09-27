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
        let mut bytes = vec![];
        let min_x = self.pos[0].saturating_sub(self.hw[0]) as f32;
        let min_y = self.pos[1].saturating_sub(self.hw[1]) as f32;
        let max_x = self.pos[0].saturating_add(self.hw[0]) as f32;
        let max_y = self.pos[1].saturating_add(self.hw[1]) as f32;

        let (width, height) = if let Some(allocation) = &self.texture {
            let (h, w) = allocation.size();
            (h as f32, w as f32)
        } else {
            (1.0, 1.0)
        };

        let uv_x = self.uv[0] as f32 / width;
        let uv_y = self.uv[1] as f32 / height;
        let uv_h = self.uv[0].saturating_add(self.uv[2]) as f32 / width;
        let uv_w = self.uv[1].saturating_add(self.uv[3]) as f32 / height;

        let buffer = vec![
            SpriteVertex {
                position: [min_x, min_y, self.pos[2] as f32],
                tex_coord: [uv_x, uv_y, self.layer as f32],
                color: self.color.as_slice(),
            },
            SpriteVertex {
                position: [max_x, min_y, self.pos[2] as f32],
                tex_coord: [uv_w, uv_y, self.layer as f32],
                color: self.color.as_slice(),
            },
            SpriteVertex {
                position: [max_x, max_y, self.pos[2] as f32],
                tex_coord: [uv_w, uv_h, self.layer as f32],
                color: self.color.as_slice(),
            },
            SpriteVertex {
                position: [min_x, max_y, self.pos[2] as f32],
                tex_coord: [uv_x, uv_h, self.layer as f32],
                color: self.color.as_slice(),
            },
        ];

        for vertex in buffer.iter() {
            vertex.to_bytes(&mut bytes);
        }

        self.bytes = bytes;
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
