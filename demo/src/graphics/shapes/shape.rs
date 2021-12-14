pub(crate) use crate::graphics::{
    allocation::Allocation, BufferLayout, BufferPass, ShapeVertex,
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
pub struct Shape {
    pub points: Vec<Vec3>,
    pub closed: bool,
    pub cap_style: CapStyle,
    pub join_style: JoinStyle,
    pub fill: bool,
    pub buffers: BufferPass,
    pub thickness: f32,
    pub color: [u32; 4],
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Default for Shape {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            closed: false,
            cap_style: CapStyle::Butt,
            join_style: JoinStyle::Miter,
            fill: false,
            buffers: BufferPass::new(),
            thickness: 1.0,
            color: [0, 0, 0, 255],
            changed: true,
        }
    }
}

impl Shape {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_point(&mut self, x: f32, y: f32, z: f32) {
        self.points.push(Vec3::new(x, y, z));
        self.changed = true;
    }

    pub fn clear_points(&mut self) {
        self.points.clear();
        self.changed = true;
    }

    pub fn set_thickness(&mut self, thickness: f32) {
        self.thickness = thickness;
        self.changed = true;
    }

    pub fn set_closed(&mut self, closed: bool) {
        self.closed = closed;
        self.changed = true;
    }

    pub fn set_fill(&mut self, filled: bool) {
        self.fill = filled;
        self.changed = true;
    }

    pub fn set_color(&mut self, color: [u32; 4]) {
        self.color = color;
        self.changed = true;
    }

    pub fn set_cap_style(&mut self, cap_style: CapStyle) {
        self.cap_style = cap_style;
        self.changed = true;
    }

    pub fn set_join_style(&mut self, join_style: JoinStyle) {
        self.join_style = join_style;
        self.changed = true;
    }

    fn compute_normals(&self) -> Vec<Vec3> {
        let mut normals: Vec<Vec3> = Vec::new();

        for i0 in 0..self.points.len() {
            let i1 = (i0 + 1) % self.points.len();
            let mut normal = self.points[i1] - self.points[i0];
            normal.normalize();
            normals.push(Vec3::new(normal.y, -normal.x, normal.z));
        }

        normals
    }

    fn compute_cross(&self, normals: &[Vec3]) -> Vec<f32> {
        let mut cross: Vec<f32> = Vec::new();

        for i0 in 0..normals.len() {
            let i1 = (i0 + 1) % normals.len();
            let n0 = normals[i0];
            let n1 = normals[i1];

            cross.push(n1.x * n0.y - n0.x * n1.y);
        }

        cross
    }

    fn compute_dm(&self, normals: &[Vec3]) -> Vec<Vec3> {
        let mut result = Vec::new();

        for i0 in 0..self.points.len() {
            let i1 = (i0 + 1) % self.points.len();

            let mut dm = (normals[i0] + normals[i1]) * 0.5;
            let dmr2 = dm.mag_sq();

            if dmr2 > 0.000001 {
                let mut scale = 1.0 / dmr2;

                if scale > 600.0 {
                    scale = 600.0;
                }

                dm = dm * scale;
            }

            result.push(dm);
        }

        result
    }

    fn start_butt_cap(
        &mut self,
        buffers: &mut Vec<ShapeVertex>,
        p: Vec3,
        diff: Vec3,
        normal: Vec3,
        w: f32,
        aa: f32,
    ) {
        let p = p + diff * aa * 0.5;
        let transparent = [self.color[0], self.color[1], self.color[2], 0];

        buffers.push(ShapeVertex {
            position: (p + normal * w + diff * aa).into(),
            color: transparent,
        });
        buffers.push(ShapeVertex {
            position: (p - normal * w - diff * aa).into(),
            color: transparent,
        });
        buffers.push(ShapeVertex {
            position: (p + normal * w).into(),
            color: self.color,
        });
        buffers.push(ShapeVertex {
            position: (p - normal * w).into(),
            color: self.color,
        });
    }

