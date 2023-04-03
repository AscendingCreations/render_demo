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

/*pub type InternalDrawRef<T> =
    fn(&mut Widget<T>, &mut UI<T>, );
pub type InternalBooleanRef<T> =
    fn(&mut Widget<T>, &mut UI<T>, &mut GpuRenderer, bool);
pub type InternalMouseScrollRef<T> = fn(
    &mut Widget<T>,
    &mut UI<T>,
    &mut GpuRenderer,
    (f32, f32),
    ModifiersState,
);
pub type InternalMousePressRef<T> = fn(
    &mut Widget<T>,
    &mut UI<T>,
    &mut GpuRenderer,
    MouseButton,
    bool,
    ModifiersState,
);
pub type InternalKeyPressRef<T> = fn(
    &mut Widget<T>,
    &mut UI<T>,
    &mut GpuRenderer,
    KeyboardInput,
    ModifiersState,
);
pub type InternalUpdate<T> = fn(&mut Widget<T>, &mut UI<T>, &mut GpuRenderer);

#[derive(Copy, Clone)]
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
    fn(&mut Widget<T>, &mut UI<T>, &mut GpuRenderer, &FrameTime, &mut T);
pub type MousePresentRef<T> =
    fn(&mut Widget<T>, &mut UI<T>, &mut GpuRenderer, bool, &mut T);
pub type MouseScrollRef<T> =
    fn(
        &mut Widget<T>,
        &mut UI<T>,
        &mut GpuRenderer,
        (f32, f32),
        ModifiersState,
        &mut T,
    );
pub type MousePressRef<T> =
    fn(
        &mut Widget<T>,
        &mut UI<T>,
        &mut GpuRenderer,
        MouseButton,
        bool,
        ModifiersState,
        &mut T,
    );
pub type KeyPressRef<T> =
    fn(
        &mut Widget<T>,
        &mut UI<T>,
        &mut GpuRenderer,
        KeyboardInput,
        ModifiersState,
        &mut T,
    );
pub type ValueChangedRef<T> =
    fn(&mut Widget<T>, &mut UI<T>, &mut GpuRenderer, ReturnValue, &mut T);

#[derive(Copy, Clone)]
pub enum CallBacks<T> {
    Draw(DrawRef<T>),
    MousePresent(MousePresentRef<T>),
    MouseScroll(MouseScrollRef<T>),
    MousePress(MousePressRef<T>),
    KeyPress(KeyPressRef<T>),
    ValueChanged(ValueChangedRef<T>),
}
*/
