use crate::{AscendingError, AtlasGroup, Color, TextVertex};
use cosmic_text::{
    Attrs, Buffer, CacheKey, FontSystem, Metrics, SwashCache, SwashContent,
};

pub struct Text {
    pub buffer: Buffer<'static>,
    pub pos: [i32; 3],
    pub size: [u32; 2],
    pub default_color: Color,
    pub bytes: Vec<u8>,
    /// if the shader should render with the camera's view.
    pub use_camera: bool,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Text {
    pub fn create_quad(
        &mut self,
        cache: &mut SwashCache,
        text_atlas: &mut AtlasGroup<CacheKey, (i32, i32)>,
        emoji_atlas: &mut AtlasGroup<CacheKey, (i32, i32)>,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) -> Result<bool, AscendingError> {
        for run in self.buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                if text_atlas.atlas.get(&glyph.cache_key).is_some()
                    || emoji_atlas.atlas.get(&glyph.cache_key).is_some()
                {
                    continue;
                }

                let image = cache.get_image_uncached(glyph.cache_key).unwrap();
                let bitmap = image.data;
                let is_color = match image.content {
                    SwashContent::Color => true,
                    SwashContent::Mask => false,
                    SwashContent::SubpixelMask => false,
                };

                let width = image.placement.width;
                let height = image.placement.height;

                if width > 0 && height > 0 {
                    if is_color {
                        let _ = emoji_atlas
                            .atlas
                            .upload(
                                glyph.cache_key,
                                &bitmap,
                                width,
                                height,
                                device,
                                queue,
                            )
                            .ok_or(AscendingError::AtlasFull)?;
                        emoji_atlas.upload_data(
                            glyph.cache_key,
                            (image.placement.left, image.placement.top),
                        );
                    } else {
                        let _ = text_atlas
                            .atlas
                            .upload(
                                glyph.cache_key,
                                &bitmap,
                                width,
                                height,
                                device,
                                queue,
                            )
                            .ok_or(AscendingError::AtlasFull)?;
                        text_atlas.upload_data(
                            glyph.cache_key,
                            (image.placement.left, image.placement.top),
                        );
                    }
                }
            }
        }

        let mut text_buf = Vec::with_capacity(64 * 4);

        for run in self.buffer.layout_runs() {
            let line_y = run.line_y;

            for glyph in run.glyphs.iter() {
                let (allocation, is_color, position) = if let Some(allocation) =
                    text_atlas.atlas.get(&glyph.cache_key)
                {
                    let position =
                        text_atlas.get_data(&glyph.cache_key).unwrap_or((0, 0));
                    (allocation, false, position)
                } else if let Some(allocation) =
                    emoji_atlas.atlas.get(&glyph.cache_key)
                {
                    let position = emoji_atlas
                        .get_data(&glyph.cache_key)
                        .unwrap_or((0, 0));
                    (allocation, true, position)
                } else {
                    continue;
                };

                let (u, v, width, height) = allocation.rect();
                let (u, v, width, height) =
                    (u as i32, v as i32, width as f32, height as f32);

                let (x, y) = (
                    (self.pos[0] + glyph.x_int + position.0) as f32,
                    (self.pos[1] + glyph.y_int - line_y) as f32,
                );

                let (u1, v1) = (u as f32, v as f32);

                let color = if is_color {
                    Color::rgba(255, 255, 255, 255)
                } else {
                    match glyph.color_opt {
                        Some(color) => color,
                        None => self.default_color,
                    }
                };

                let default = TextVertex {
                    position: [x, y, self.pos[2] as f32],
                    hw: [width, height],
                    tex_coord: [u1, v1],
                    layer: allocation.layer as u32,
                    color: color.0,
                    use_camera: u32::from(self.use_camera),
                    is_color: is_color as u32,
                };

                text_buf.push(default);
            }
        }

        self.bytes = bytemuck::cast_slice(&text_buf).to_vec();
        Ok(true)
    }

    pub fn new(
        font_system: &'static FontSystem,
        metrics: Option<Metrics>,
        pos: [i32; 3],
        size: [u32; 2],
    ) -> Self {
        Self {
            buffer: Buffer::new(
                font_system,
                metrics.unwrap_or(Metrics::new(16, 16).scale(1)),
            ),
            pos,
            size,
            bytes: Vec::new(),
            changed: true,
            default_color: Color::rgba(0, 0, 0, 255),
            use_camera: false,
        }
    }

    /// resets the TextRender bytes to empty for new bytes
    pub fn set_text(&mut self, text: &str, attrs: Attrs<'static>) {
        self.buffer.set_text(text, attrs);
        self.changed = true;
    }

    pub fn set_buffer_size(&mut self, width: i32, height: i32) {
        self.buffer.set_size(width, height);
        self.changed = true;
    }

    /// resets the TextRender bytes to empty for new bytes
    pub fn clear(&mut self) {
        self.buffer.set_text("", cosmic_text::Attrs::new());
        self.changed = true;
    }

    /// used to check and update the vertex array.
    /// must call build_layout before you can Call this.
    pub fn update(
        &mut self,
        cache: &mut SwashCache,
        text_atlas: &mut AtlasGroup<CacheKey, (i32, i32)>,
        emoji_atlas: &mut AtlasGroup<CacheKey, (i32, i32)>,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) -> Result<bool, AscendingError> {
        if self.changed {
            if self.create_quad(
                cache,
                text_atlas,
                emoji_atlas,
                queue,
                device,
            )? {
                self.changed = false;
                return Ok(true);
            }
        }

        Ok(false)
    }
}

// This is a text layer buffer for rendering text to the screen.
// Can be used multiple times for multiple layers of text.
pub struct TextRender {
    /// Vertex array in bytes. This Holds regular glyphs
    pub text_bytes: Vec<u8>,
}

impl TextRender {
    pub fn new() -> Self {
        Self {
            //set this to be the same as the size held in the GPU buffer.
            text_bytes: Vec::with_capacity(16_384),
        }
    }

    /// resets the TextRender bytes to empty for new bytes generally at each redraw
    pub fn clear(&mut self) {
        self.text_bytes.clear()
    }

    /// Pushes to the end by cloning whats in text.
    pub fn push(&mut self, text: &Text) {
        self.text_bytes.append(&mut text.bytes.clone());
    }

    /// Appends to the end by cloning whats in text. We dont move them from one to the other
    /// Since moving would invalidate text and we need it per Render loop since its a cache.
    pub fn append(&mut self, arr: &[Text]) {
        for text in arr {
            self.text_bytes.append(&mut text.bytes.clone());
        }
    }
}
