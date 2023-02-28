use crate::{Control, Identity, UIBuffer, Widget, WidgetRef, UI};
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