    fn start_round_cap(
        &mut self,
        buffers: &mut Vec<ShapeVertex>,
        p: Vec3,
        diff: Vec3,
        normal: Vec3,
        w: f32,
    ) {
        let ncap = 16;

        for i in 0..ncap {
            let a = i as f32 / (ncap - 1) as f32 * std::f32::consts::PI;
            let ax = Vec3::new(a.cos(), a.sin(), p.z) * w;

            buffers.push(ShapeVertex {
                position: (p - normal * ax.x - diff * ax.y).into(),
                color: self.color,
            });
            buffers.push(ShapeVertex {
                position: p.into(),
                color: self.color,
            });
        }

        buffers.push(ShapeVertex {
            position: (p + normal * w).into(),
            color: self.color,
        });
        buffers.push(ShapeVertex {
            position: (p - normal * w).into(),
            color: self.color,
        });
    }

    fn start_square_cap(
        &mut self,
        buffers: &mut Vec<ShapeVertex>,
        p: Vec3,
        diff: Vec3,
        normal: Vec3,
        w: f32,
        aa: f32,
    ) {
        let p = p + diff * (w - aa);
        let transparent = [self.color[0], self.color[1], self.color[2], 0];

        buffers.push(ShapeVertex {
            position: (p + normal * w + diff * aa).into(),
            color: transparent.into(),
        });
        buffers.push(ShapeVertex {
            position: (p - normal * w - diff * aa).into(),
            color: transparent.into(),
        });
        buffers.push(ShapeVertex {
            position: (p + normal * w).into(),
            color: self.color,
        });
        buffers.push(ShapeVertex {
            position: (p - normal * w).into(),
            color: self.color,
        });
    }

    fn end_butt_cap(
        &mut self,
        buffers: &mut Vec<ShapeVertex>,
        p: Vec3,
        diff: Vec3,
        normal: Vec3,
        w: f32,
        aa: f32,
    ) {
        let p = p - diff * aa * 0.5;
        let transparent = [self.color[0], self.color[1], self.color[2], 0];

        buffers.push(ShapeVertex {
            position: (p + normal * w).into(),
            color: self.color,
        });
        buffers.push(ShapeVertex {
            position: (p - normal * w).into(),
            color: self.color,
        });
        buffers.push(ShapeVertex {
            position: (p + normal * w + diff * aa).into(),
            color: transparent.into(),
        });
        buffers.push(ShapeVertex {
            position: (p - normal * w + diff * aa).into(),
            color: transparent.into(),
        });
    }

    fn end_round_cap(
        &mut self,
        buffers: &mut Vec<ShapeVertex>,
        p: Vec3,
        diff: Vec3,
        normal: Vec3,
        w: f32,
    ) {
        let ncap = 16;

        buffers.push(ShapeVertex {
            position: (p + normal * w).into(),
            color: self.color,
        });
        buffers.push(ShapeVertex {
            position: (p - normal * w).into(),
            color: self.color,
        });

        for i in 0..ncap {
            let a = i as f32 / (ncap - 1) as f32 * std::f32::consts::PI;
            let ax = Vec3::new(a.cos(), a.sin(), p.z) * w;

            buffers.push(ShapeVertex {
                position: p.into(),
                color: self.color,
            });
            buffers.push(ShapeVertex {
                position: (p - normal * ax.x + diff * ax.y).into(),
                color: self.color,
            });
        }
    }

    fn end_square_cap(
        &mut self,
        buffers: &mut Vec<ShapeVertex>,
        p: Vec3,
        diff: Vec3,
        normal: Vec3,
        w: f32,
        aa: f32,
    ) {
        let p = p - diff * (w - aa);
        let transparent = [self.color[0], self.color[1], self.color[2], 0];

        buffers.push(ShapeVertex {
            position: (p + normal * w).into(),
            color: self.color,
        });
        buffers.push(ShapeVertex {
            position: (p - normal * w).into(),
            color: self.color,
        });
        buffers.push(ShapeVertex {
            position: (p + normal * w + diff * aa).into(),
            color: transparent.into(),
        });
        buffers.push(ShapeVertex {
            position: (p - normal * w + diff * aa).into(),
            color: transparent.into(),
        });
    }

