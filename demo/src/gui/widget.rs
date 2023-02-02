use crate::GuiRender;
use graphics::*;
use input::FrameTime;
use std::any::Any;
use std::marker::PhantomData;
use std::{cell::RefCell, collections::VecDeque, rc::Rc, vec::Vec};
use ubits::bitfield;
use winit::event::{KeyboardInput, ModifiersState};

pub type WidgetRef = Rc<RefCell<Widget>>;

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

pub trait UI {
    fn check_mouse_bounds(&self, mouse_pos: [i32; 2]) -> bool;
    fn get_mut_actions(&mut self) -> &mut UiField;
    fn get_name(&self) -> String;
    //This is an Id given by the end user to help identify it for events.
    fn get_id(&self) -> u64;
    fn get_bounds(&self) -> (i32, i32, i32, i32);
    fn set_position(&mut self, position: [i32; 2]);
}

pub struct Widget {
    /// The UI holder for the Specific Widget.
    pub ui: Box<dyn UI>,
    ///If none then it is the Top most in the widget Tree.
    pub parent: Option<WidgetRef>,
    ///The visible children in the Tree.
    pub children: VecDeque<WidgetRef>,
    ///The loaded but hidden children in the Tree.
    pub hidden: Vec<WidgetRef>,
    pub actions: UiField,
}
