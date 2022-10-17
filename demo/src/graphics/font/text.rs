pub(crate) use crate::graphics::{
    allocation::Allocation, Atlas, BufferLayout, BufferPass, TextVertex,
};
use core::borrow::Borrow;
use fontdue::{
    layout::{
        CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle,
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

#[derive(Clone, Copy, Default, Debug)]
pub struct Glyph {
    pub ch: char,
    pub color: FontColor,
}

impl Glyph {
    pub fn new(ch: char, color: FontColor) -> Self {
        Self { ch, color }
    }
}

struct UploadBounds {
    x_min: usize,
    x_max: usize,
    y_min: usize,
    y_max: usize,
}

pub struct Text {
    /// glyph string layout.
    pub glyphs: Vec<Glyph>,
    /// Font PX size,
    pub px: f32,
    /// Font Index,
    pub font_index: usize,
    /// Layout settings for the Text.
    pub settings: LayoutSettings,
    /// The direction that the Y coordinate increases in. Defaults to PositiveYDown
    pub coord_sys: CoordinateSystem,
    /// Vertex array in bytes. Does not need to get changed except on Text update.
    /// This Holds the Created layout for rendering.
    pub bytes: Vec<u8>,
    /// The string index where rendering starts from.
    pub cursor: usize,
    /// If the location or map array id changed. Rebuild the vertex buffer.
    pub changed: bool,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            glyphs: Vec::new(),
            px: 12.0,
            font_index: 0,
            settings: LayoutSettings::default(),
            coord_sys: CoordinateSystem::PositiveYDown,
            cursor: 0,
            bytes: Vec::new(),
            changed: true,
        }
    }
}