    fn join_bevel(
        &self,
        buffers: &mut Vec<ShapeVertex>,
        p1: Vec3,
        n0: Vec3,
        n1: Vec3,
        dm: Vec3,
        cross: f32,
        lw: f32,
        rw: f32,
    ) {
        if cross > 0.0 {
            let l0 = p1 + dm * lw;
            let l1 = p1 + dm * lw;

            buffers.push(ShapeVertex {
                position: l0.into(),
                color: self.color,
            });
            buffers.push(ShapeVertex {
                position: (p1 - n0 * rw).into(),
                color: self.color,
            });

            buffers.push(ShapeVertex {
                position: l1.into(),
                color: self.color,
            });
            buffers.push(ShapeVertex {
                position: (p1 - n1 * rw).into(),
                color: self.color,
            });
        } else {
            let l0 = p1 - dm * lw;
            let l1 = p1 - dm * lw;

            buffers.push(ShapeVertex {
                position: (p1 + n0 * rw).into(),
                color: self.color,
            });
            buffers.push(ShapeVertex {
                position: l0.into(),
                color: self.color,
            });

            buffers.push(ShapeVertex {
                position: (p1 + n1 * rw).into(),
                color: self.color,
            });
            buffers.push(ShapeVertex {
                position: l1.into(),
                color: self.color,
            });
        }
    }

    fn join_round(
        &self,
        buffers: &mut Vec<ShapeVertex>,
        p1: Vec3,
        n0: Vec3,
        n1: Vec3,
        dm: Vec3,
        cross: f32,
        lw: f32,
        rw: f32,
    ) {
        let ncap = 16;

        if cross > 0.0 {
            let l0 = p1 + dm * lw;
            let l1 = p1 + dm * lw;

            buffers.push(ShapeVertex {
                position: l0.into(),
                color: self.color,
            });
            buffers.push(ShapeVertex {
                position: (p1 - n0 * rw).into(),
                color: self.color,
            });

            let a0 = (-n0.y).atan2(-n0.x);
            let mut a1 = (-n1.y).atan2(-n1.x);

            if a1 > a0 {
                a1 -= std::f32::consts::PI * 2.0;
            }

            for i in 0..ncap {
                let a = i as f32 / (ncap - 1) as f32 * (a1 - a0) + a0;
                let ax = Vec3::new(a.cos(), a.sin(), 0.0) * rw;

                buffers.push(ShapeVertex {
                    position: p1.into(),
                    color: self.color,
                });
                buffers.push(ShapeVertex {
                    position: (p1 + ax).into(),
                    color: self.color,
                });
            }

            buffers.push(ShapeVertex {
                position: l1.into(),
                color: self.color,
            });
            buffers.push(ShapeVertex {
                position: (p1 - n1 * rw).into(),
                color: self.color,
            });
        } else {
            let r0 = p1 - dm * lw;
            let r1 = p1 - dm * lw;

            buffers.push(ShapeVertex {
                position: (p1 + n0 * rw).into(),
                color: self.color,
            });
            buffers.push(ShapeVertex {
                position: r0.into(),
                color: self.color,
            });

            let a0 = n0.y.atan2(n0.x);
            let mut a1 = n1.y.atan2(n1.x);

            if a1 < a0 {
                a1 += std::f32::consts::PI * 2.0;
            }

            for i in 0..ncap {
                let a = i as f32 / (ncap - 1) as f32 * (a1 - a0) + a0;
                let ax = Vec3::new(a.cos(), a.sin(), 0.0) * rw;

                buffers.push(ShapeVertex {
                    position: (p1 + ax).into(),
                    color: self.color,
                });
                buffers.push(ShapeVertex {
                    position: p1.into(),
                    color: self.color,
                });
            }

            buffers.push(ShapeVertex {
                position: (p1 + n1 * rw).into(),
                color: self.color,
            });
            buffers.push(ShapeVertex {
                position: r1.into(),
                color: self.color,
            });
        }
    }

