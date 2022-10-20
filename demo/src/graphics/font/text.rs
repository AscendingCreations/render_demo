pub(crate) use crate::graphics::{
    allocation::Allocation, Atlas, AtlasGroup, BufferLayout, BufferPass,
    ScreenUniform, TextVertex,
};
use core::borrow::Borrow;
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

pub struct Text {
    /// glyph string layout.
    pub glyphs: Vec<Glyph>,
    /// Font PX size,
    pub px: f32,
    /// Font Index,
    pub font_index: usize,
    /// Layout settings for the Text.
    pub settings: LayoutSettings,
    /// Vertex array in bytes. Does not need to get changed except on Text update.
    /// This Holds the Created layout for rendering.
    pub bytes: Vec<u8>,
    /// The string index where rendering starts from.
    pub cursor: usize,
    //Position to Render the Text At based on its Bottom Left Corner.
    pub pos: [f32; 3],
    /// The system layout of the Text Appened
    layout: Layout,
    /// If the location or map array id changed. Rebuild the vertex buffer.
    pub changed: bool,
}

impl Default for Text {
    fn default() -> Self {
        let mut layout = Layout::new(CoordinateSystem::PositiveYUp);
        layout.reset(&LayoutSettings::default());

        Self {
            glyphs: Vec::new(),
            px: 12.0,
            font_index: 0,
            cursor: 0,
            settings: LayoutSettings::default(),
            bytes: Vec::new(),
            pos: [0.0, 0.0, 1.0],
            layout,
            changed: true,
        }
    }
}

