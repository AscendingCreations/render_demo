use crate::{
    AscendingError, AtlasGroup, BufferStoreRef, Color, System, TextVertex,
    Vec2, Vec3, Vec4,
};
use cosmic_text::{
    Attrs, Buffer, CacheKey, FontSystem, Metrics, SwashCache, SwashContent,
};

/// Controls the visible area of the text. Any text outside of the visible area will be clipped.
/// This is given by glyphon.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextBounds(pub Vec4);

impl TextBounds {
    pub fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self(Vec4::new(left, top, right, bottom))
    }
}

impl Default for TextBounds {
    fn default() -> Self {
        Self(Vec4::new(f32::MIN, f32::MIN, f32::MAX, f32::MAX))
    }
}

pub struct Text {
    pub buffer: Buffer<'static>,
    pub pos: Vec3,
    pub size: Vec2,
    pub default_color: Color,
    pub bounds: TextBounds,
    pub store: BufferStoreRef,
    /// if the shader should render with the camera's view.
    pub use_camera: bool,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Text {
    pub fn create_quad<Controls>(
        &mut self,
        cache: &mut SwashCache,
        text_atlas: &mut AtlasGroup<CacheKey, Vec2>,
        emoji_atlas: &mut AtlasGroup<CacheKey, Vec2>,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        system: &System<Controls>,
    ) -> Result<(), AscendingError>
    where
        Controls: camera::controls::Controls,
    {
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
                                Vec2::new(
                                    image.placement.left as f32,
                                    image.placement.top as f32,
                                ),
                                device,
                                queue,
                            )
                            .ok_or(AscendingError::AtlasFull)?;
                    } else {
                        let _ = text_atlas
                            .atlas
                            .upload(
                                glyph.cache_key,
                                &bitmap,
                                width,
                                height,
                                Vec2::new(
                                    image.placement.left as f32,
                                    image.placement.top as f32,
                                ),
                                device,
                                queue,
                            )
                            .ok_or(AscendingError::AtlasFull)?;
                    }
                }
            }
        }

        let mut text_buf = Vec::with_capacity(64 * 4);

        for run in self.buffer.layout_runs() {
            let line_y = run.line_y as f32;

            for glyph in run.glyphs.iter() {
                let (allocation, is_color) = if let Some(allocation) =
                    text_atlas.atlas.get(&glyph.cache_key)
                {
                    (allocation, false)
                } else if let Some(allocation) =
                    emoji_atlas.atlas.get(&glyph.cache_key)
                {
                    (allocation, true)
                } else {
                    continue;
                };

                let position = allocation.data;
                let (u, v, width, height) = allocation.rect();
                let (mut u, mut v, mut width, mut height) =
                    (u as f32, v as f32, width as f32, height as f32);

                let (mut x, mut y) = (
                    (self.pos.x + glyph.x_int as f32 + position.x),
                    (self.pos.y + glyph.y_int as f32 - line_y),
                );

                let color = is_color
                    .then(|| Color::rgba(255, 255, 255, 255))
                    .unwrap_or(match glyph.color_opt {
                        Some(color) => color,
                        None => self.default_color,
                    });

                //Bounds used from Glyphon
                let bounds_min_x = self.bounds.0.x.max(0.0);
                let bounds_min_y = self.bounds.0.w.max(0.0);
                let bounds_max_x = self.bounds.0.z.min(system.screen_size[0]);
                let bounds_max_y = self.bounds.0.y.min(system.screen_size[1]);

                // Starts beyond right edge or ends beyond left edge
                let max_x = x + width;
                if x > bounds_max_x || max_x < bounds_min_x {
                    continue;
                }

                // Starts beyond bottom edge or ends beyond top edge
                let max_y = y + height;
                if y > bounds_max_y || max_y < bounds_min_y {
                    continue;
                }

                // Clip left edge
                if x < bounds_min_x {
                    let right_shift = bounds_min_x - x;

                    x = bounds_min_x;
                    width = max_x - bounds_min_x;
                    u += right_shift;
                }

                // Clip right edge
                if x + width > bounds_max_x {
                    width = bounds_max_x - x;
                }

                // Clip top edge
                if y < bounds_min_y {
                    height -= bounds_min_y;
                    y = bounds_min_y;
                }

                // Clip top edge
                if y + height > bounds_max_y {
                    let bottom_shift = (y + height) - bounds_max_y;

                    v += bottom_shift;
                    height -= bottom_shift;
                }

                let default = TextVertex {
                    position: [x, y, self.pos.z],
                    hw: [width, height],
                    tex_coord: [u, v],
                    layer: allocation.layer as u32,
                    color: color.0,
                    use_camera: u32::from(self.use_camera),
                    is_color: is_color as u32,
                };

                text_buf.push(default);
            }
        }

        self.store.borrow_mut().store =
            bytemuck::cast_slice(&text_buf).to_vec();
        self.store.borrow_mut().changed = true;
        self.changed = false;
        Ok(())
    }

    pub fn new(
        font_system: &'static FontSystem,
        metrics: Option<Metrics>,
        pos: Vec3,
        size: Vec2,
        bounds: Option<TextBounds>,
    ) -> Self {
        Self {
            buffer: Buffer::new(
                font_system,
                metrics.unwrap_or(Metrics::new(16, 16).scale(1)),
            ),
            pos,
            size,
            bounds: bounds.unwrap_or_default(),
            store: BufferStoreRef::default(),
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
    pub fn update<Controls>(
        &mut self,
        cache: &mut SwashCache,
        text_atlas: &mut AtlasGroup<CacheKey, Vec2>,
        emoji_atlas: &mut AtlasGroup<CacheKey, Vec2>,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        system: &System<Controls>,
    ) -> Result<BufferStoreRef, AscendingError>
    where
        Controls: camera::controls::Controls,
    {
        if self.changed {
            self.create_quad(
                cache,
                text_atlas,
                emoji_atlas,
                queue,
                device,
                system,
            )?;
        }

        Ok(self.store.clone())
    }
}

/*// This is a text layer buffer for rendering text to the screen.
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

impl Default for TextRender {
    fn default() -> Self {
        Self::new()
    }
}*/
