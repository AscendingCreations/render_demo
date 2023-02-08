use crate::{GuiRender, Identity, Widget, Widgets};
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
}

pub enum CallBacks<T> {
    Draw(
        Box<
            dyn Fn(
                &mut Widget,
                FrameTime,
                &mut GuiRender,
                &mut Renderer,
                &mut T,
            ),
        >,
    ),
    MousePresent(Box<dyn Fn(&mut Widget, bool, &mut T)>),
    MouseScroll(Box<dyn Fn(&mut Widget, (f32, f32), ModifiersState, &mut T)>),
    MousePress(Box<dyn Fn(&mut Widget, u32, bool, ModifiersState, &mut T)>),
    KeyPress(Box<dyn Fn(&mut Widget, KeyboardInput, ModifiersState, &mut T)>),
    PositionChange(Box<dyn Fn(&mut Widget, &mut T)>),
}

//TODO Check what other commands may be needed!
pub enum Commands {
    RemoveByHandle(Handle),
    RemoveById(Identity),
    AddWidgetToParentId { widget: Widget, id: Identity },
    AddWidgetToParentHandle { widget: Widget, handle: Handle },
}
