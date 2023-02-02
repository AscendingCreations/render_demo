use crate::{GuiRender, Widget, Widgets};
use graphics::*;
use input::FrameTime;
use std::any::Any;
use std::marker::PhantomData;
use std::{cell::RefCell, collections::VecDeque, rc::Rc, vec::Vec};
use ubits::bitfield;
use winit::event::{KeyboardInput, ModifiersState};

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