impl Text {
    pub fn create_quad(
        &mut self,
        fonts: &[Font],
        atlas: &mut Atlas<GlyphRasterConfig>,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) {
        for glyph in self.layout.glyphs() {
            if atlas.get(&glyph.key).is_some() {
                continue;
            }

            let font = &fonts[glyph.font_index];
            let (metrics, bitmap) = font.rasterize_config(glyph.key);

            if glyph.char_data.rasterize() {
                // Find a position in the packer
                let mut rows: Vec<u8> =
                    Vec::with_capacity(metrics.width * metrics.height + 1);
                rows.extend_from_slice(
                    &bitmap[0..metrics.width * metrics.height],
                );

                let _ = atlas
                    .upload(
                        glyph.key,
                        &rows,
                        metrics.width as u32,
                        metrics.height as u32,
                        device,
                        queue,
                    )
                    .unwrap();
            }
        }

        let mut buffer = Vec::with_capacity(self.glyphs.len() * 4);

        for (pos, glyph) in self.layout.glyphs().iter().enumerate() {
            if let Some(allocation) = atlas.get(&glyph.key) {
                let (u, v, width, height) = allocation.rect();
                let (u, v, width, height) =
                    (u as i32, v as i32, width as i32, height as i32);
                let (x, y) = (
                    self.pos[0] + glyph.x.trunc(),
                    self.pos[1] + glyph.y.trunc(),
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

                let color = [
                    self.glyphs[pos].color.r,
                    self.glyphs[pos].color.g,
                    self.glyphs[pos].color.b,
                    self.glyphs[pos].color.a,
                ];

                let mut other = vec![
                    TextVertex {
                        position: [x, y, self.pos[2]],
                        tex_coord: [u1, v2],
                        layer: allocation.layer as u32,
                        color,
                    },
                    TextVertex {
                        position: [w, y, self.pos[2]],
                        tex_coord: [u2, v2],
                        layer: allocation.layer as u32,
                        color,
                    },
                    TextVertex {
                        position: [w, h, self.pos[2]],
                        tex_coord: [u2, v1],
                        layer: allocation.layer as u32,
                        color,
                    },
                    TextVertex {
                        position: [x, h, self.pos[2]],
                        tex_coord: [u1, v1],
                        layer: allocation.layer as u32,
                        color,
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

    pub fn set_pos(&mut self, pos: &[f32; 3]) {
        self.pos = *pos;
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
        self.layout = Layout::new(coord_sys);
        self.changed = true;
        self
    }

    ///This will set the Text Blobs Settings. but will not rebuild the Layout. Must call Build_layout;
    pub fn settings(mut self, settings: LayoutSettings) -> Self {
        self.settings = settings;
        self.changed = true;
        self
    }

    pub fn build_layout(&mut self, fonts: &[Font]) {
        let string: String = self.glyphs.iter().map(|glyph| glyph.ch).collect();
        self.layout.reset(&self.settings);
        self.layout
            .append(fonts, &TextStyle::new(&string, self.px, self.font_index));
        self.changed = true;
    }

    /// Gets the height of the Box so you can Position
    /// the Text from the bottom left corner rather than Top left.
    pub fn get_box_height(&self) -> f32 {
        self.layout.height()
    }

    /// Appends to end of string.
    /// Must call build_layout after you are finished Modifing Text.
    pub fn append(&mut self, string: &str) {
        self.append_with(string, FontColor::default());
    }

    /// clears the string.
    /// Must call build_layout after you are finished Modifing Text.
    pub fn clear(&mut self) {
        self.bytes.clear();
        self.glyphs.clear();
        self.changed = true;
    }

    /// Appends to end of string with Color
    /// Must call build_layout after you are finished Modifing Text.
    pub fn append_with(&mut self, string: &str, color: FontColor) {
        string
            .chars()
            .for_each(|ch| self.glyphs.push(Glyph::new(ch, color)));
        self.changed = true;
    }

    /// Inserts char into Cursor Position, Will panic if Cursor is outside of bounds.
    /// Must call build_layout after you are finished Modifing Text.
    pub fn insert(&mut self, ch: char, cursor: usize) {
        self.insert_with(ch, cursor, FontColor::default());
    }

    /// Inserts char into Cursor Position with color, Will panic if Cursor is outside of bounds.
    /// Must call build_layout after you are finished Modifing Text.
    pub fn insert_with(&mut self, ch: char, cursor: usize, color: FontColor) {
        self.glyphs.insert(cursor, Glyph::new(ch, color));
        self.changed = true;
    }

    /// Inserts str into Cursor Position Will panic if Cursor is outside of bounds.
    /// Must call build_layout after you are finished Modifing Text.
    pub fn insert_str(&mut self, string: &str, cursor: usize) {
        self.insert_str_with(string, cursor, FontColor::default())
    }

    /// Inserts str into Cursor Position with color, Will panic if Cursor is outside of bounds.
    /// Must call build_layout after you are finished Modifing Text.
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

    /// Replaces cursor Range with str into Cursor Position, Will panic if Cursor is outside of bounds.
    /// string can be bigger that replacing parts or smaller.
    /// Must call build_layout after you are finished Modifing Text.
    pub fn replace_range(&mut self, string: &str, range: Range<usize>) {
        self.replace_range_with(string, range, FontColor::default());
    }

    /// Replaces cursor Range with str into Cursor Position with color, Will panic if Cursor is outside of bounds.
    /// string can be bigger that replacing parts or smaller.
    /// Must call build_layout after you are finished Modifing Text.
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

    /// removes cursor Range from the char string, Will panic if Cursor is outside of bounds.
    /// Must call build_layout after you are finished Modifing Text.
    pub fn remove_range(&mut self, range: Range<usize>) {
        self.glyphs.drain(range).for_each(drop);
        self.changed = true;
    }

    /// used to check and update the vertex array.
    /// must call build_layout before you can Call this.
    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        fonts: &[Font],
        atlas: &mut AtlasGroup<GlyphRasterConfig>,
    ) -> bool {
        if self.changed {
            self.create_quad(fonts, &mut atlas.atlas, queue, device);
            self.changed = false;
            true
        } else {
            false
        }
    }
}
