use crate::graphics::{allocation::Allocation, Rgba, Vertex};

pub struct Sprite {
    pos: [u32; 3],
    hw: [u32; 2],
    uv: [u32; 4],
    layer: u32,
    color: Rgba,
    //Texture area location in Atlas.
    texture: Option<Allocation>,
    buffer: Vec<Vertex>,
    //if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            pos: [0; 3],
            hw: [0; 2],
            uv: [0; 4],
            layer: 1,
            color: Rgba::new(1.0, 1.0, 1.0, 1.0),
            texture: None,
            buffer: Vec::new(),
            changed: true,
        }
    }
}

impl Sprite {
    fn new(texture: Allocation) -> Self {
        Self {
            pos: [0; 3],
            hw: [0; 2],
            uv: [0; 4],
            layer: texture.layer as u32,
            color: Rgba::new(1.0, 1.0, 1.0, 1.0),
            texture: Some(texture),
            buffer: Vec::new(),
            changed: true,
        }
    }

    pub fn create_quad(&mut self) {
        let min_x = (self.pos[0] - self.hw[0]) as f32 * 0.5;
        let min_y = (self.pos[1] - self.hw[1]) as f32 * 0.5;
        let max_x = (self.pos[0] + self.hw[0]) as f32 * 0.5;
        let max_y = (self.pos[1] + self.hw[1]) as f32 * 0.5;

        let (width, height) = if let Some(allocation) = self.texture {
            let (h, w) = allocation.size();
            (h as f32, w as f32)
        } else {
            (1.0, 1.0)
        };

        let uv_x = self.uv[0] as f32 / width;
        let uv_y = self.uv[1] as f32 / height;
        let uv_h = (self.uv[0] + self.uv[2]) as f32 / width;
        let uv_w = (self.uv[1] + self.uv[3]) as f32 / height;

        self.buffer = vec![
            Vertex {
                position: [min_x, min_y, self.pos[3] as f32],
                tex_coord: [uv_x, uv_y, self.layer as f32],
                color: self.color.as_slice(),
            },
            Vertex {
                position: [max_x, min_y, self.pos[3] as f32],
                tex_coord: [uv_w, uv_y, self.layer as f32],
                color: self.color.as_slice(),
            },
            Vertex {
                position: [max_x, max_y, self.pos[3] as f32],
                tex_coord: [uv_w, uv_h, self.layer as f32],
                color: self.color.as_slice(),
            },
            Vertex {
                position: [min_x, max_y, self.pos[3] as f32],
                tex_coord: [uv_x, uv_h, self.layer as f32],
                color: self.color.as_slice(),
            },
        ];
    }

    pub fn update(&mut self) {
        //if pos or tex_pos or color changed.
        if self.changed {
            self.create_quad();
        }
    }
}
