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

pub enum InternalCallBacks {
    Draw(Box<dyn Fn(&mut Widget, FrameTime, &mut GuiRender, &mut Renderer)>),
    MousePresent(Box<dyn Fn(&mut Widget, bool)>),
    MouseScroll(Box<dyn Fn(&mut Widget, (f32, f32), ModifiersState)>),
    MousePress(Box<dyn Fn(&mut Widget, u32, bool, ModifiersState)>),
    KeyPress(Box<dyn Fn(&mut Widget, KeyboardInput, ModifiersState)>),
    PositionChange(Box<dyn Fn(&mut Widget)>),
    BoundsChange(Box<dyn Fn(&mut Widget)>),
}

pub enum CallBacks<T> {
    Draw(
        Box<
            dyn Fn(
                &mut Widget,
                FrameTime,
                &mut GuiRender,
                &mut Renderer,
                &mut Commands,
                &mut T,
            ),
        >,
    ),
    MousePresent(Box<dyn Fn(&mut Widget, bool, &mut Commands, &mut T)>),
    MouseScroll(Box<dyn Fn(&mut Widget, (f32, f32), ModifiersState, &mut Commands, &mut T)>),
    MousePress(Box<dyn Fn(&mut Widget, u32, bool, ModifiersState, &mut Commands, &mut T)>),
    KeyPress(Box<dyn Fn(&mut Widget, KeyboardInput, ModifiersState, &mut Commands, &mut T)>),
}

#[derive(Debug, Clone, Default)]
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
