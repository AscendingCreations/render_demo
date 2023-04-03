use crate::{
    Event, FrameTime, GpuDevice, Handle, Identity, UIBuffer, UiFlags,
    UserInterface, Widget, WidgetRef,
};
use graphics::*;
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

pub struct UI<T, Message: Clone> {
    ui_buffer: UIBuffer,
    name_map: HashMap<Identity, Handle>,
    widgets: Slab<WidgetRef<T, Message>>,
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
}

impl<T, Message: Clone> UI<T, Message> {
    pub fn new(ui_buffer: UIBuffer) -> Self {
        UI {
            ui_buffer,
            name_map: HashMap::with_capacity(100),
            widgets: Slab::with_capacity(100),
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
        }
    }

    pub fn ui_buffer(&self) -> &UIBuffer {
        &self.ui_buffer
    }

    pub fn ui_buffer_mut(&mut self) -> &mut UIBuffer {
        &mut self.ui_buffer
    }

    pub fn get_widget(&self, handle: Handle) -> WidgetRef<T, Message> {
        self.widgets
            .get(handle.get_key())
            .expect("ID Existed but widget does not exist?")
            .clone()
    }

    pub fn get_widget_by_id(&self, id: Identity) -> WidgetRef<T, Message> {
        let handle = self.name_map.get(&id).unwrap();
        self.widgets
            .get(handle.get_key())
            .expect("ID Existed but widget does not exist?")
            .clone()
    }

    pub fn set_action(widget: &WidgetRef<T, Message>, action: UiFlags) {
        widget.borrow_mut().actions.set(action);
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

    pub fn add_widget_by_handle(
        &mut self,
        parent_handle: Option<Handle>,
        control: WidgetRef<T, Message>,
    ) {
        if let Some(handle) = parent_handle {
            self.widget_add(Some(&self.get_widget(handle)), control);
        } else {
            self.widget_add(None, control);
        }
    }

    pub fn add_widget_by_id(
        &mut self,
        parent_id: Option<Identity>,
        control: WidgetRef<T, Message>,
    ) {
        if let Some(id) = parent_id {
            let handle = self.name_map.get(&id).unwrap();
            self.widget_add(Some(&self.get_widget(*handle)), control);
        } else {
            self.widget_add(None, control);
        }
    }

    pub fn add_hidden_widget_by_handle(
        &mut self,
        parent_handle: Option<Handle>,
        control: WidgetRef<T, Message>,
    ) {
        if let Some(handle) = parent_handle {
            self.widget_add_hidden(Some(&self.get_widget(handle)), control);
        } else {
            self.widget_add_hidden(None, control);
        }
    }

    pub fn add_hidden_widget_by_id(
        &mut self,
        parent_id: Option<Identity>,
        control: WidgetRef<T, Message>,
    ) {
        if let Some(id) = parent_id {
            let handle = self.name_map.get(&id).unwrap();
            self.widget_add_hidden(Some(&self.get_widget(*handle)), control);
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
