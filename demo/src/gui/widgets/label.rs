use crate::{
    CallBack, CallBackKey, Control, FrameTime, Identity, InternalCallBacks,
    Metrics, ModifiersState, MouseButton, TextBounds, UIBuffer, UiFlags,
    Widget, WidgetRef, UI,
};
use cosmic_text::Attrs;
use graphics::*;

pub struct Label {
    text: Text,
}

fn draw<T>(
    control: &mut Widget<T>,
    ui: &mut UI<T>,
    renderer: &mut GpuRenderer,
    _time: &FrameTime,
) {
    if let Some(label) =
        control.ui.as_mut().as_mut_any().downcast_mut::<Label>()
    {
        let ui_buffer = ui.ui_buffer_mut();
        ui_buffer
            .text_renderer
            .text_update(&mut label.text, &mut ui_buffer.text_atlas, renderer)
            .unwrap();
    }
}

impl Label {
    pub fn new(
        renderer: &mut GpuRenderer,
        metrics: Option<Metrics>,
        pos: Vec3,
        size: Vec2,
        value: String,
        default_color: Color,
        attrs: Attrs,
    ) -> Label {
        let bounds = Some(TextBounds::new(
            pos.x,
            pos.y,
            pos.x + size.x,
            (pos.y - size.y).max(0.0),
        ));
        let mut text = Text::new(renderer, metrics, pos, size, bounds);

        text.set_default_color(default_color);
        text.set_text(renderer, &value, attrs);
        text.set_buffer_size(renderer, size.x as i32, size.y as i32);

        Self { text }
    }
}

impl<T: 'static> Control<T> for Label {
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

    fn get_internal_callbacks(
        &self,
        id: &Identity,
    ) -> Vec<(InternalCallBacks<T>, CallBackKey)> {
        vec![(
            InternalCallBacks::Draw(draw),
            CallBackKey::new(id, CallBack::Draw),
        )]
    }

    fn default_actions(&self) -> Vec<UiFlags> {
        vec![UiFlags::CanClickBehind]
    }
}
