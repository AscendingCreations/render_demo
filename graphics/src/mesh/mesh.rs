use crate::{
    Allocation, AscendingError, AtlasGroup, DrawOrder, GpuRenderer, Index,
    OrderedIndex, OtherError, RectVertex, Texture, Vec2, Vec3, Vec4,
    WorldBounds,
};
use cosmic_text::Color;
use image::{self, ImageBuffer};

pub struct Rect {
    pub position: Vec3,
    pub size: Vec2,
    pub border_width: f32,
    pub container: Option<Allocation>,
    pub container_uv: Vec4,
    pub border: Option<Allocation>,
    pub border_uv: Vec4,
    pub radius: Option<f32>,
    pub store_id: Index,
    pub order: DrawOrder,
    pub bounds: Option<WorldBounds>,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Rect {
    pub fn new(renderer: &mut GpuRenderer) -> Self {
        Self {
            position: Vec3::default(),
            size: Vec2::default(),
            border_width: 0.0,
            container: None,
            container_uv: Vec4::default(),
            border: None,
            border_uv: Vec4::default(),
            radius: None,
            store_id: renderer.new_buffer(),
            order: DrawOrder::default(),
            bounds: None,
            changed: true,
        }
    }

    fn add_color(
        renderer: &GpuRenderer,
        atlas: &mut AtlasGroup,
        color: Color,
    ) -> Option<Allocation> {
        let col =
            format!("{},{},{},{}", color.r(), color.g(), color.b(), color.a());
        let mut image: ImageBuffer<image::Rgba<u8>, Vec<u8>> =
            ImageBuffer::new(1, 1);
        let pixel = image.get_pixel_mut(0, 0);
        *pixel = image::Rgba([color.r(), color.g(), color.b(), color.a()]);
        atlas.upload(col, image.as_raw(), 1, 1, 0, renderer)
    }

    pub fn set_color(
        &mut self,
        renderer: &GpuRenderer,
        atlas: &mut AtlasGroup,
        color: Color,
    ) -> &mut Self {
        if let Some(allocation) = Self::add_color(renderer, atlas, color) {
            let rect = allocation.rect();

            self.container_uv =
                Vec4::new(0.0, 0.0, rect.2 as f32, rect.3 as f32);
            self.container = Some(allocation);
            self.changed = true;
        }

        self
    }

    pub fn set_border_color(
        &mut self,
        renderer: &GpuRenderer,
        atlas: &mut AtlasGroup,
        color: Color,
    ) -> &mut Self {
        if let Some(allocation) = Self::add_color(renderer, atlas, color) {
            let rect = allocation.rect();

            self.border_uv = Vec4::new(0.0, 0.0, rect.2 as f32, rect.3 as f32);
            self.border = Some(allocation);
            self.changed = true;
        }

        self
    }

    pub fn set_texture(
        &mut self,
        renderer: &GpuRenderer,
        atlas: &mut AtlasGroup,
        path: String,
    ) -> Result<&mut Self, AscendingError> {
        let allocation = Texture::from_file(path)?
            .group_upload(atlas, renderer)
            .ok_or_else(|| OtherError::new("failed to upload image"))?;

        let rect = allocation.rect();

        self.container_uv = Vec4::new(0.0, 0.0, rect.2 as f32, rect.3 as f32);
        self.container = Some(allocation);
        self.changed = true;
        Ok(self)
    }

    pub fn set_border_texture(
        &mut self,
        renderer: &GpuRenderer,
        atlas: &mut AtlasGroup,
        path: String,
    ) -> Result<&mut Self, AscendingError> {
        let allocation = Texture::from_file(path)?
            .group_upload(atlas, renderer)
            .ok_or_else(|| OtherError::new("failed to upload image"))?;
        let rect = allocation.rect();

        self.border_uv = Vec4::new(
            rect.0 as f32,
            rect.1 as f32,
            rect.2 as f32,
            rect.3 as f32,
        );
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
    pub fn set_bounds(&mut self, bounds: Option<WorldBounds>) -> &mut Self {
        self.bounds = bounds;
        self.changed = true;
        self
    }

    pub fn set_position(&mut self, position: Vec3) -> &mut Self {
        self.position = position;
        self.changed = true;
        self
    }

    pub fn set_size(&mut self, size: Vec2) -> &mut Self {
        self.size = size;
        self.changed = true;
        self
    }

    pub fn set_border_width(&mut self, size: f32) -> &mut Self {
        self.border_width = size;
        self.changed = true;
        self
    }

    pub fn set_radius(&mut self, radius: Option<f32>) -> &mut Self {
        self.radius = radius;
        self.changed = true;
        self
    }

    pub fn create_quad(&mut self, renderer: &mut GpuRenderer) {
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
            position: self.position.to_array(),
            size: self.size.to_array(),
            border_width: self.border_width,
            radius: self.radius.unwrap_or_default(),
            container_data,
            border_data,
            layer: containter_tex.layer as u32,
            border_layer: border_tex.layer as u32,
        };

        if let Some(store) = renderer.get_buffer_mut(&self.store_id) {
            store.store = bytemuck::bytes_of(&buffer).to_vec();
            store.bounds = self.bounds;
            store.changed = true;
        }

        self.order = DrawOrder::new(false, &self.position, 1);
    }

    /// used to check and update the ShapeVertex array.
    pub fn update(&mut self, renderer: &mut GpuRenderer) -> OrderedIndex {
        // if points added or any data changed recalculate paths.
        if self.changed {
            self.create_quad(renderer);
            self.changed = false;
        }

        OrderedIndex::new(self.order, self.store_id)
    }

    pub fn check_mouse_bounds(&self, mouse_pos: Vec2) -> bool {
        if let Some(radius) = self.radius {
            let pos = [self.position.x, self.position.y];

            let inner_size =
                [self.size.x - radius * 2.0, self.size.y - radius * 2.0];
            let top_left = [pos[0] + radius, pos[1] + radius];
            let bottom_right =
                [top_left[0] + inner_size[0], top_left[1] + inner_size[1]];

            let top_left_distance =
                [top_left[0] - mouse_pos.x, top_left[1] - mouse_pos.y];
            let bottom_right_distance =
                [mouse_pos.x - bottom_right[0], mouse_pos.y - bottom_right[1]];

            let dist = [
                top_left_distance[0].max(bottom_right_distance[0]).max(0.0),
                top_left_distance[1].max(bottom_right_distance[1]).max(0.0),
            ];

            let dist = (dist[0] * dist[0] + dist[1] * dist[1]).sqrt();

            dist < radius
        } else {
            mouse_pos[0] > self.position.x
                && mouse_pos[0] < self.position.x + self.size.x
                && mouse_pos[1] > self.position.y
                && mouse_pos[1] < self.position.y + self.size.y
        }
    }
}
