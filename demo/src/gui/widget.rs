use crate::{SystemEvent, UIBuffer, UI};
use graphics::*;
use hecs::Entity;
use input::FrameTime;
use std::any::Any;
use std::{cell::RefCell, collections::VecDeque, rc::Rc, vec::Vec};
use ubits::bitfield;
use wgpu::StencilFaceState;
use winit::event::{KeyboardInput, ModifiersState};

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct Handle(pub(crate) Entity);

impl Handle {
    pub fn get_key(&self) -> Entity {
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

pub trait Control<Message> {
    /// Widgets Name and user given ID All widgets must contain this.
    fn get_id(&self) -> &Identity;

    fn check_mouse_bounds(&self, mouse_pos: Vec2) -> bool;

    fn get_bounds(&self) -> Vec4;

    fn get_size(&self) -> Vec2;

    fn get_position(&mut self) -> Vec3;

    fn set_position(&mut self, position: Vec3);

    fn event(
        &mut self,
        actions: UiField,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        event: SystemEvent,
        events: &mut Vec<Message>,
    );

    fn draw(
        &mut self,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        frametime: &FrameTime,
    ) -> Result<(), AscendingError>;

    fn default_actions(&self) -> Vec<UiFlags>;
}

pub trait AnyData<Message>: Control<Message> {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl<Message, U: Any + Control<Message>> AnyData<Message> for U {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct Parent(pub Handle);

impl Parent {
    pub fn get_id(&self) -> Handle {
        self.0
    }
}

pub struct Actions(pub UiField);

impl Actions {
    pub fn get(&self) -> &UiField {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut UiField {
        &mut self.0
    }
}

pub struct Hidden(pub Vec<Handle>);

impl Hidden {
    pub fn get(&self) -> &Vec<Handle> {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut Vec<Handle> {
        &mut self.0
    }
}

pub struct Visible(pub VecDeque<Handle>);

impl Visible {
    pub fn get(&self) -> &VecDeque<Handle> {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut VecDeque<Handle> {
        &mut self.0
    }
}

pub struct WidgetAny<Message>(Box<dyn AnyData<Message>>);

impl<Message> WidgetAny<Message> {
    pub fn get(&self) -> &dyn AnyData<Message> {
        self.0.as_ref()
    }

    pub fn get_mut(&mut self) -> &mut dyn AnyData<Message> {
        self.0.as_mut()
    }
}

pub struct Widget;

pub struct WidgetBounds(pub Bounds);

impl WidgetBounds {
    pub fn get(&self) -> &Bounds {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut Bounds {
        &mut self.0
    }
}

/*
/// TODO: Make Bounds Updater that will Update all the internal Bounds based on
/// Parents Bounds if they got changed or if the childrens positions changed.
pub struct Widget<Message> {
    /// System Granted ID.
    pub id: Handle,
    /// Used to Calculate and set the internal bounds of the widgets Data.
    pub bounds: Bounds,
    /// The UI holder for the Specific Widget.
    pub ui: Box<dyn AnyData<Message>>,
    ///If none then it is the Top most in the widget Tree.
    pub parent: Option<Handle>,
    ///The visible children in the Tree.
    pub visible: VecDeque<Handle>,
    ///The loaded but hidden children in the Tree.
    pub hidden: Vec<Handle>,
    /// Boolean Field of Actions Widgets can use.
    pub actions: UiField,
}

impl<Message> Widget<Message> {
    pub fn new(control: (impl AnyData<Message> + 'static)) -> Self {
        Self {
            ui: Box::new(control),
            bounds: Bounds::default(),
            id: Handle(Entity::DANGLING),
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

    pub fn get_identity(&self) -> Identity {
        self.ui.get_id().clone()
    }
}

impl<Message> From<Widget<Message>> for Widget<Message> {
    fn from(widget: Widget<Message>) -> Self {
        Rc::new(RefCell::new(widget))
    }
}*/
