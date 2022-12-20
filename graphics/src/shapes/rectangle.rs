use crate::{Color, RectVertex};

/// rendering data for all sprites.
/// not to be confused with Actual NPC or Player data.
pub struct Rectangles {
    pub rects: Vec<Rect>,
    pub buffers: Vec<u8>,
    pub indices_count: usize,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
}

pub struct Rect {
    pub position: [u32; 3],
    pub size: [u32; 2],
    pub border_width: u32,
    pub color: Color,
    pub border_color: Color,
    pub radius: f32,
    pub use_camera: bool,
}

impl Default for Rectangles {
    fn default() -> Self {
        Self {
            rects: Vec::new(),
            buffers: Vec::new(),
            indices_count: 0,
            changed: true,
        }
    }
}

impl Rectangles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_rect(&mut self, shape: Rect) {
        self.rects.push(shape);
        self.changed = true;
    }

    pub fn clear_rects(&mut self) {
        self.rects.clear();
        self.changed = true;
    }

    pub fn get_mut_rects(&mut self) -> &mut Vec<Rect> {
        self.changed = true;
        &mut self.rects
    }

    pub fn fill(&mut self) {
        let mut rects = Vec::new();

        for shape in &self.rects {
            let buffer = RectVertex {
                position: [
                    shape.position[0] as f32,
                    shape.position[1] as f32,
                    shape.position[2] as f32,
                ],
                color: shape.color.0,
                size: [shape.size[0] as f32, shape.size[1] as f32],
                border_width: shape.border_width as f32,
                border_color: shape.border_color.0,
                radius: shape.radius,
                use_camera: u32::from(shape.use_camera),
            };

            rects.push(buffer);
        }

        self.buffers = bytemuck::cast_slice(&rects).to_vec();
    }

    /// used to check and update the ShapeVertex array.
    pub fn update(&mut self) -> bool {
        // if points added or any data changed recalculate paths.
        if self.changed {
            self.fill();

            self.changed = false;
            true
        } else {
            false
        }
    }
}
