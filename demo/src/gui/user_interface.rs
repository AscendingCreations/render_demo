use crate::{
    Actions, AnyData, Event, FrameTime, GpuDevice, Handle, Hidden, Identity,
    Parent, UIBuffer, UiFlags, Widget, WidgetAny, WidgetBounds,
};
use graphics::*;
use hecs::{Entity, World};
use slab::Slab;
use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, VecDeque},
    marker::PhantomData,
    rc::Rc,
    vec::Vec,
};
use winit::event::{KeyboardInput, ModifiersState, MouseButton};
use winit::window::Window;

pub mod events;
pub mod internals;

pub struct UI<Message> {
    name_map: HashMap<Identity, Handle>,
    ///Contains All Visible widgets in rendering order
    zlist: VecDeque<Handle>,
    focused: Option<Handle>,
    over: Option<Handle>,
    clicked: Option<Handle>,
    widget_moving: Option<Handle>,
    ///Saved States.
    mouse_clicked: Vec2,
    mouse_pos: Vec2,
    new_mouse_pos: Vec2,
    moving: bool,
    button: MouseButton,
    modifier: ModifiersState,
    phantom: PhantomData<Message>,
}

impl<Message> UI<Message> {
    pub fn new() -> Self {
        UI {
            name_map: HashMap::with_capacity(100),
            zlist: VecDeque::with_capacity(100),
            focused: None,
            over: None,
            clicked: None,
            widget_moving: None,
            mouse_clicked: Vec2::default(),
            mouse_pos: Vec2::default(),
            new_mouse_pos: Vec2::default(),
            moving: false,
            button: MouseButton::Left,
            modifier: ModifiersState::default(),
            phantom: PhantomData::default(),
        }
    }

    /*pub fn get_widget_by_id(&self, world: &mut World, id: Identity) -> &Widget<Message> {
        let handle = self.name_map.get(&id).unwrap();
        self.widgets
            .get(handle.get_key())
            .expect("ID Existed but widget does not exist?")
    }*/

    pub fn set_action(world: &mut World, id: Handle, action: UiFlags) {
        let actions: Actions = world.get(id.get_key());
        actions.get_mut().set(action);
    }

    pub fn remove_widget_by_handle(
        &mut self,
        world: &mut World,
        handle: Handle,
    ) {
        self.widget_clear_self(&self.get_widget(handle));
    }

    pub fn show_widget_by_handle(
        &mut self,
        renderer: &mut GpuRenderer,
        handle: Handle,
    ) {
        self.widget_show(renderer, &self.get_widget(handle));
    }

    pub fn hide_widget_by_handle(&mut self, handle: Handle) {
        self.widget_hide(&self.get_widget(handle));
    }

    pub(crate) fn create_widget<T>(
        &mut self,
        world: &mut World,
        control: T,
    ) -> Handle
    where
        T: AnyData<Message>,
    {
        let actions = Actions(control.default_actions());
        let bounds = WidgetBounds(Bounds::default());
        let identity = control.get_id().clone();

        if self.name_map.contains_key(&identity) {
            panic!("You can not use the same Identity for multiple widgets");
        }

        let ui = WidgetAny(Box::new(control));
        let handle = Handle(world.spawn((Widget, actions, bounds, ui)));

        self.name_map.insert(identity, handle);
        handle
    }

    pub fn add_widget<T>(
        &mut self,
        world: &mut World,
        parent_handle: Option<Handle>,
        control: T,
    ) -> Handle
    where
        T: AnyData<Message>,
    {
        let handle = self.create_widget(world, control);

        if let Some(parent_handle) = parent_handle {
            world.insert_one(handle.get_key(), Parent(parent_handle));
        }

        self.widget_add(world, parent_handle, handle);
        handle
    }

    pub fn add_widget_hidden<T>(
        &mut self,
        world: &mut World,
        parent_handle: Option<Handle>,
        control: T,
    ) -> Handle
    where
        T: AnyData<Message>,
    {
        let handle = self.create_widget(world, control);

        if let Some(parent_handle) = parent_handle {
            world.insert_one(handle.get_key(), Parent(parent_handle));
        }

        world.insert_one(handle.get_key(), Hidden);
        handle
    }

    pub fn clear_widgets(&mut self, world: &mut World) {
        self.visible.clear();
        self.zlist.clear();
        self.hidden.clear();
        self.name_map.clear();
        self.focused = None;
        self.over = None;
        self.clicked = None;
    }
}
