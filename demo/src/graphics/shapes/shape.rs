use crate::graphics::{
    Allocation, BufferLayout, BufferPass, Color, ShapeVertex,
};
use std::cmp;
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
    pub buffers: BufferPass,
    pub indices_count: usize,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
}

pub enum Shape {
    Rect {
        position: [u32; 3],
        size: [u32; 2],
        thickness: u32,
        filled: bool,
        color: Color,
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
            buffers: BufferPass::new(),
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
        let index = 0;
        /* let mut vertices = Vec::new();
        let mut indices = Vec::new();*/

        for shape in &self.shapes {
            match shape {
                Shape::Rect {
                    position,
                    size,
                    thickness,
                    filled,
                    color,
                } => {}
                Shape::Line {
                    positions,
                    thickness,
                    color,
                } => {}
                Shape::None => continue,
            }
        }

        /*  for i in 0..self.points.len() {
            vertices.push(ShapeVertex {
                position: self.points[i].into(),
                color: self.color.0,
            });
        }

        for i in (index as u32..(vertices.len() as u32 - 2)).step_by(3) {
            indices.push(i + 0);
            indices.push(i + 1);
            indices.push(i + 2);
        }

        indices.push(vertices.len() as u32 - 2);
        indices.push(vertices.len() as u32 - 1);
        indices.push(index as u32);

        self.buffers = BufferPass {
            vertices: bytemuck::cast_slice(&vertices).to_vec(),
            indices: bytemuck::cast_slice(&indices).to_vec(),
        };
        self.indices_count = indices.len();*/
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
