use crate::{
    Actions, AnyData, Event, FrameTime, GpuDevice, Handle, Identity, UIBuffer,
    UiFlags, Widget,
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
    ///The Visible Top widgets.
    visible: VecDeque<Handle>,
    ///The loaded but hidden Top children.
    hidden: Vec<Handle>,
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
            visible: VecDeque::with_capacity(100),
            hidden: Vec::with_capacity(100),
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

    pub fn remove_widget_by_handle(&mut self, handle: Handle) {
        self.widget_clear_self(&self.get_widget(handle));
    }

    pub fn remove_widget_by_id(&mut self, id: Identity) {
        let handle = self.name_map.get(&id).unwrap();
        self.widget_clear_self(&self.get_widget(*handle));
    }

    pub fn show_widget_by_handle(
        &mut self,
        renderer: &mut GpuRenderer,
        handle: Handle,
    ) {
        self.widget_show(renderer, &self.get_widget(handle));
    }

    pub fn show_widget_by_id(
        &mut self,
        renderer: &mut GpuRenderer,
        id: Identity,
    ) {
        let handle = self.name_map.get(&id).unwrap();
        self.widget_show(renderer, &self.get_widget(*handle));
    }

    pub fn hide_widget_by_handle(&mut self, handle: Handle) {
        self.widget_hide(&self.get_widget(handle));
    }

    pub fn hide_widget_by_id(&mut self, id: Identity) {
        let handle = self.name_map.get(&id).unwrap();
        self.widget_hide(&self.get_widget(*handle));
    }

    pub fn add_widget<T>(&mut self, parent_handle: Option<Handle>, control: T)
    where
        T: AnyData<Message>,
    {
        if let Some(handle) = parent_handle {
            self.widget_add(Some(&self.get_widget(handle)), control);
        } else {
            self.widget_add(None, control);
        }
    }

    pub fn add_hidden_widget<T>(
        &mut self,
        parent_handle: Option<Handle>,
        control: T,
    ) where
        T: AnyData<Message>,
    {
        if let Some(handle) = parent_handle {
            self.widget_add_hidden(Some(&self.get_widget(handle)), control);
        } else {
            self.widget_add_hidden(None, control);
        }
    }

    pub fn clear_widgets(&mut self) {
        self.visible.clear();
        self.zlist.clear();
        self.hidden.clear();
        self.name_map.clear();
        self.widgets.clear();
        self.focused = None;
        self.over = None;
        self.clicked = None;
    }
}