    fn join_miter(
        &self,
        buffers: &mut Vec<ShapeVertex>,
        p: Vec3,
        dm: Vec3,
        lw: f32,
        rw: f32,
    ) {
        buffers.push(ShapeVertex {
            position: (p + dm * lw).into(),
            color: self.color,
        });
        buffers.push(ShapeVertex {
            position: (p - dm * rw).into(),
            color: self.color,
        });
    }

    pub fn stroke(&mut self) {
        let normals = self.compute_normals();
        let cross = self.compute_cross(&normals);
        let dm = self.compute_dm(&normals);
        let index = 0;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let (mut i0, mut i1, start, end) = if self.closed {
            (self.points.len() - 1, 0, 0, self.points.len())
        } else {
            (0, 1, 1, self.points.len() - 1)
        };

        let mut p0 = self.points[i0];
        let mut p1 = self.points[i1];

        if !self.closed {
            let mut diff = p1 - p0;
            diff.normalize();

            let aa = 1.0;

            match self.cap_style {
                CapStyle::Butt => self.start_butt_cap(
                    &mut vertices,
                    p0,
                    diff,
                    normals[i0],
                    self.thickness,
                    aa,
                ),
                CapStyle::Round => self.start_round_cap(
                    &mut vertices,
                    p0,
                    diff,
                    normals[i0],
                    self.thickness,
                ),
                CapStyle::Square => self.start_square_cap(
                    &mut vertices,
                    p0,
                    diff,
                    normals[i0],
                    self.thickness,
                    aa,
                ),
            }
        }

        for _ in start..end {
            match self.join_style {
                JoinStyle::Bevel => self.join_bevel(
                    &mut vertices,
                    p1,
                    normals[i0],
                    normals[i1],
                    dm[i0],
                    cross[i0],
                    self.thickness,
                    self.thickness,
                ),
                JoinStyle::Round => self.join_round(
                    &mut vertices,
                    p1,
                    normals[i0],
                    normals[i1],
                    dm[i0],
                    cross[i0],
                    self.thickness,
                    self.thickness,
                ),
                JoinStyle::Miter => self.join_miter(
                    &mut vertices,
                    p1,
                    dm[i0],
                    self.thickness,
                    self.thickness,
                ),
            }

            i0 = i1;
            p0 = p1;
            i1 = (i1 + 1) % self.points.len();
            p1 = self.points[i1];
        }

        if !self.closed {
            let mut diff = p1 - p0;
            diff.normalize();

            let aa = 1.0;

            match self.cap_style {
                CapStyle::Butt => self.end_butt_cap(
                    &mut vertices,
                    p1,
                    diff,
                    normals[i0],
                    self.thickness,
                    aa,
                ),
                CapStyle::Round => self.end_round_cap(
                    &mut vertices,
                    p1,
                    diff,
                    normals[i0],
                    self.thickness,
                ),
                CapStyle::Square => self.end_square_cap(
                    &mut vertices,
                    p1,
                    diff,
                    normals[i0],
                    self.thickness,
                    aa,
                ),
            }
        }

        /* Build the index buffer. */
        for i in (index as u32..(vertices.len() as u32 - 2)).step_by(2) {
            indices.push(i + 0);
            indices.push(i + 1);
            indices.push(i + 2);

            indices.push(i + 2);
            indices.push(i + 3);
            indices.push(i + 1);
        }

        /* Loop the end to the beginning if the path is closed. */
        if self.closed {
            indices.push(vertices.len() as u32 - 2);
            indices.push(vertices.len() as u32 - 1);
            indices.push(index as u32 + 0);

            indices.push(vertices.len() as u32 - 1);
            indices.push(index as u32 + 0);
            indices.push(index as u32 + 1);
        }

        self.buffers = BufferPass {
            vertices: bytemuck::cast_slice(&vertices).to_vec(),
            indices: bytemuck::cast_slice(&indices).to_vec(),
        }
    }

