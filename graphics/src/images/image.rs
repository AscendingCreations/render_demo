use crate::{Allocation, BufferStoreRef, Color, ImageVertex, Vec2, Vec3, Vec4};

/// rendering data for all images.
pub struct Image {
    pub pos: Vec3,
    pub hw: Vec2,
    // used for static offsets or animation Start positions
    pub uv: Vec4,
    /// Color dah  number / 255.
    pub color: Color,
    // frames, frames_per_row: this will cycle thru
    // frames per rox at the uv start.
    pub frames: Vec2,
    /// in millsecs 1000 = 1sec
    pub switch_time: u32,
    /// turn on animation if set.
    pub animate: bool,
    pub use_camera: bool,
    /// Texture area location in Atlas.
    pub texture: Option<Allocation>,
    pub store: BufferStoreRef,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Default for Image {
    fn default() -> Self {
        Self {
            pos: Vec3::default(),
            hw: Vec2::default(),
            uv: Vec4::default(),
            frames: Vec2::default(),
            switch_time: 0,
            animate: false,
            use_camera: true,
            color: Color::rgba(255, 255, 255, 255),
            texture: None,
            store: BufferStoreRef::default(),
            changed: true,
        }
    }
}

impl Image {
    pub fn create_quad(&mut self) {
        let allocation = match &self.texture {
            Some(allocation) => allocation,
            None => return,
        };

        let (u, v, width, height) = allocation.rect();
        let (u, v, width, height) = (
            self.uv.x + u as f32,
            self.uv.y + v as f32,
            self.uv.z.min(width as f32),
            self.uv.w.min(height as f32),
        );

        let instance = ImageVertex {
            position: *self.pos.as_array(),
            hw: *self.hw.as_array(),
            tex_data: [u, v, width, height],
            color: self.color.0,
            frames: *self.frames.as_array(),
            animate: u32::from(self.animate),
            use_camera: u32::from(self.use_camera),
            time: self.switch_time,
            layer: allocation.layer as i32,
        };

        self.store.borrow_mut().store = bytemuck::bytes_of(&instance).to_vec();
        self.store.borrow_mut().changed = true;
        self.changed = false;
    }

    pub fn new(texture: Allocation) -> Self {
        Self {
            texture: Some(texture),
            ..Default::default()
        }
    }

    /// used to check and update the vertex array.
    pub fn update(&mut self) -> BufferStoreRef {
        // if pos or tex_pos or color changed.
        if self.changed {
            self.create_quad();
        }

        self.store.clone()
    }
}
