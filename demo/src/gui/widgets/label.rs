use crate::{
    Control, Event, FrameTime, Identity, Metrics, ModifiersState, MouseButton,
    SystemEvent, UIBuffer, UiField, UiFlags, Widget, WidgetEvent, WorldBounds,
    UI,
};
use cosmic_text::{Align, Attrs};
use gpu_graphics::*;

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
        let bounds = WorldBounds::new(
            pos.x,
            pos.y.max(0.0),
            pos.x + size.x,
            pos.y + size.y,
            1.0,
        );
        let mut text = Text::new(renderer, metrics, pos, size);

        text.set_text(renderer, &value, attrs)
            .set_bounds(Some(bounds));
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

    fn get_bounds(&self) -> Option<WorldBounds> {
        self.text.bounds
    }

    fn get_view_bounds(&self) -> Option<WorldBounds> {
        self.text.bounds
    }

    fn get_size(&self) -> Vec2 {
        self.text.size
    }

    fn get_position(&mut self) -> Vec3 {
        self.text.pos
    }

    fn default_actions(&self) -> UiField {
        let mut field = UiField::default();
        field.set(UiFlags::CanClickBehind);
        field
    }

    fn event(
        &mut self,
        _actions: UiField,
        _ui_buffer: &mut UIBuffer,
        _renderer: &mut GpuRenderer,
        event: SystemEvent,
        _events: &mut Vec<Message>,
    ) -> WidgetEvent {
        match event {
            SystemEvent::PositionChange(offset) => {
                self.text.set_position(self.text.pos + offset);
            }
            SystemEvent::BoundsChange(_offset, parent_bounds) => {
                self.text.set_bounds(Some(parent_bounds));
            }
            _ => {}
        }

        WidgetEvent::None
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
