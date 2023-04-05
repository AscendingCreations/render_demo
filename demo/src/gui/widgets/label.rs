use crate::{
    Control, Event, FrameTime, Identity, Metrics, ModifiersState, MouseButton,
    SystemEvent, TextBounds, UIBuffer, UiField, UiFlags, Widget, WidgetRef, UI,
};
use cosmic_text::{Align, Attrs};
use graphics::*;

pub struct Label {
    identity: Identity,
    text: Text,
}

impl Label {
    pub fn new(
        renderer: &mut GpuRenderer,
        identity: Identity,
        metrics: Option<Metrics>,
        pos: Vec3,
        size: Vec2,
        value: String,
        attrs: Attrs,
    ) -> Label {
        let bounds = Some(TextBounds::new(
            pos.x,
            pos.y + size.y,
            pos.x + size.x,
            pos.y.max(0.0),
        ));
        let mut text = Text::new(renderer, metrics, pos, size, bounds);

        text.set_text(renderer, &value, attrs);
        text.set_buffer_size(renderer, size.x as i32, size.y as i32);

        Self { identity, text }
    }

    pub fn set_default_color(&mut self, default_color: Color) -> &mut Self {
        self.text.set_default_color(default_color);
        self
    }

    pub fn set_offset(&mut self, offsets: Vec2) -> &mut Self {
        self.text.set_offset(offsets);
        self
    }
}

impl<Message> Control<Message> for Label {
    fn get_id(&self) -> &Identity {
        &self.identity
    }

    fn check_mouse_bounds(&self, mouse_pos: Vec2) -> bool {
        self.text.check_mouse_bounds(mouse_pos)
    }

    fn get_bounds(&self) -> Vec4 {
        let pos = self.text.pos;
        let size = self.text.size;

        Vec4::new(pos.x, pos.y, size.x, size.y)
    }

    fn get_size(&self) -> Vec2 {
        self.text.size
    }

    fn get_position(&mut self) -> Vec3 {
        self.text.pos
    }

    fn set_position(&mut self, position: Vec3) {
        self.text.pos = position;
    }

    fn default_actions(&self) -> Vec<UiFlags> {
        vec![UiFlags::CanClickBehind]
    }

    fn event(
        &mut self,
        _actions: UiField,
        _ui_buffer: &mut UIBuffer,
        _renderer: &mut GpuRenderer,
        _event: SystemEvent,
        _events: &mut Vec<Message>,
    ) {
    }

    fn draw(
        &mut self,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        _frametime: &FrameTime,
    ) -> Result<(), AscendingError> {
        ui_buffer.text_renderer.text_update(
            &mut self.text,
            &mut ui_buffer.text_atlas,
            renderer,
        )
    }
}
