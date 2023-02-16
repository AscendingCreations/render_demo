use crate::{
    Allocation, AscendingError, AtlasGroup, Bounds, OtherError, RectVertex,
    Texture, Vec2, Vec3, Vec4,
};
use cosmic_text::Color;
use image::{self, ImageBuffer};

pub struct Rect {
    pub position: Vec3,
    pub size: Vec2,
    pub border_width: u32,
    pub container: Option<Allocation>,
    pub container_uv: Vec4,
    pub border: Option<Allocation>,
    pub border_uv: Vec4,
    pub radius: Option<f32>,
    pub bytes: Vec<u8>,
    pub bounds: Bounds,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            position: Vec3::default(),
            size: Vec2::default(),
            border_width: 0,
            container: None,
            container_uv: Vec4::default(),
            border: None,
            border_uv: Vec4::default(),
            radius: None,
            bytes: Vec::new(),
            bounds: Bounds::default(),
            changed: true,
        }
    }
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
        atlas.upload(col, image.as_raw(), 1, 1, 0, device, queue)
    }

    pub fn set_color(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &mut AtlasGroup,
        color: Color,
    ) -> &mut Self {
        self.container = Self::add_color(device, queue, atlas, color);
        self.changed = true;
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
        self.changed = true;
        self
    }

    pub fn set_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &mut AtlasGroup,
        path: String,
    ) -> Result<&mut Self, AscendingError> {
        let allocation = Texture::from_file(path)?
            .group_upload(atlas, device, queue)
            .ok_or_else(|| OtherError::new("failed to upload image"))?;
        self.container = Some(allocation);
        self.changed = true;
        Ok(self)
    }

    pub fn set_border_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &mut AtlasGroup,
        path: String,
    ) -> Result<&mut Self, AscendingError> {
        let allocation = Texture::from_file(path)?
            .group_upload(atlas, device, queue)
            .ok_or_else(|| OtherError::new("failed to upload image"))?;
        self.border = Some(allocation);
        self.changed = true;
        Ok(self)
    }

    //Set the Rendering Offset of the container.
    pub fn set_container_uv(&mut self, uv: Vec4) -> &mut Self {
        self.container_uv = uv;
        self.changed = true;
        self
    }

    //Set the Rendering Offset of the border.
    pub fn set_border_uv(&mut self, uv: Vec4) -> &mut Self {
        self.border_uv = uv;
        self.changed = true;
        self
    }

    ///This sets how a object should be Clip manipulated. Width and/or Height as 0 means unlimited.
    pub fn set_bounds(&mut self, bounds: Bounds) -> &mut Self {
        self.bounds = bounds;
        self.changed = true;
        self
    }

    pub fn create_quad(&mut self) {
        let containter_tex = match self.container {
            Some(allocation) => allocation,
            None => return,
        };

        let border_tex = match self.border {
            Some(allocation) => allocation,
            None => return,
        };

        let (u, v, width, height) = containter_tex.rect();
        let container_data = [
            self.container_uv.x + u as f32,
            self.container_uv.y + v as f32,
            self.container_uv.z.min(width as f32),
            self.container_uv.w.min(height as f32),
        ];

        let (u, v, width, height) = border_tex.rect();
        let border_data = [
            self.border_uv.x + u as f32,
            self.border_uv.y + v as f32,
            self.border_uv.z.min(width as f32),
            self.border_uv.w.min(height as f32),
        ];

        let buffer = RectVertex {
            position: *self.position.as_array(),
            size: *self.size.as_array(),
            border_width: self.border_width as f32,
            radius: self.radius.unwrap_or_default(),
            container_data,
            border_data,
            layer: containter_tex.layer as u32,
            border_layer: border_tex.layer as u32,
        };

        self.bytes = bytemuck::bytes_of(&buffer).to_vec();
    }

    /// used to check and update the ShapeVertex array.
    pub fn update(&mut self) -> bool {
        // if points added or any data changed recalculate paths.
        if self.changed {
            self.create_quad();

            self.changed = false;
            true
        } else {
            false
        }
    }

    pub fn check_mouse_bounds(&self, mouse_pos: [i32; 2]) -> bool {
        if let Some(radius) = self.radius {
            let pos = [self.position.x, self.position.y];

            let inner_size =
                [self.size.x - radius * 2.0, self.size.y - radius * 2.0];
            let top_left = [pos[0] + radius, pos[1] + radius];
            let bottom_right =
                [top_left[0] + inner_size[0], top_left[1] + inner_size[1]];

            let top_left_distance = [
                top_left[0] - mouse_pos[0] as f32,
                top_left[1] - mouse_pos[1] as f32,
            ];
            let bottom_right_distance = [
                mouse_pos[0] as f32 - bottom_right[0],
                mouse_pos[1] as f32 - bottom_right[1],
            ];

            let dist = [
                top_left_distance[0].max(bottom_right_distance[0]).max(0.0),
                top_left_distance[1].max(bottom_right_distance[1]).max(0.0),
            ];

            let dist = (dist[0] * dist[0] + dist[1] * dist[1]).sqrt();

            dist < radius
        } else {
            mouse_pos[0] > self.position[0] as i32
                && mouse_pos[0] < self.position[0] as i32 + self.size[0] as i32
                && mouse_pos[1] > self.position[1] as i32
                && mouse_pos[1] < self.position[1] as i32 + self.size[1] as i32
        }
    }
}
