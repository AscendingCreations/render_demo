use crate::{Handle, Identity, UIBuffer, Value, Widget, UI};
use graphics::*;
use input::FrameTime;
use std::any::Any;
use std::marker::PhantomData;
use std::{cell::RefCell, collections::VecDeque, rc::Rc, vec::Vec};
use ubits::bitfield;
use winit::event::{KeyboardInput, ModifiersState, MouseButton};

#[derive(Clone)]
pub enum SystemEvent {
    /// Present?.
    MousePresent(bool),
    /// Delta X, Y, Mod State.
    MouseScroll(f32, f32, ModifiersState),
    /// Button, Pressed?, Mod State.
    MousePress(MouseButton, bool, ModifiersState),
    /// Input Mod state.
    KeyPress(KeyboardInput, ModifiersState),
    /// Offset.
    PositionChange(Vec3),
    /// Offset, Parent Bounds.
    BoundsChange(Vec3, Option<WorldBounds>),
    /// Offset.
    Scroll(Vec3),
    // Changed?.
    FocusChange(bool),
}

#[derive(Clone)]
pub enum WidgetEvent {
    /// current value, Max value
    Scroll(usize, usize),
    None,
}
