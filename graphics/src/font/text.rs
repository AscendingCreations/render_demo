use crate::{
    AscendingError, Bounds, Color, DrawOrder, GpuRenderer, Index, OrderedIndex,
    TextAtlas, TextVertex, Vec2, Vec3,
};
use cosmic_text::{
    Attrs, Buffer, Cursor, Metrics, SwashCache, SwashContent, Wrap,
};

pub struct Text {
    pub buffer: Buffer,
    pub pos: Vec3,
    pub size: Vec2,
    pub offsets: Vec2,
    pub default_color: Color,
    pub bounds: Option<Bounds>,
    pub store_id: Index,
    pub order: DrawOrder,
    /// Cursor the shaping is set too.
    pub cursor: Cursor,
    /// line the shaping is set too.
    pub line: i32,
    /// set scroll to render too.
    pub scroll: i32,
    /// Word Wrap Type. Default is Wrap::Word.
    pub wrap: Wrap,
    /// if the shader should render with the camera's view.
    pub use_camera: bool,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Text {
    pub fn create_quad(
        &mut self,
        cache: &mut SwashCache,
        atlas: &mut TextAtlas,
        renderer: &mut GpuRenderer,
    ) -> Result<(), AscendingError> {
        let count: usize =
            self.buffer.lines.iter().map(|line| line.text().len()).sum();
        let mut text_buf = Vec::with_capacity(count);

        for run in self.buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((0., 0.), 1.0);

                let (allocation, is_color) = if let Some(allocation) =
                    atlas.text.atlas.get(&physical_glyph.cache_key)
                {
                    (allocation, false)
                } else if let Some(allocation) =
                    atlas.emoji.atlas.get(&physical_glyph.cache_key)
                {
                    (allocation, true)
                } else {
                    let image = cache
                        .get_image_uncached(
                            &mut renderer.font_sys,
                            physical_glyph.cache_key,
                        )
                        .unwrap();
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
                            let allocation = atlas
                                .emoji
                                .atlas
                                .upload(
                                    physical_glyph.cache_key,
                                    &bitmap,
                                    width,
                                    height,
                                    Vec2::new(
                                        image.placement.left as f32,
                                        image.placement.top as f32,
                                    ),
                                    renderer,
                                )
                                .ok_or(AscendingError::AtlasFull)?;
                            (allocation, is_color)
                        } else {
                            let allocation = atlas
                                .text
                                .atlas
                                .upload(
                                    physical_glyph.cache_key,
                                    &bitmap,
                                    width,
                                    height,
                                    Vec2::new(
                                        image.placement.left as f32,
                                        image.placement.top as f32,
                                    ),
                                    renderer,
                                )
                                .ok_or(AscendingError::AtlasFull)?;
                            (allocation, is_color)
                        }
                    } else {
                        continue;
                    }
                };

                let position = allocation.data;
                let (u, v, width, height) = allocation.rect();
                let (mut u, mut v, mut width, mut height) =
                    (u as f32, v as f32, width as f32, height as f32);

                let (mut x, mut y) = (
                    (self.pos.x
                        + self.offsets.x
                        + physical_glyph.x as f32
                        + position.x),
                    (self.pos.y
                        + self.offsets.y
                        + self.size.y
                        + physical_glyph.y as f32
                        - run.line_y),
                );

                let color = is_color
                    .then(|| Color::rgba(255, 255, 255, 255))
                    .unwrap_or(match glyph.color_opt {
                        Some(color) => color,
                        None => self.default_color,
                    });

                let screensize = renderer.size();

                if let Some(bounds) = self.bounds {
                    //Bounds used from Glyphon
                    let bounds_min_x = bounds.left.max(0.0);
                    let bounds_min_y = bounds.bottom.max(0.0);
                    let bounds_max_x = bounds.right.min(screensize.width);
                    let bounds_max_y = bounds.top.min(screensize.height);

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

        if let Some(store) = renderer.get_buffer_mut(&self.store_id) {
            store.store = bytemuck::cast_slice(&text_buf).to_vec();
            store.changed = true;
        }

        self.order = DrawOrder::new(false, &self.pos, 1);
        self.changed = false;
        self.buffer.set_redraw(false);
        Ok(())
    }

    pub fn new(
        renderer: &mut GpuRenderer,
        metrics: Option<Metrics>,
        pos: Vec3,
        size: Vec2,
    ) -> Self {
        Self {
            buffer: Buffer::new(
                &mut renderer.font_sys,
                metrics.unwrap_or(Metrics::new(16.0, 16.0).scale(1.0)),
            ),
            pos,
            size,
            offsets: Vec2 { x: 0.0, y: 0.0 },
            bounds: None,
            store_id: renderer.new_buffer(),
            order: DrawOrder::new(false, &pos, 1),
            changed: true,
            default_color: Color::rgba(0, 0, 0, 255),
            use_camera: false,
            cursor: Cursor::default(),
            wrap: Wrap::Word,
            line: 0,
            scroll: 0,
        }
    }

