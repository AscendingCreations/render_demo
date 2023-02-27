use crate::{Handle, Identity, ReturnValue, UIBuffer, Widget, UI};
use graphics::*;
use input::FrameTime;
use std::any::Any;
use std::marker::PhantomData;
use std::{cell::RefCell, collections::VecDeque, rc::Rc, vec::Vec};
use ubits::bitfield;
use winit::event::{KeyboardInput, ModifiersState};

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct CallBackKey {
    identity: Identity,
    callback: CallBack,
}

impl CallBackKey {
    pub(crate) fn new(identity: &Identity, callback: CallBack) -> Self {
        Self {
            identity: identity.to_owned(),
            callback,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub(crate) enum CallBack {
    Draw,
    MousePresent,
    MouseScroll,
    MousePress,
    KeyPress,
    PositionChange,
    BoundsChange,
    FocusChange,
    ValueChanged,
}

pub type InternalDrawRef<T> = Box<dyn Fn(&mut Widget, &mut UI<T>, &FrameTime)>;
pub type InternalBooleanRef<T> =
    Box<dyn Fn(&mut Widget, &mut UI<T>, bool) -> bool>;
pub type InternalMouseScrollRef<T> =
    Box<dyn Fn(&mut Widget, &mut UI<T>, (f32, f32), ModifiersState) -> bool>;
pub type InternalMousePressRef<T> =
    Box<dyn Fn(&mut Widget, &mut UI<T>, u32, bool, ModifiersState) -> bool>;
pub type InternalKeyPressRef<T> =
    Box<dyn Fn(&mut Widget, &mut UI<T>, KeyboardInput, ModifiersState) -> bool>;
pub type InternalUpdate<T> = Box<dyn Fn(&mut Widget, &mut UI<T>)>;

pub enum InternalCallBacks<T> {
    Draw(InternalDrawRef<T>),
    MousePresent(InternalBooleanRef<T>),
    MouseScroll(InternalMouseScrollRef<T>),
    MousePress(InternalMousePressRef<T>),
    KeyPress(InternalKeyPressRef<T>),
    PositionChange(InternalUpdate<T>),
    BoundsChange(InternalUpdate<T>),
    FocusChange(InternalBooleanRef<T>),
}

pub type DrawRef<T> = Box<dyn Fn(&mut Widget, &mut UI<T>, &FrameTime, &mut T)>;
pub type MousePresentRef<T> =
    Box<dyn Fn(&mut Widget, &mut UI<T>, bool, &mut T)>;
pub type MouseScrollRef<T> =
    Box<dyn Fn(&mut Widget, &mut UI<T>, (f32, f32), ModifiersState, &mut T)>;
pub type MousePressRef<T> =
    Box<dyn Fn(&mut Widget, &mut UI<T>, u32, bool, ModifiersState, &mut T)>;
pub type KeyPressRef<T> =
    Box<dyn Fn(&mut Widget, &mut UI<T>, KeyboardInput, ModifiersState, &mut T)>;
pub type ValueChangedRef<T> =
    Box<dyn Fn(&mut Widget, &mut UI<T>, ReturnValue, &mut T)>;

pub enum CallBacks<T> {
    Draw(DrawRef<T>),
    MousePresent(MousePresentRef<T>),
    MouseScroll(MouseScrollRef<T>),
    MousePress(MousePressRef<T>),
    KeyPress(KeyPressRef<T>),
    ValueChanged(ValueChangedRef<T>),
}
