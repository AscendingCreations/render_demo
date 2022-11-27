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
        let index = 0;
        let mut vertices = Vec::new();
        /*let mut indices = Vec::new();*/

        for shape in &self.shapes {
            let buffer = match shape {
                Shape::Rect {
                    position,
                    size,
                    color,
                    border_width,
                    border_color,
                    radius,
                } => {
                    let mut b_width = *border_width as f32;

                    if b_width > 1.0 {
                        b_width = 1.0;
                    }

                    iter::repeat(ShapeVertex {
                        position: [
                            position[0] as f32,
                            position[1] as f32,
                            position[2] as f32,
                        ],
                        color: color.0,
                        size: [size[0] as f32, size[1] as f32],
                        border_width: b_width,
                        border_color: border_color.0,
                        radius,
                    })
                    .take(4)
                    .collect::<Vec<ShapeVertex>>()
                }
                Shape::Line {
                    positions,
                    thickness,
                    color,
                } => continue,
                Shape::None => continue,
            };

            vertices.append(&mut bytemuck::cast_slice(&buffer).to_vec());
        }

        self.buffers = vertices;

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