    /// resets the TextRender bytes to empty for new bytes
    pub fn set_text(
        &mut self,
        renderer: &mut GpuRenderer,
        text: &str,
        attrs: Attrs,
    ) -> &mut Self {
        self.buffer.set_text(
            &mut renderer.font_sys,
            text,
            attrs,
            cosmic_text::Shaping::Advanced,
        );
        self.changed = true;
        self
    }

    /// For more advanced shaping and usage. Use set_changed() to set if you need it to make changes or not.
    /// This will not set the change to true. when changes are made you must set changed to true.
    pub fn get_text_buffer(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    /// cursor shaping sets the scroll.
    pub fn shape_until_cursor(
        &mut self,
        renderer: &mut GpuRenderer,
        cursor: Cursor,
    ) -> &mut Self {
        if self.cursor != cursor || self.changed {
            self.cursor = cursor;
            self.line = 0;
            self.changed = true;
            self.buffer
                .shape_until_cursor(&mut renderer.font_sys, cursor);
            self.scroll = self.buffer.scroll();
        }

        self
    }

    /// Line shaping does not use scroll or cursor for shaping.
    pub fn shape_until(
        &mut self,
        renderer: &mut GpuRenderer,
        line: i32,
    ) -> &mut Self {
        if self.line != line || self.changed {
            self.cursor = Cursor::default();
            self.line = line;
            self.changed = true;
            self.buffer.shape_until(&mut renderer.font_sys, line);
        }

        self
    }

    /// Does not use cursor or line but will use the last set scroll.
    pub fn shape_until_scroll(
        &mut self,
        renderer: &mut GpuRenderer,
    ) -> &mut Self {
        if self.changed {
            self.buffer.shape_until_scroll(&mut renderer.font_sys);
        }

        self
    }

    pub fn set_scroll(
        &mut self,
        renderer: &mut GpuRenderer,
        scroll: i32,
    ) -> &mut Self {
        if self.scroll != scroll {
            self.scroll = scroll;
            self.buffer.set_scroll(scroll);
            self.changed = true;
            self.buffer.shape_until_scroll(&mut renderer.font_sys);
        }

        self
    }

    pub fn set_change(&mut self, changed: bool) -> &mut Self {
        self.changed = changed;
        self
    }

    pub fn set_wrap(
        &mut self,
        renderer: &mut GpuRenderer,
        wrap: Wrap,
    ) -> &mut Self {
        if self.wrap != wrap {
            self.wrap = wrap;
            self.buffer.set_wrap(&mut renderer.font_sys, wrap);
            self.changed = true;
        }

        self
    }

    pub fn set_bounds(&mut self, bounds: Option<Bounds>) -> &mut Self {
        self.bounds = bounds;
        self.changed = true;
        self
    }

    pub fn set_position(&mut self, position: Vec3) -> &mut Self {
        self.pos = position;
        self.changed = true;
        self
    }

    pub fn set_default_color(&mut self, color: Color) -> &mut Self {
        self.default_color = color;
        self.changed = true;
        self
    }

    pub fn set_offset(&mut self, offsets: Vec2) -> &mut Self {
        self.offsets = offsets;
        self.changed = true;
        self
    }

    pub fn set_buffer_size(
        &mut self,
        renderer: &mut GpuRenderer,
        width: i32,
        height: i32,
    ) -> &mut Self {
        self.buffer.set_size(
            &mut renderer.font_sys,
            width as f32,
            height as f32,
        );
        self.changed = true;
        self
    }

    /// resets the TextRender bytes to empty for new bytes
    pub fn clear(&mut self, renderer: &mut GpuRenderer) -> &mut Self {
        self.buffer.set_text(
            &mut renderer.font_sys,
            "",
            cosmic_text::Attrs::new(),
            cosmic_text::Shaping::Basic,
        );
        self.changed = true;
        self
    }

    /// used to check and update the vertex array.
    /// must call build_layout before you can Call this.
    pub fn update(
        &mut self,
        cache: &mut SwashCache,
        atlas: &mut TextAtlas,
        renderer: &mut GpuRenderer,
    ) -> Result<OrderedIndex, AscendingError> {
        if self.changed {
            self.create_quad(cache, atlas, renderer)?;
        }

        Ok(OrderedIndex::new(self.order, self.store_id, 0))
    }

    pub fn check_mouse_bounds(&self, mouse_pos: Vec2) -> bool {
        mouse_pos[0] > self.pos.x
            && mouse_pos[0] < self.pos.x + self.size.x
            && mouse_pos[1] > self.pos.y
            && mouse_pos[1] < self.pos.y + self.size.y
    }
}
