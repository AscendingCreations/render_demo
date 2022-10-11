use fontdue::{
    layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle},
    Font, FontSettings,
};
use std::ops::Range;

#[repr(C)]
#[derive(Clone, Copy)]
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

pub struct Text {
    /// String Blob used to Recreate the layout when changes are made.
    pub chars: Vec<(char, FontColor)>,
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
            chars: Vec::new(),
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
    pub fn create_quad(&mut self) {}

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            chars: Vec::with_capacity(capacity),
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
        let chars = string.chars();

        for ch in chars {
            self.chars.push((ch, color));
        }

        self.changed = true;
    }

    pub fn insert(&mut self, ch: char, cursor: usize) {
        self.insert_with(ch, cursor, FontColor::default());
    }

    pub fn insert_with(&mut self, ch: char, cursor: usize, color: FontColor) {
        self.chars.insert(cursor, (ch, color));
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
        let chars = string.chars();

        for ch in chars {
            self.chars.insert(cursor, (ch, color));
            cursor += 1;
        }

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
        let chars: Vec<char> = string.chars().collect();
        let len = chars.len();
        let iters = chars.into_iter().zip((0..len).into_iter().map(|_| color));
        let _: Vec<(char, FontColor)> =
            self.chars.splice(range, iters).collect();
        self.changed = true;
    }

    /// used to check and update the vertex array.
    pub fn update(&mut self, _queue: &wgpu::Queue) {
        if self.changed {
            self.create_quad();
            self.changed = false;
        }
    }
}
