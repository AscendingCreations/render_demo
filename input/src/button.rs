use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Button {
    // A virtual key on the keyboard.
    Key(winit::event::VirtualKeyCode),
    // A physical key on the keyboard independent of the keyboard layout.
    ScanCode(u32),
    // A mouse button.
    Mouse(winit::event::MouseButton),
}

impl From<winit::event::VirtualKeyCode> for Button {
    fn from(value: winit::event::VirtualKeyCode) -> Self {
        Button::Key(value)
    }
}

impl From<winit::event::MouseButton> for Button {
    fn from(value: winit::event::MouseButton) -> Self {
        Button::Mouse(value)
    }
}
