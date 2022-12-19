use crate::{AscendingError, AtlasGroup, Color, TextVertex};
use cosmic_text::{Buffer, CacheKey, FontSystem, SwashCache, SwashContent};

// This is a text layer buffer for rendering text to the screen.
// Can be used multiple times for multiple layers of text.
pub struct Text {
    pub cache: SwashCache<'static>,
    /// Vertex array in bytes. This Holds regular glyphs
    pub text_bytes: Vec<u8>,
    ///default color.
    pub color: Color,
    /// will rerender everything and needs to be reset to false.
    cleared: bool,
}

impl Text {
    pub fn create_quad(
        &mut self,
        pos: [i32; 3],
        buffer: &mut Buffer<'static>,
        text_atlas: &mut AtlasGroup<CacheKey, (i32, i32)>,
        emoji_atlas: &mut AtlasGroup<CacheKey, (i32, i32)>,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) -> Result<(), AscendingError> {
        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                if text_atlas.atlas.get(&glyph.cache_key).is_some()
                    || emoji_atlas.atlas.get(&glyph.cache_key).is_some()
                {
                    continue;
                }

                let image =
                    self.cache.get_image_uncached(glyph.cache_key).unwrap();
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

        for run in buffer.layout_runs() {
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
                    (pos[0] + glyph.x_int + position.0) as f32,
                    (pos[1] + glyph.y_int - line_y) as f32,
                );

                let (u1, v1) = (u as f32, v as f32);

                let color = if is_color {
                    Color::rgba(255, 255, 255, 255)
                } else {
                    match glyph.color_opt {
                        Some(color) => color,
                        None => self.color,
                    }
                };

                let default = TextVertex {
                    position: [x, y, pos[2] as f32],
                    hw: [width, height],
                    tex_coord: [u1, v1],
                    layer: allocation.layer as u32,
                    color: color.0,
                    is_color: is_color as u32,
                };

                text_buf.push(default);
            }
        }

        self.text_bytes = bytemuck::cast_slice(&text_buf).to_vec();
        Ok(())
    }

    pub fn new(font_system: &'static FontSystem, color: Option<Color>) -> Self {
        Self {
            cache: SwashCache::new(font_system),
            text_bytes: Vec::new(),
            color: color.unwrap_or(Color::rgba(0, 0, 0, 255)),
            cleared: false,
        }
    }

    /// Sets cleared to false.
    pub fn reset_cleared(&mut self) {
        self.cleared = false;
    }

    /// used to check and update the vertex array.
    /// must call build_layout before you can Call this.
    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        pos: [i32; 3],
        buffer: &mut Buffer<'static>,
        text_atlas: &mut AtlasGroup<CacheKey, (i32, i32)>,
        emoji_atlas: &mut AtlasGroup<CacheKey, (i32, i32)>,
    ) -> bool {
        if buffer.redraw() || self.cleared {
            let _ = self.create_quad(
                pos,
                buffer,
                text_atlas,
                emoji_atlas,
                queue,
                device,
            );

            true
        } else {
            false
        }
    }
}
