pub(crate) use crate::graphics::{
    allocation::Allocation, Atlas, AtlasGroup, BufferLayout, BufferPass,
    RendererError, ScreenUniform, TextVertex,
};
use core::borrow::Borrow;
use cosmic_text::{CacheKey, FontSystem, SwashCache, SwashContent, TextBuffer};
use fontdue::{
    layout::{
        CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle,
        VerticalAlign,
    },
    Font, FontSettings,
};
use std::ops::Range;
use std::{
    borrow::Cow,
    collections::HashSet,
    error::Error,
    fmt::{self, Display, Formatter},
    iter,
    mem::size_of,
    num::{NonZeroU32, NonZeroU64},
    slice,
    sync::Arc,
};
use swash::scale::{image::Content, ScaleContext};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FontColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Default for FontColor {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
}

pub struct Text<'a> {
    pub cache: SwashCache<'a>,
    /// Vertex array in bytes. This Holds colored Glyphs
    pub emoji_bytes: Vec<u8>,
    /// Vertex array in bytes. This Holds regular glyphs
    pub text_bytes: Vec<u8>,
    pub cleared: bool,
}

impl<'a> Text<'a> {
    pub fn create_quad(
        &mut self,
        pos: [i32; 3],
        buffer: &mut TextBuffer<'a>,
        text_atlas: &mut AtlasGroup<CacheKey>,
        emoji_atlas: &mut AtlasGroup<CacheKey>,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) -> Result<(), RendererError> {
        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                if text_atlas.atlas.get(&glyph.cache_key).is_some()
                    || emoji_atlas.atlas.get(&glyph.cache_key).is_some()
                {
                    continue;
                }

                let matches = buffer.font_matches();

                let image =
                    self.cache.get_image_uncached(glyph.cache_key).unwrap();
                let bitmap = image.data;
                let is_color = match image.content {
                    SwashContent::Color => true,
                    SwashContent::Mask => false,
                    SwashContent::SubpixelMask => continue,
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
                            .ok_or(RendererError::AtlasFull)?;
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
                            .ok_or(RendererError::AtlasFull)?;
                    }
                }
            }
        }

        let mut emoji_buf = Vec::with_capacity(32 * 4);
        let mut text_buf = Vec::with_capacity(32 * 4);

        for run in buffer.layout_runs() {
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

                let (u, v, width, height) = allocation.rect();
                let (u, v, width, height) =
                    (u as i32, v as i32, width as i32, height as i32);

                let (x, y) = (
                    (pos[0] + glyph.x_int) as f32,
                    (pos[1] + line_y + glyph.y_int) as f32,
                );
                let (w, h) = (
                    (x as i32).saturating_add(width) as f32,
                    (y as i32).saturating_add(height) as f32,
                );

                let (u1, v1, u2, v2) = (
                    u as u16,
                    v as u16,
                    u.saturating_add(width) as u16,
                    v.saturating_add(height) as u16,
                );

                let color = [255, 255, 255, 255];

                let mut other = vec![
                    TextVertex {
                        position: [x, y, pos[2] as f32],
                        tex_coord: [u1, v2],
                        layer: allocation.layer as u32,
                        color,
                    },
                    TextVertex {
                        position: [w, y, pos[2] as f32],
                        tex_coord: [u2, v2],
                        layer: allocation.layer as u32,
                        color,
                    },
                    TextVertex {
                        position: [w, h, pos[2] as f32],
                        tex_coord: [u2, v1],
                        layer: allocation.layer as u32,
                        color,
                    },
                    TextVertex {
                        position: [x, h, pos[2] as f32],
                        tex_coord: [u1, v1],
                        layer: allocation.layer as u32,
                        color,
                    },
                ];

                if is_color {
                    emoji_buf.append(&mut other);
                } else {
                    text_buf.append(&mut other);
                }
            }
        }

        self.text_bytes = bytemuck::cast_slice(&text_buf).to_vec();
        self.emoji_bytes = bytemuck::cast_slice(&emoji_buf).to_vec();
        Ok(())
    }

    pub fn new(font_system: &'a FontSystem<'a>) -> Self {
        Self {
            cache: SwashCache::new(font_system),
            emoji_bytes: Vec::new(),
            text_bytes: Vec::new(),
            cleared: false,
        }
    }

    /// used to check and update the vertex array.
    /// must call build_layout before you can Call this.
    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        pos: [i32; 3],
        buffer: &mut TextBuffer<'a>,
        text_atlas: &mut AtlasGroup<CacheKey>,
        emoji_atlas: &mut AtlasGroup<CacheKey>,
    ) -> bool {
        if buffer.redraw {
            self.create_quad(
                pos,
                buffer,
                text_atlas,
                emoji_atlas,
                queue,
                device,
            );
            buffer.redraw = false;
            true
        } else {
            false
        }
    }
}
