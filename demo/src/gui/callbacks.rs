use crate::{Handle, Identity, ReturnValue, UIBuffer, Widget, UI};
use graphics::*;
use input::FrameTime;
use std::any::Any;
use std::marker::PhantomData;
use std::{cell::RefCell, collections::VecDeque, rc::Rc, vec::Vec};
use ubits::bitfield;
use winit::event::{KeyboardInput, ModifiersState, MouseButton};

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
pub enum CallBack {
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

pub type InternalDrawRef<T> =
    Box<dyn Fn(&mut Widget<T>, &mut UI<T>, &GpuDevice, &FrameTime)>;
pub type InternalBooleanRef<T> =
    Box<dyn Fn(&mut Widget<T>, &mut UI<T>, &GpuDevice, bool)>;
pub type InternalMouseScrollRef<T> = Box<
    dyn Fn(&mut Widget<T>, &mut UI<T>, &GpuDevice, (f32, f32), ModifiersState),
>;
pub type InternalMousePressRef<T> = Box<
    dyn Fn(
        &mut Widget<T>,
        &mut UI<T>,
        &GpuDevice,
        MouseButton,
        bool,
        ModifiersState,
    ),
>;
pub type InternalKeyPressRef<T> = Box<
    dyn Fn(
        &mut Widget<T>,
        &mut UI<T>,
        &GpuDevice,
        KeyboardInput,
        ModifiersState,
    ),
>;
pub type InternalUpdate<T> =
    Box<dyn Fn(&mut Widget<T>, &mut UI<T>, &GpuDevice)>;

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

pub type DrawRef<T> =
    Box<dyn Fn(&mut Widget<T>, &mut UI<T>, &GpuDevice, &FrameTime, &mut T)>;
pub type MousePresentRef<T> =
    Box<dyn Fn(&mut Widget<T>, &mut UI<T>, &GpuDevice, bool, &mut T)>;
pub type MouseScrollRef<T> = Box<
    dyn Fn(
        &mut Widget<T>,
        &mut UI<T>,
        &GpuDevice,
        (f32, f32),
        ModifiersState,
        &mut T,
    ),
>;
pub type MousePressRef<T> = Box<
    dyn Fn(
        &mut Widget<T>,
        &mut UI<T>,
        &GpuDevice,
        MouseButton,
        bool,
        ModifiersState,
        &mut T,
    ),
>;
pub type KeyPressRef<T> = Box<
    dyn Fn(
        &mut Widget<T>,
        &mut UI<T>,
        &GpuDevice,
        KeyboardInput,
        ModifiersState,
        &mut T,
    ),
>;
pub type ValueChangedRef<T> =
    Box<dyn Fn(&mut Widget<T>, &mut UI<T>, &GpuDevice, ReturnValue, &mut T)>;

pub enum CallBacks<T> {
    Draw(DrawRef<T>),
    MousePresent(MousePresentRef<T>),
    MouseScroll(MouseScrollRef<T>),
    MousePress(MousePressRef<T>),
    KeyPress(KeyPressRef<T>),
    ValueChanged(ValueChangedRef<T>),
}
