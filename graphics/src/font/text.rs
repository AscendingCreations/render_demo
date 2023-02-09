use crate::{AscendingError, AtlasGroup, Color, System, TextVertex};
use cosmic_text::{
    Attrs, Buffer, CacheKey, FontSystem, Metrics, SwashCache, SwashContent,
};

/// Controls the visible area of the text. Any text outside of the visible area will be clipped.
/// This is given by glyphon.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextBounds {
    /// The position of the left edge of the visible area.
    pub left: i32,
    /// The position of the top edge of the visible area.
    pub top: i32,
    /// The position of the right edge of the visible area.
    pub right: i32,
    /// The position of the bottom edge of the visible area.
    pub bottom: i32,
}

impl TextBounds {
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}

impl Default for TextBounds {
    fn default() -> Self {
        Self {
            left: i32::MIN,
            top: i32::MIN,
            right: i32::MAX,
            bottom: i32::MAX,
        }
    }
}

pub struct Text {
    pub buffer: Buffer<'static>,
    pub pos: [i32; 3],
    pub size: [u32; 2],
    pub default_color: Color,
    pub bounds: TextBounds,
    pub bytes: Vec<u8>,
    /// if the shader should render with the camera's view.
    pub use_camera: bool,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Text {
    pub fn create_quad<Controls>(
        &mut self,
        cache: &mut SwashCache,
        text_atlas: &mut AtlasGroup<CacheKey, (i32, i32)>,
        emoji_atlas: &mut AtlasGroup<CacheKey, (i32, i32)>,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        system: &System<Controls>,
    ) -> Result<bool, AscendingError>
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
                                (image.placement.left, image.placement.top),
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
                                (image.placement.left, image.placement.top),
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
            let line_y = run.line_y;

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
                    (u as i32, v as i32, width as i32, height as i32);

                let (mut x, mut y) = (
                    (self.pos[0] + glyph.x_int + position.0),
                    (self.pos[1] + glyph.y_int - line_y),
                );

                let color = if is_color {
                    Color::rgba(255, 255, 255, 255)
                } else {
                    match glyph.color_opt {
                        Some(color) => color,
                        None => self.default_color,
                    }
                };

                //Bounds used from Glyphon
                let bounds_min_x = self.bounds.left.max(0);
                let bounds_min_y = self.bounds.bottom.max(0);
                let bounds_max_x =
                    self.bounds.right.min(system.screen_size[0] as i32);
                let bounds_max_y =
                    self.bounds.top.min(system.screen_size[1] as i32);

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
                    position: [x as f32, y as f32, self.pos[2] as f32],
                    hw: [width as f32, height as f32],
                    tex_coord: [u as f32, v as f32],
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
    pub fn update<Controls>(
        &mut self,
        cache: &mut SwashCache,
        text_atlas: &mut AtlasGroup<CacheKey, (i32, i32)>,
        emoji_atlas: &mut AtlasGroup<CacheKey, (i32, i32)>,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        system: &System<Controls>,
    ) -> Result<bool, AscendingError>
    where
        Controls: camera::controls::Controls,
    {
        if self.changed
            && self.create_quad(
                cache,
                text_atlas,
                emoji_atlas,
                queue,
                device,
                system,
            )?
        {
            self.changed = false;
            return Ok(true);
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

impl Default for TextRender {
    fn default() -> Self {
        Self::new()
    }
}
