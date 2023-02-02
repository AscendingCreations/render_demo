use crate::{
    CallBacks, GuiRender, InternalCallBacks, UiFlags, Widget, WidgetRef,
};
use graphics::*;
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
    callbacks: HashMap<String, InternalCallBacks>,
    user_callbacks: HashMap<String, CallBacks<T>>,
    ///Contains All Visible widgets in rendering order
    zlist: VecDeque<WidgetRef>,
    ///The Visible Top children.
    children: VecDeque<WidgetRef>,
    ///The loaded but hidden Top children.
    hidden: Vec<WidgetRef>,
    focused: Option<WidgetRef>,
    over: Option<WidgetRef>,
    clicked: Option<WidgetRef>,
    mouse_clicked: [i32; 2],
    mouse_pos: [i32; 2],
    new_mouse_pos: [i32; 2],
    screensize: [i32; 2],
    moving: bool,
    button: i32,
}

impl<T> Widgets<T> {
    pub fn new(screensize: [i32; 2]) -> Self {
        let mut widgets = Widgets {
            callbacks: HashMap::new(),
            user_callbacks: HashMap::new(),
            zlist: VecDeque::new(),
            children: VecDeque::new(),
            hidden: Vec::new(),
            focused: Option::None,
            over: Option::None,
            clicked: Option::None,
            mouse_clicked: [0; 2],
            mouse_pos: [0; 2],
            new_mouse_pos: [0; 2],
            screensize,
            moving: false,
            button: 0,
        };

        widgets.screensize = screensize;
        widgets
    }

    pub fn event_mouse_position(
        &mut self,
        window: &mut Window,
        position: [i32; 2],
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
            if let Some(focused) = self.focused.clone() {
                if focused.borrow().actions.get(UiFlags::Moving) {
                    let pos = [
                        position[0] - self.mouse_pos[0],
                        position[1] - self.mouse_pos[1],
                    ];
                    let widget = focused.borrow();
                    let mut bounds = widget.ui.get_bounds();

                    if bounds.0 + pos[0] <= 0
                        || bounds.1 + pos[1] <= 0
                        || bounds.0 + bounds.2 + pos[0] >= self.screensize[0]
                        || bounds.1 + bounds.3 + pos[1] >= self.screensize[1]
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

    pub fn clear_hidden(&mut self) {
        self.hidden.clear();
    }

    pub fn clear_visible(&mut self) {
        self.children.clear();
    }

    fn widget_position_update(
        &mut self,
        parent: &mut Widget,
        user_data: &mut T,
    ) {
        let key = format!(
            "{}_{}_pos_update",
            &parent.ui.get_name(),
            parent.ui.get_id(),
        );

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

        for child in &parent.children {
            if !child.borrow().children.is_empty() {
                self.widget_position_update(&mut child.borrow_mut(), user_data);
            } else {
                let key = format!(
                    "{}_{}_pos_update",
                    &child.borrow().ui.get_name(),
                    child.borrow().ui.get_id(),
                );

                if let Some(InternalCallBacks::PositionChange(
                    internal_update_pos,
                )) = self.callbacks.get(&key)
                {
                    internal_update_pos(&mut child.borrow_mut());
                }

                if let Some(CallBacks::PositionChange(user_update_pos)) =
                    self.user_callbacks.get(&key)
                {
                    user_update_pos(&mut child.borrow_mut(), user_data);
                }
            }
        }
    }
}
