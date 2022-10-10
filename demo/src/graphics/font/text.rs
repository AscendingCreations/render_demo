use fontdue::{
    layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle},
    Font, FontSettings,
};

#[repr(C)]
#[derive(Clone)]
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

pub struct TextLayout {
    /// X, Y, GroupID for loaded map.
    pub layout: Layout<U = FontColor>,
    /// Vertex array in bytes. Does not need to get changed exept on Text update.
    pub bytes: Vec<u8>,
    /// If the location or map array id changed. Rebuild the vertex buffer.
    pub changed: bool,
}

impl Map {
    pub fn create_quad(&mut self) {}

    pub fn new(coord_sys: CoordinateSystem) -> Self {
        let image = ImageBuffer::new(32, 256);

        Self {
            layout: Layout::new(coord_sys),
            bytes: Vec::new(),
            changed: true,
        }
    }

    pub fn reset(&mut self, settings: LayoutSettings) {
        self.layout.reset(&settings);
    }

    pub fn append(
        &mut self,
        text: &str,
        fonts: &[Font],
        font_size: f32,
        font_index: usize,
    ) {
        self.layout.append(
            fonts.as_slice(),
            &TextStyle::with_user_data(
                text,
                font_size,
                font_index,
                FontColor::default(),
            ),
        );
        self.changed = true;
    }

    /// used to check and update the vertex array.
    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        map_textures: &mut MapTextures,
    ) {
        if self.changed {
            self.create_quad();
            self.changed = false;
        }
    }
}
