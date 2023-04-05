use crate::{
    Control, FrameTime, Handle, Identity, ModifiersState, MouseButton,
    SystemEvent, UIBuffer, UiField, UiFlags, Widget, UI,
};
use graphics::*;

pub struct Button<Message> {
    identity: Identity,
    #[allow(clippy::type_complexity)]
    on_press:
        Box<dyn Fn(Identity, (MouseButton, bool, ModifiersState)) -> Message>,
    over_color: Color,
    clicked_color: Color,
    color: Color,
    border_over_color: Color,
    border_clicked_color: Color,
    border_color: Color,
    shape: Rect,
}

impl<Message> Button<Message> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        identity: Identity,
        position: Vec3,
        size: Vec2,
        border_width: f32,
        radius: Option<f32>,
        on_press: impl Fn(Identity, (MouseButton, bool, ModifiersState)) -> Message
            + 'static,
    ) -> Button<Message> {
        let mut shape = Rect::new(renderer);

        shape
            .set_color(
                renderer,
                &mut ui_buffer.ui_atlas,
                Color::rgba(20, 20, 20, 255),
            )
            .set_border_color(
                renderer,
                &mut ui_buffer.ui_atlas,
                Color::rgba(0, 0, 0, 255),
            )
            .set_border_width(border_width)
            .set_radius(radius)
            .set_position(position)
            .set_size(size);

        Self {
            identity,
            on_press: Box::new(on_press),
            over_color: Color::rgba(40, 40, 40, 255),
            clicked_color: Color::rgba(60, 60, 60, 255),
            color: Color::rgba(20, 20, 20, 255),
            border_over_color: Color::rgba(0, 0, 0, 255),
            border_clicked_color: Color::rgba(0, 0, 0, 255),
            border_color: Color::rgba(0, 0, 0, 255),
            shape,
        }
    }
}

impl<Message> Control<Message> for Button<Message> {
    fn get_id(&self) -> &Identity {
        &self.identity
    }

    fn check_mouse_bounds(&self, mouse_pos: Vec2) -> bool {
        self.shape.check_mouse_bounds(mouse_pos)
    }

    fn get_bounds(&self) -> Vec4 {
        let pos = self.shape.position;
        let size = self.shape.size;

        Vec4::new(pos.x, pos.y, size.x, size.y)
    }

    fn get_size(&self) -> Vec2 {
        self.shape.size
    }

    fn get_position(&mut self) -> Vec3 {
        self.shape.position
    }

    fn set_position(&mut self, position: Vec3) {
        self.shape.position = position;
    }

    fn default_actions(&self) -> Vec<UiFlags> {
        vec![UiFlags::ClickAble]
    }

    fn event(
        &mut self,
        actions: UiField,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        event: SystemEvent,
        events: &mut Vec<Message>,
    ) {
        match event {
            SystemEvent::MousePresent(is_over) => {
                self.shape
                    .set_color(
                        renderer,
                        &mut ui_buffer.ui_atlas,
                        if is_over { self.over_color } else { self.color },
                    )
                    .set_border_color(
                        renderer,
                        &mut ui_buffer.ui_atlas,
                        if is_over {
                            self.border_over_color
                        } else {
                            self.border_color
                        },
                    );
            }
            SystemEvent::MousePress(mouse_btn, is_pressed, mods) => {
                let mouse_over = actions.get(crate::gui::UiFlags::MouseOver);

                if mouse_btn == MouseButton::Left {
                    if mouse_over {
                        let colors = if is_pressed {
                            events.push((self.on_press)(
                                self.identity.clone(),
                                (mouse_btn, is_pressed, mods),
                            ));
                            (self.clicked_color, self.border_clicked_color)
                        } else {
                            (self.over_color, self.border_over_color)
                        };

                        self.shape
                            .set_color(
                                renderer,
                                &mut ui_buffer.ui_atlas,
                                colors.0,
                            )
                            .set_border_color(
                                renderer,
                                &mut ui_buffer.ui_atlas,
                                colors.1,
                            );
                    } else {
                        self.shape
                            .set_color(
                                renderer,
                                &mut ui_buffer.ui_atlas,
                                self.color,
                            )
                            .set_border_color(
                                renderer,
                                &mut ui_buffer.ui_atlas,
                                self.border_color,
                            );
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(
        &mut self,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        _frametime: &FrameTime,
    ) -> Result<(), AscendingError> {
        let index = self.shape.update(renderer);
        ui_buffer.ui_buffer.add_buffer_store(renderer, index);
        Ok(())
    }
}
