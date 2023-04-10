use crate::{Handle, Identity, ReturnValue, UIBuffer, Widget, UI};
use graphics::*;
use input::FrameTime;
use std::any::Any;
use std::marker::PhantomData;
use std::{cell::RefCell, collections::VecDeque, rc::Rc, vec::Vec};
use ubits::bitfield;
use winit::event::{KeyboardInput, ModifiersState, MouseButton};

#[derive(Clone)]
pub enum SystemEvent {
    MousePresent(bool),
    MouseScroll(f32, f32, ModifiersState),
    MousePress(MouseButton, bool, ModifiersState),
    KeyPress(KeyboardInput, ModifiersState),
    PositionChange,
    BoundsChange,
    FocusChange(bool),
}
