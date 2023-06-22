use crate::{
    Allocation, AscendingError, AtlasGroup, DrawOrder, GpuRenderer, Index,
    MeshInstance, MeshOrderIndex, MeshVertex, OrderedIndex, OtherError,
    Texture, Vec2, Vec3, Vec4, WorldBounds,
};
use cosmic_text::Color;

pub struct Mesh {
    pub position: Vec3,
    pub size: Vec2,
    pub color: Color,
    pub image: Option<Allocation>,
    pub image_uv: Vec4,
    pub vbo_store_id: Index,
    pub ibo_store_id: Index,
    pub order: DrawOrder,
    pub bounds: Option<WorldBounds>,
    // if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Mesh {
    pub fn new(renderer: &mut GpuRenderer) -> Self {
        Self {
            position: Vec3::default(),
            size: Vec2::default(),
            color: Color::rgba(255, 255, 255, 255),
            image: None,
            image_uv: Vec4::default(),
            vbo_store_id: renderer.new_buffer(),
            ibo_store_id: renderer.new_buffer(),
            order: DrawOrder::default(),
            bounds: None,
            changed: true,
        }
    }

    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self.changed = true;
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

        self.image_uv = Vec4::new(0.0, 0.0, rect.2 as f32, rect.3 as f32);
        self.image = Some(allocation);
        self.changed = true;
        Ok(self)
    }

    //Set the Rendering Offset of the container.
    pub fn set_image_uv(&mut self, uv: Vec4) -> &mut Self {
        self.image_uv = uv;
        self.changed = true;
        self
    }

    //This sets how a object should be Clip manipulated. Width and/or Height as 0 means unlimited.
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

    pub fn create_quad(&mut self, renderer: &mut GpuRenderer) {
        let containter_tex = match self.image {
            Some(allocation) => allocation,
            None => return,
        };

        let (u, v, width, height) = containter_tex.rect();
        let _image_data = [
            self.image_uv.x + u as f32,
            self.image_uv.y + v as f32,
            self.image_uv.z.min(width as f32),
            self.image_uv.w.min(height as f32),
        ];

        //TODO Cycle through each vertex created to get correct information.
        let buffer = MeshVertex {
            position: self.position.to_array(),
            uv: [self.position.x, self.position.y],
            color: self.color.0,
        };

        if let Some(store) = renderer.get_buffer_mut(&self.vbo_store_id) {
            store.store = bytemuck::bytes_of(&buffer).to_vec();
            store.bounds = self.bounds;
            store.changed = true;
        }

        self.order = DrawOrder::new(false, &self.position, 1);
    }

    // used to check and update the ShapeVertex array.
    pub fn update(&mut self, renderer: &mut GpuRenderer) -> MeshOrderIndex {
        // if points added or any data changed recalculate paths.
        if self.changed {
            self.create_quad(renderer);
            self.changed = false;
        }

        MeshOrderIndex {
            vbo: OrderedIndex::new(self.order, self.vbo_store_id),
            ibo: OrderedIndex::new(self.order, self.ibo_store_id),
        }
    }

    /*pub fn check_mouse_bounds(&self, mouse_pos: Vec2) -> bool {
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
    }*/
}
