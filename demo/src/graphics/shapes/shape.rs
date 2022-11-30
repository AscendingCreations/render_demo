use crate::graphics::{
    Allocation, BufferLayout, BufferPass, Color, ShapeVertex,
};
use std::cmp;
use std::iter;
use ultraviolet::vec::Vec3;

#[derive(Copy, Clone, Debug)]
pub enum CapStyle {
    Butt,
    Round,
    Square,
}

#[derive(Copy, Clone, Debug)]
pub enum JoinStyle {
    Bevel,
    Miter,
    Round,
}

/// rendering data for all sprites.
/// not to be confused with Actual NPC or Player data.
pub struct Shapes {
    pub shapes: Vec<Shape>,
    pub buffers: Vec<u8>,
    pub indices_count: usize,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
}

pub enum Shape {
    Rect {
        position: [u32; 3],
        size: [u32; 2],
        border_width: u32,
        color: Color,
        border_color: Color,
        radius: f32,
    },
    Line {
        positions: [u32; 6],
        thickness: u32,
        color: Color,
    },
    None,
}

impl Default for Shapes {
    fn default() -> Self {
        Self {
            shapes: Vec::new(),
            buffers: Vec::new(),
            indices_count: 0,
            changed: true,
        }
    }
}

impl Shapes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_shape(&mut self, shape: Shape) {
        self.shapes.push(shape);
        self.changed = true;
    }

    pub fn clear_shapes(&mut self) {
        self.shapes.clear();
        self.changed = true;
    }

    pub fn get_mut_shapes(&mut self) -> &mut Vec<Shape> {
        self.changed = true;
        &mut self.shapes
    }

    pub fn fill(&mut self) {
        let mut shapes = Vec::new();

        for shape in &self.shapes {
            let buffer = match shape {
                Shape::Rect {
                    position,
                    size,
                    color,
                    border_width,
                    border_color,
                    radius,
                } => ShapeVertex {
                    position: [
                        position[0] as f32,
                        position[1] as f32,
                        position[2] as f32,
                    ],
                    color: color.0,
                    size: [size[0] as f32, size[1] as f32],
                    border_width: *border_width as f32,
                    border_color: border_color.0,
                    radius: *radius,
                },
                Shape::Line {
                    positions: _,
                    thickness: _,
                    color: _,
                } => continue,
                Shape::None => continue,
            };

            shapes.push(buffer);
        }

        self.buffers = bytemuck::cast_slice(&shapes).to_vec();
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