    pub fn fill(&mut self) {
        let normals = self.compute_normals();
        let cross = self.compute_cross(&normals);
        let dm = self.compute_dm(&normals);
        let woff = 0.0;
        let index = 0;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        if self.thickness > 0.0 {
            let mut i0 = self.points.len() - 1;
            let mut i1 = 0;

            for _ in 0..self.points.len() {
                let p1 = self.points[i1];

                match self.join_style {
                    JoinStyle::Bevel | JoinStyle::Round => {
                        if cross[i1] > 0.0 {
                            vertices.push(ShapeVertex {
                                position: (p1 + dm[i1] * woff).into(),
                                color: self.color,
                            });
                        } else {
                            vertices.push(ShapeVertex {
                                position: (p1 + normals[i0] * woff).into(),
                                color: self.color,
                            });
                            vertices.push(ShapeVertex {
                                position: (p1 + normals[i1] * woff).into(),
                                color: self.color,
                            });
                        }
                    }
                    _ => {
                        vertices.push(ShapeVertex {
                            position: (p1 + dm[i1] * woff).into(),
                            color: self.color,
                        });
                    }
                }

                i0 = i1;
                i1 = i1 + 1;
            }
        } else {
            for i in 0..self.points.len() {
                vertices.push(ShapeVertex {
                    position: self.points[i].into(),
                    color: self.color,
                });
            }
        }

        for i in (index as u32..(vertices.len() as u32 - 2)).step_by(3) {
            indices.push(i + 0);
            indices.push(i + 1);
            indices.push(i + 2);
        }

        indices.push(vertices.len() as u32 - 2);
        indices.push(vertices.len() as u32 - 1);
        indices.push(index as u32);

        if self.thickness > 0.0 {
            let index = vertices.len();
            let mut i0 = self.points.len() - 1;
            let mut i1 = 0;

            for _ in 0..self.points.len() {
                let p1 = self.points[i1];

                match self.join_style {
                    JoinStyle::Bevel => self.join_bevel(
                        &mut vertices,
                        p1,
                        normals[i0],
                        normals[i1],
                        dm[i0],
                        cross[i0],
                        0.0,
                        self.thickness,
                    ),
                    JoinStyle::Round => self.join_round(
                        &mut vertices,
                        p1,
                        normals[i0],
                        normals[i1],
                        dm[i0],
                        cross[i0],
                        0.0,
                        self.thickness,
                    ),
                    JoinStyle::Miter => self.join_miter(
                        &mut vertices,
                        p1,
                        dm[i0],
                        0.0,
                        self.thickness,
                    ),
                }

                i0 = i1;
                i1 = (i1 + 1) % self.points.len();
            }

            for i in (index as u32..(vertices.len() as u32 - 2)).step_by(2) {
                indices.push(i + 0);
                indices.push(i + 1);
                indices.push(i + 2);

                indices.push(i + 2);
                indices.push(i + 3);
                indices.push(i + 1);
            }

            indices.push(vertices.len() as u32 - 2);
            indices.push(vertices.len() as u32 - 1);
            indices.push(index as u32);

            indices.push(vertices.len() as u32 - 1);
            indices.push(index as u32 + 0);
            indices.push(index as u32 + 1);
        }

        self.buffers = BufferPass {
            vertices: bytemuck::cast_slice(&vertices).to_vec(),
            indices: bytemuck::cast_slice(&indices).to_vec(),
        }
    }

    /// used to check and update the ShapeVertex array.
    pub fn update(&mut self) {
        // if points added or any data changed recalculate paths.
        if self.changed {
            if self.fill {
                self.fill();
            } else {
                self.stroke();
            }

            self.changed = false;
        }
    }
}
