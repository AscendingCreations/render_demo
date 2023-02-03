use crate::{
    CallBack, CallBackKey, CallBacks, GuiRender, Handle, Identity,
    InternalCallBacks, UiFlags, Widget, WidgetRef,
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

#[derive(Default)]
pub struct Widgets<T> {
    /// Callback mapper. Hashes must be different.
    callbacks: HashMap<CallBackKey, InternalCallBacks>,
    user_callbacks: HashMap<CallBackKey, CallBacks<T>>,
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
    mouse_clicked: [i32; 2],
    mouse_pos: [i32; 2],
    new_mouse_pos: [i32; 2],
    moving: bool,
    button: i32,
}

impl<T> Widgets<T> {
    pub fn new() -> Self {
        Widgets {
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
            mouse_clicked: [0; 2],
            mouse_pos: [0; 2],
            new_mouse_pos: [0; 2],
            moving: false,
            button: 0,
        }
    }

    fn get_widget(&self, handle: Handle) -> WidgetRef {
        self.widgets
            .get(handle.get_key())
            .expect("ID Existed but widget does not exist?")
            .clone()
    }

    pub fn event_mouse_position(
        &mut self,
        window: &mut Window,
        position: [i32; 2],
        screensize: [i32; 2],
        user_data: &mut T,
    ) {
        self.new_mouse_pos = position;

        if self.moving {
            if let Ok(mut win_pos) = window.outer_position() {
                win_pos.x = position[0] + win_pos.x - self.mouse_clicked[0];
                win_pos.y = position[1] + win_pos.y - self.mouse_clicked[1];
                window.set_outer_position(win_pos);
            } else {
                panic!("Not Supported. This will be a Soft warning via log later on.")
            }
        } else {
            if let Some(handle) = self.focused {
                let focused = self.get_widget(handle);

                if focused.borrow().actions.get(UiFlags::Moving) {
                    let pos = [
                        position[0] - self.mouse_pos[0],
                        position[1] - self.mouse_pos[1],
                    ];
                    let mut bounds = focused.borrow().ui.get_bounds();

                    if bounds.0 + pos[0] <= 0
                        || bounds.1 + pos[1] <= 0
                        || bounds.0 + bounds.2 + pos[0] >= screensize[0]
                        || bounds.1 + bounds.3 + pos[1] >= screensize[1]
                    {
                        return;
                    }

                    bounds.0 += pos[0];
                    bounds.1 += pos[1];

                    focused.borrow_mut().ui.set_position([bounds.0, bounds.1]);
                    self.widget_position_update(
                        &mut focused.borrow_mut(),
                        user_data,
                    );
                }
            }

            //TODO handle Widget mouse over here.
        }

        self.mouse_pos = position;
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

    fn widget_position_update(
        &mut self,
        parent: &mut Widget,
        user_data: &mut T,
    ) {
        let key = parent.callback_key(CallBack::PositionChange);

        if let Some(InternalCallBacks::PositionChange(internal_update_pos)) =
            self.callbacks.get(&key)
        {
            internal_update_pos(parent);
        }

        if let Some(CallBacks::PositionChange(user_update_pos)) =
            self.user_callbacks.get(&key)
        {
            user_update_pos(parent, user_data);
        }

        for handle in &parent.children {
            let widget = self.get_widget(*handle);

            if !widget.borrow().children.is_empty() {
                self.widget_position_update(
                    &mut widget.borrow_mut(),
                    user_data,
                );
            } else {
                let key =
                    widget.borrow().callback_key(CallBack::PositionChange);
                let mut mut_wdgt = widget.borrow_mut();

                if let Some(InternalCallBacks::PositionChange(
                    internal_update_pos,
                )) = self.callbacks.get(&key)
                {
                    internal_update_pos(&mut mut_wdgt);
                }

                if let Some(CallBacks::PositionChange(user_update_pos)) =
                    self.user_callbacks.get(&key)
                {
                    user_update_pos(&mut mut_wdgt, user_data);
                }
            }
        }
    }
}
