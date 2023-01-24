use crate::GuiRender;
use graphics::*;
use input::FrameTime;
use std::{cell::RefCell, collections::VecDeque, rc::Rc, vec::Vec};
use ubits::bitfield;

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
    fn draw(&mut self, frame_time: FrameTime, renders: &mut GuiRender);
}

pub struct Widget {
    pub name: String,
    pub data: Option<Box<dyn UI>>,
    pub parent: Option<Rc<RefCell<Widget>>>,
    pub children: VecDeque<Rc<RefCell<Widget>>>,
    pub hidden: Vec<Rc<RefCell<Widget>>>,
}
