use crate::{
    Control, FrameTime, Identity, ModifiersState, MouseButton, UIBuffer,
    UiFlags, Widget, WidgetRef, UI,
};
use graphics::*;

pub struct Button {
    over_color: Color,
    clicked_color: Color,
    color: Color,
    border_over_color: Color,
    border_clicked_color: Color,
    border_color: Color,
    shape: Rect,
}

fn draw<T>(
    control: &mut Widget,
    ui: &mut UI<T>,
    _device: &GpuDevice,
    _time: &FrameTime,
) {
    if let Some(button) =
        control.ui.as_mut().as_mut_any().downcast_mut::<Button>()
    {
        ui.ui_buffer_mut()
            .ui_buffer
            .add_buffer_store(button.shape.update())
    }
}

fn mouse_over<T>(
    control: &mut Widget,
    ui: &mut UI<T>,
    device: &GpuDevice,
    is_over: bool,
) {
    if let Some(button) =
        control.ui.as_mut().as_mut_any().downcast_mut::<Button>()
    {
        button.shape.set_color(
            device,
            &mut ui.ui_buffer_mut().ui_atlas,
            if is_over {
                button.color
            } else {
                button.border_over_color
            },
        );
    }
}

fn mouse_button<T>(
    control: &mut Widget,
    ui: &mut UI<T>,
    device: &GpuDevice,
    mouse_btn: MouseButton,
    is_pressed: bool,
    _mods: ModifiersState,
) {
    let mouse_over = control.actions.get(crate::gui::UiFlags::MouseOver);

    if let Some(button) =
        control.ui.as_mut().as_mut_any().downcast_mut::<Button>()
    {
        if mouse_btn == MouseButton::Left {
            if mouse_over {
                button.shape.set_color(
                    device,
                    &mut ui.ui_buffer_mut().ui_atlas,
                    if is_pressed {
                        button.clicked_color
                    } else {
                        button.border_clicked_color
                    },
                );
            } else {
                button.shape.set_color(
                    device,
                    &mut ui.ui_buffer_mut().ui_atlas,
                    button.color,
                );
            }
        }
    }
}

impl Button {
    fn button<T>(
        ui: &mut UI<T>,
        device: &GpuDevice,
        id: Identity,
        position: Vec3,
        size: Vec2,
        parent_id: Option<Identity>,
        hidden: bool,
    ) -> WidgetRef {
        let mut shape = Rect::default();

        shape
            .set_color(
                device,
                &mut ui.ui_buffer_mut().ui_atlas,
                Color::rgba(20, 20, 20, 255),
            )
            .set_position(position)
            .set_size(size);

        let button = Self {
            over_color: Color::rgba(40, 40, 40, 255),
            clicked_color: Color::rgba(60, 60, 60, 255),
            color: Color::rgba(20, 20, 20, 255),
            border_over_color: Color::rgba(0, 0, 0, 255),
            border_clicked_color: Color::rgba(0, 0, 0, 255),
            border_color: Color::rgba(0, 0, 0, 255),
            shape,
        };

        let widget: WidgetRef = Widget::new(id, button).into();

        if hidden {
            ui.add_hidden_widget_by_id(parent_id, widget.clone());
        } else {
            ui.add_widget_by_id(parent_id, widget.clone());
        }

        widget
    }
}

impl Control for Button {
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
}
