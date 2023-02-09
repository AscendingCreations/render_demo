use crate::{GuiRender, Handle, Identity, Widget, Widgets};
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
}

pub type InternalDrawRef =
    Box<dyn Fn(&mut Widget, FrameTime, &mut GuiRender, &mut Renderer)>;
pub type InternalMousePresentRef = Box<dyn Fn(&mut Widget, bool)>;
pub type InternalMouseScrollRef =
    Box<dyn Fn(&mut Widget, (f32, f32), ModifiersState)>;
pub type InternalMousePressRef =
    Box<dyn Fn(&mut Widget, u32, bool, ModifiersState)>;
pub type InternalKeyPressRef =
    Box<dyn Fn(&mut Widget, KeyboardInput, ModifiersState)>;

pub enum InternalCallBacks {
    Draw(InternalDrawRef),
    MousePresent(InternalMousePresentRef),
    MouseScroll(InternalMouseScrollRef),
    MousePress(InternalMousePressRef),
    KeyPress(InternalKeyPressRef),
    PositionChange(Box<dyn Fn(&mut Widget)>),
    BoundsChange(Box<dyn Fn(&mut Widget)>),
}

pub type DrawRef<T> = Box<
    dyn Fn(
        &mut Widget,
        FrameTime,
        &mut GuiRender,
        &mut Renderer,
        &mut Commands,
        &mut T,
    ),
>;
pub type MousePresentRef<T> =
    Box<dyn Fn(&mut Widget, bool, &mut Commands, &mut T)>;
pub type MouseScrollRef<T> =
    Box<dyn Fn(&mut Widget, (f32, f32), ModifiersState, &mut Commands, &mut T)>;
pub type MousePressRef<T> =
    Box<dyn Fn(&mut Widget, u32, bool, ModifiersState, &mut Commands, &mut T)>;
pub type KeyPressRef<T> = Box<
    dyn Fn(&mut Widget, KeyboardInput, ModifiersState, &mut Commands, &mut T),
>;

pub enum CallBacks<T> {
    Draw(DrawRef<T>),
    MousePresent(MousePresentRef<T>),
    MouseScroll(MouseScrollRef<T>),
    MousePress(MousePressRef<T>),
    KeyPress(KeyPressRef<T>),
}

#[derive(Default)]
pub struct Commands {
    pub commands: Vec<Command>,
}

impl Commands {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, command: Command) {
        self.commands.push(command);
    }
}

//TODO Check what other commands may be needed!
pub enum Command {
    // Remove a widget by its known handle.
    RemoveByHandle(Handle),

    //remove the widget by its known name and id
    RemoveById(Identity),

    //use when adding new widgets that also have new parents.
    AddWidgetToParentId { widget: Widget, id: Identity },

    //add widgets to existing parents.
    AddWidgetToParentHandle { widget: Widget, handle: Handle },
}
