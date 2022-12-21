use crate::{Allocation, AtlasGroup, RectVertex};
use cosmic_text::Color;
use image::{self, ImageBuffer};
use std::cmp;

/// rendering data for all sprites.
/// not to be confused with Actual NPC or Player data.
pub struct Rectangles {
    pub rects: Vec<Rect>,
    pub buffers: Vec<u8>,
    pub indices_count: usize,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
}

#[derive(Default)]
pub struct Rect {
    pub position: [u32; 3],
    pub size: [u32; 2],
    pub border_width: u32,
    pub container: Option<Allocation>,
    pub container_uv: [u16; 4],
    pub border: Option<Allocation>,
    pub border_uv: [u16; 4],
    pub radius: f32,
}

impl Rect {
    fn add_color(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &mut AtlasGroup,
        color: Color,
    ) -> Option<Allocation> {
        let col =
            format!("{},{},{},{}", color.r(), color.g(), color.b(), color.a());
        let mut image: ImageBuffer<image::Rgba<u8>, Vec<u8>> =
            ImageBuffer::new(1, 1);
        let pixel = image.get_pixel_mut(0, 0);
        *pixel = image::Rgba([color.r(), color.g(), color.b(), color.a()]);
        atlas.upload(col, image.as_raw(), 1, 1, device, queue)
    }

    pub fn set_color(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &mut AtlasGroup,
        color: Color,
    ) -> &mut Self {
        self.container = Self::add_color(device, queue, atlas, color);
        self
    }

    pub fn set_border_color(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &mut AtlasGroup,
        color: Color,
    ) -> &mut Self {
        self.border = Self::add_color(device, queue, atlas, color);
        self
    }

    pub fn set_container_uv(
        &mut self,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
    ) -> &mut Self {
        self.container_uv = [x, y, w, h];
        self
    }

    pub fn set_border_uv(
        &mut self,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
    ) -> &mut Self {
        self.border_uv = [x, y, w, h];
        self
    }
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
            let containter_tex = match shape.container {
                Some(allocation) => allocation,
                None => continue,
            };

            let border_tex = match shape.border {
                Some(allocation) => allocation,
                None => continue,
            };

            let (u, v, width, height) = containter_tex.rect();
            let container_data = [
                shape.container_uv[0].saturating_add(u as u16),
                shape.container_uv[1].saturating_add(v as u16),
                cmp::min(shape.container_uv[2], width as u16),
                cmp::min(shape.container_uv[3], height as u16),
            ];

            let (u, v, width, height) = border_tex.rect();
            let border_data = [
                shape.border_uv[0].saturating_add(u as u16),
                shape.border_uv[1].saturating_add(v as u16),
                cmp::min(shape.border_uv[2], width as u16),
                cmp::min(shape.border_uv[3], height as u16),
            ];

            let buffer = RectVertex {
                position: [
                    shape.position[0] as f32,
                    shape.position[1] as f32,
                    shape.position[2] as f32,
                ],
                size: [shape.size[0] as f32, shape.size[1] as f32],
                border_width: shape.border_width as f32,
                radius: shape.radius,
                container_data,
                border_data,
                layer: containter_tex.layer as u32,
                border_layer: border_tex.layer as u32,
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