impl Text {
    pub fn create_quad(
        &mut self,
        layout: Layout,
        fonts: &[Font],
        atlas: &mut Atlas<GlyphRasterConfig>,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) {
        let mut upload_bounds = None::<UploadBounds>;

        for glyph in layout.glyphs() {
            if atlas.get(&glyph.key).is_some() {
                continue;
            }

            let font = &fonts[glyph.font_index];
            let (metrics, bitmap) = font.rasterize_config(glyph.key);

            if glyph.char_data.rasterize() {
                // Find a position in the packer
                let mut rows: Vec<u8> =
                    Vec::with_capacity(metrics.width * metrics.height + 1);
                rows.extend_from_slice(&bitmap);
                //rows.copy_from_slice(&bitmap);

                let allocation = atlas
                    .upload(
                        glyph.key,
                        &rows,
                        metrics.width as u32,
                        metrics.height as u32,
                        device,
                        queue,
                    )
                    .unwrap();
                let bounds = allocation.rect();
                match upload_bounds.as_mut() {
                    Some(ub) => {
                        ub.x_min = ub.x_min.min(bounds.0 as usize);
                        ub.x_max = ub.x_max.max(bounds.2 as usize);
                        ub.y_min = ub.y_min.min(bounds.1 as usize);
                        ub.y_max = ub.y_max.max(bounds.3 as usize);
                    }
                    None => {
                        upload_bounds = Some(UploadBounds {
                            x_min: bounds.0 as usize,
                            x_max: bounds.2 as usize,
                            y_min: bounds.1 as usize,
                            y_max: bounds.3 as usize,
                        });
                    }
                }
            }
        }

        let mut buffer = Vec::with_capacity(self.glyphs.len() * 4);

        for (pos, glyph) in layout.glyphs().iter().enumerate() {
            if let Some(allocation) = atlas.get(&glyph.key) {
                let (u, v, width, height) = allocation.rect();

                let (x, y, w, h) = (
                    glyph.x.round(),
                    glyph.y.round(),
                    (glyph.x.round() as i32).saturating_add((width - 1) as i32)
                        as f32,
                    (glyph.y.round() as i32).saturating_add((height - 1) as i32)
                        as f32,
                );
                let (u1, v1, u2, v2) = (
                    u as f32,
                    v as f32,
                    u.saturating_add(width) as f32,
                    v.saturating_add(height) as f32,
                );

                let color = self.glyphs[pos].color;

                let mut other = vec![
                    TextVertex {
                        position: [x, y],
                        tex_coord: [u1, v2, allocation.layer as f32],
                        color: [
                            color.r as u32,
                            color.g as u32,
                            color.b as u32,
                            color.a as u32,
                        ],
                        dimension: [width as f32, height as f32],
                    },
                    TextVertex {
                        position: [w, y],
                        tex_coord: [u2, v2, allocation.layer as f32],
                        color: [
                            color.r as u32,
                            color.g as u32,
                            color.b as u32,
                            color.a as u32,
                        ],
                        dimension: [width as f32, height as f32],
                    },
                    TextVertex {
                        position: [w, h],
                        tex_coord: [u2, v1, allocation.layer as f32],
                        color: [
                            color.r as u32,
                            color.g as u32,
                            color.b as u32,
                            color.a as u32,
                        ],
                        dimension: [width as f32, height as f32],
                    },
                    TextVertex {
                        position: [x, h],
                        tex_coord: [u1, v1, allocation.layer as f32],
                        color: [
                            color.r as u32,
                            color.g as u32,
                            color.b as u32,
                            color.a as u32,
                        ],
                        dimension: [width as f32, height as f32],
                    },
                ];

                buffer.append(&mut other);
            }
        }

        self.bytes = bytemuck::cast_slice(&buffer).to_vec();
        self.changed = false;
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            glyphs: Vec::with_capacity(capacity),
            bytes: Vec::with_capacity(capacity * 20 * 4),
            ..Default::default()
        }
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
        self.changed = true;
    }

    pub fn font_index(mut self, index: usize) -> Self {
        self.font_index = index;
        self.changed = true;
        self
    }

    pub fn font_size(mut self, px: f32) -> Self {
        self.px = px;
        self.changed = true;
        self
    }

    pub fn coordinate_system(mut self, coord_sys: CoordinateSystem) -> Self {
        self.coord_sys = coord_sys;
        self.changed = true;
        self
    }

    pub fn settings(mut self, settings: LayoutSettings) -> Self {
        self.settings = settings;
        self.changed = true;
        self
    }

    //Appends to end of string.
    pub fn append(&mut self, string: &str) {
        self.append_with(string, FontColor::default());
    }

    pub fn append_with(&mut self, string: &str, color: FontColor) {
        string
            .chars()
            .for_each(|ch| self.glyphs.push(Glyph::new(ch, color)));
        self.changed = true;
    }

    pub fn insert(&mut self, ch: char, cursor: usize) {
        self.insert_with(ch, cursor, FontColor::default());
    }

    pub fn insert_with(&mut self, ch: char, cursor: usize, color: FontColor) {
        self.glyphs.insert(cursor, Glyph::new(ch, color));
        self.changed = true;
    }

    pub fn insert_str(&mut self, string: &str, cursor: usize) {
        self.insert_str_with(string, cursor, FontColor::default())
    }

    pub fn insert_str_with(
        &mut self,
        string: &str,
        mut cursor: usize,
        color: FontColor,
    ) {
        string.chars().for_each(|ch| {
            self.glyphs.insert(cursor, Glyph::new(ch, color));
            cursor += 1;
        });
        self.changed = true;
    }

    pub fn replace_range(&mut self, string: &str, range: Range<usize>) {
        self.replace_range_with(string, range, FontColor::default());
    }

    pub fn replace_range_with(
        &mut self,
        string: &str,
        range: Range<usize>,
        color: FontColor,
    ) {
        self.glyphs
            .splice(range, string.chars().map(|ch| Glyph::new(ch, color)))
            .for_each(drop);
        self.changed = true;
    }

    pub fn remove_range(&mut self, range: Range<usize>) {
        self.glyphs.drain(range).for_each(drop);
        self.changed = true;
    }

    /// used to check and update the vertex array.
    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        fonts: &[Font],
        atlas: &mut Atlas<GlyphRasterConfig>,
    ) {
        if self.changed {
            let string: String =
                self.glyphs.iter().map(|glyph| glyph.ch).collect();
            let mut layout = Layout::new(self.coord_sys);
            layout.reset(&self.settings);
            layout.append(
                fonts,
                &TextStyle::new(&string, self.px, self.font_index),
            );
            self.create_quad(layout, fonts, atlas, queue, device);
            self.changed = false;
        }
    }
}
