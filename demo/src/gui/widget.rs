use crate::{CallBack, InternalCallBacks, UIBuffer};
use graphics::*;
use input::FrameTime;
use std::any::Any;
use std::{cell::RefCell, collections::VecDeque, rc::Rc, vec::Vec};
use ubits::bitfield;
use wgpu::StencilFaceState;
use winit::event::{KeyboardInput, ModifiersState};

use super::CallBackKey;

pub type WidgetRef<T> = Rc<RefCell<Widget<T>>>;

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

pub trait Control<T> {
    fn check_mouse_bounds(&self, mouse_pos: Vec2) -> bool;
    fn get_bounds(&self) -> Vec4;
    fn get_size(&self) -> Vec2;
    fn get_position(&mut self) -> Vec3;
    fn set_position(&mut self, position: Vec3);

    fn into_widget(self, id: Identity) -> WidgetRef<T>
    where
        Self: std::marker::Sized + 'static,
    {
        let actions = self.default_actions();
        let mut widget = Widget::new(id, self);

        for action in actions {
            widget.actions.set(action);
        }

        widget.into()
    }

    fn get_internal_callbacks(
        &self,
        id: &Identity,
    ) -> Vec<(InternalCallBacks<T>, CallBackKey)>;
    fn default_actions(&self) -> Vec<UiFlags>;
}

pub trait AnyData<T>: Control<T> {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl<T, U: Any + Control<T>> AnyData<T> for U {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

/// TODO: Make Bounds Updater that will Update all the internal Bounds based on
/// Parents Bounds if they got changed or if the childrens positions changed.
pub struct Widget<T> {
    /// System Granted ID.
    pub id: Handle,
    /// Widgets Name and user given ID.
    pub identity: Identity,
    /// Used to Calculate and set the internal bounds of the widgets Data.
    pub bounds: Bounds,
    /// The UI holder for the Specific Widget.
    pub ui: Box<dyn AnyData<T>>,
    ///If none then it is the Top most in the widget Tree.
    pub parent: Option<Handle>,
    ///The visible children in the Tree.
    pub visible: VecDeque<Handle>,
    ///The loaded but hidden children in the Tree.
    pub hidden: Vec<Handle>,
    /// Boolean Field of Actions Widgets can use.
    pub actions: UiField,
}

impl<T> Widget<T> {
    pub fn new(
        identity: Identity,
        control: (impl AnyData<T> + 'static),
    ) -> Self {
        Self {
            identity,
            ui: Box::new(control),
            bounds: Bounds::default(),
            id: Handle(0),
            parent: None,
            visible: VecDeque::new(),
            hidden: Vec::new(),
            actions: UiField::new(0),
        }
    }

    pub fn clear(&mut self) {
        self.parent = None;
        self.visible.clear();
        self.hidden.clear();
    }

    pub fn callback_key(&self, callback: CallBack) -> CallBackKey {
        CallBackKey::new(&self.identity, callback)
    }

    pub fn get_identity(&self) -> Identity {
        self.identity.clone()
    }
}

impl<T> From<Widget<T>> for WidgetRef<T> {
    fn from(widget: Widget<T>) -> Self {
        Rc::new(RefCell::new(widget))
    }
}
