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
    sync::RwLock,
    vec::Vec,
};
use winit::event::{KeyboardInput, ModifiersState, MouseButton};
use winit::window::Window;

pub mod events;
pub mod internals;

pub struct UI<Message: 'static> {
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

impl<Message: 'static> UI<Message> {
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

    pub fn set_action(world: &mut World, handle: Handle, action: UiFlags) {
        let mut actions = world
            .get::<&mut Actions>(handle.get_key())
            .expect("Widget is missing its actions?");
        actions.set(action);
    }

    pub fn remove_widget_by_handle(
        &mut self,
        world: &mut World,
        handle: Handle,
    ) {
        self.widget_clear_self(world, handle);
    }

    pub fn show_widget_by_handle(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        handle: Handle,
    ) -> Vec<Message> {
        let mut events: Vec<Message> = Vec::new();
        self.widget_show(world, ui_buffer, renderer, handle, &mut events);
        events
    }

    pub fn hide_widget_by_handle(&mut self, world: &mut World, handle: Handle) {
        self.widget_hide(world, handle);
    }

    pub(crate) fn create_widget(
        &mut self,
        world: &mut World,
        control: (impl AnyData<Message> + Send + Sync + 'static),
    ) -> Handle {
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

    pub fn add_widget(
        &mut self,
        world: &mut World,
        parent_handle: Option<Handle>,
        control: (impl AnyData<Message> + Send + Sync + 'static),
    ) -> Handle {
        let handle = self.create_widget(world, control);

        if let Some(parent_handle) = parent_handle {
            let _ = world.insert_one(handle.get_key(), Parent(parent_handle));
            self.widget_show_children(world, parent_handle);
        } else {
            self.zlist.push_back(handle);
            self.widget_show_children(world, handle)
        }

        handle
    }

    pub fn add_widget_hidden(
        &mut self,
        world: &mut World,
        parent_handle: Option<Handle>,
        control: (impl AnyData<Message> + Send + Sync + 'static),
    ) -> Handle {
        let handle = self.create_widget(world, control);

        if let Some(parent_handle) = parent_handle {
            let _ = world.insert_one(handle.get_key(), Parent(parent_handle));
        }

        let _ = world.insert_one(handle.get_key(), Hidden);
        handle
    }

    pub fn clear_widgets(&mut self, world: &mut World) {
        self.zlist.clear();
        self.name_map.clear();
        self.focused = None;
        self.over = None;
        self.clicked = None;

        let widgets: Vec<Handle> = world
            .query::<&Widget>()
            .iter()
            .map(|(entity, _)| Handle(entity))
            .collect();

        for handle in widgets {
            let _ = world.despawn(handle.get_key());
        }
    }
}
