use crate::{CallBack, UIBuffer};
use graphics::*;
use input::FrameTime;
use std::any::Any;
use std::{cell::RefCell, collections::VecDeque, rc::Rc, vec::Vec};
use ubits::bitfield;
use winit::event::{KeyboardInput, ModifiersState};

use super::CallBackKey;

pub type WidgetRef = Rc<RefCell<Widget>>;

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct Handle(pub(crate) usize);

impl Handle {
    pub fn get_key(&self) -> usize {
        self.0
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Identity {
    pub name: String,
    pub id: u64,
}

impl Identity {
    pub fn new(name: &str, id: u64) -> Self {
        Self {
            name: name.to_owned(),
            id,
        }
    }
}

bitfield! {
    pub u16 UiField
    UiFlags {
        0 : IsFocused,
        1 : CanFocus,
        2 : MouseOver,
        3 : MoveAble,
        4 : Moving,
        5 : CanClickBehind,
        6 : AlwaysUseable,
        7 : Minimized,
        8 : Checked,
        9 : FocusClick,
        10 : IsPassword,
        11 : CanMoveWindow,
        12 : Clicked,
        13 : ClickAble,
    }
}

pub trait Control {
    fn check_mouse_bounds(&self, mouse_pos: [i32; 2]) -> bool;
    fn get_mut_actions(&mut self) -> &mut UiField;
    fn get_bounds(&self) -> (i32, i32, i32, i32);
    fn get_size(&self) -> (i32, i32);
    fn set_position(&mut self, position: [i32; 2]);
}

pub trait AnyData: Control {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Control> AnyData for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// TODO: Make Bounds Updater that will Update all the internal Bounds based on
/// Parents Bounds if they got changed or if the childrens positions changed.
pub struct Widget {
    /// System Granted ID.
    pub id: Handle,
    /// Widgets Name and user given ID.
    pub identity: Identity,
    /// Used to Calculate and set the internal bounds of the widgets Data.
    pub bounds: Bounds,
    /// The UI holder for the Specific Widget.
    pub ui: Box<dyn AnyData>,
    ///If none then it is the Top most in the widget Tree.
    pub parent: Option<Handle>,
    ///The visible children in the Tree.
    pub visible: VecDeque<Handle>,
    ///The loaded but hidden children in the Tree.
    pub hidden: Vec<Handle>,
    /// Boolean Field of Actions Widgets can use.
    pub actions: UiField,
}

impl Widget {
    pub fn clear(&mut self) {
        self.parent = None;
        self.visible.clear();
        self.hidden.clear();
    }

    pub(crate) fn callback_key(&self, callback: CallBack) -> CallBackKey {
        CallBackKey::new(&self.identity, callback)
    }
}
