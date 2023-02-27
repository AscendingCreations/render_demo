use crate::{
    CallBack, CallBackKey, CallBacks, FrameTime, Handle, Identity,
    InternalCallBacks, UIBuffer, UiFlags, Widget, WidgetRef,
};
use graphics::*;
use slab::Slab;
use std::{
    any::Any,
    cell::RefCell,
    collections::{HashMap, VecDeque},
    marker::PhantomData,
    rc::Rc,
    vec::Vec,
};
use winit::event::{KeyboardInput, ModifiersState};
use winit::window::Window;

pub mod events;
pub mod internals;

pub struct UI<T> {
    renderer: UIBuffer,
    /// Callback mapper. Hashes must be different.
    callbacks: HashMap<CallBackKey, Rc<InternalCallBacks<T>>>,
    user_callbacks: HashMap<CallBackKey, Rc<CallBacks<T>>>,
    name_map: HashMap<Identity, Handle>,
    widgets: Slab<WidgetRef>,
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
    mouse_clicked: [i32; 2],
    mouse_pos: [i32; 2],
    new_mouse_pos: [i32; 2],
    moving: bool,
    button: u32,
    modifier: ModifiersState,
}

impl<T> UI<T> {
    pub fn new(renderer: UIBuffer) -> Self {
        UI {
            renderer,
            callbacks: HashMap::with_capacity(100),
            user_callbacks: HashMap::with_capacity(100),
            name_map: HashMap::with_capacity(100),
            widgets: Slab::with_capacity(100),
            zlist: VecDeque::with_capacity(100),
            visible: VecDeque::with_capacity(100),
            hidden: Vec::with_capacity(100),
            focused: Option::None,
            over: Option::None,
            clicked: Option::None,
            widget_moving: Option::None,
            mouse_clicked: [0; 2],
            mouse_pos: [0; 2],
            new_mouse_pos: [0; 2],
            moving: false,
            button: 0,
            modifier: ModifiersState::default(),
        }
    }

    pub fn remove_widget_by_handle(&mut self, handle: Handle) {
        self.widget_clear_self(&self.get_widget(handle));
    }

    pub fn remove_widget_by_id(&mut self, id: Identity) {
        let handle = self.name_map.get(&id).unwrap();
        self.widget_clear_self(&self.get_widget(*handle));
    }

    pub fn show_widget_by_handle(&mut self, handle: Handle) {
        self.widget_show(&self.get_widget(handle));
    }

    pub fn show_widget_by_id(&mut self, id: Identity) {
        let handle = self.name_map.get(&id).unwrap();
        self.widget_show(&self.get_widget(*handle));
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
        control: WidgetRef,
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
        control: WidgetRef,
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
        control: WidgetRef,
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
        control: WidgetRef,
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
        self.callbacks.clear();
        self.user_callbacks.clear();
        self.name_map.clear();
        self.widgets.clear();
        self.focused = None;
        self.over = None;
        self.clicked = None;
    }
}
