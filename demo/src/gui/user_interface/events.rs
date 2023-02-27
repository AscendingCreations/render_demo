use crate::{
    CallBack, CallBackKey, CallBacks, FrameTime, Handle, Identity,
    InternalCallBacks, UIBuffer, UiFlags, Widget, WidgetRef, UI,
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

impl<T> UI<T> {
    pub fn event_render(&mut self, time: FrameTime, user_data: &mut T) {
        for handle in &self.zlist.clone() {
            let widget = self.get_widget(*handle);

            let key = widget.borrow().callback_key(CallBack::Draw);
            let mut mut_wdgt = widget.borrow_mut();

            if let Some(callback) = self.get_inner_callback(&key) {
                if let InternalCallBacks::Draw(draw) = callback.as_ref() {
                    draw(&mut mut_wdgt, self, &time);
                }
            }

            if let Some(callback) = self.get_user_callback(&key) {
                if let CallBacks::Draw(draw) = callback.as_ref() {
                    draw(&mut mut_wdgt, self, &time, user_data);
                }
            }
        }
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
                    self.widget_position_update(&mut focused.borrow_mut());
                }
            }

            self.mouse_over_event(user_data);
        }

        self.mouse_pos = position;
    }

    pub fn event_mouse_button(
        &mut self,
        button: u32,
        pressed: bool,
        user_data: &mut T,
    ) {
        self.button = button;
        self.mouse_clicked = self.mouse_pos;

        if pressed {
            self.mouse_press(user_data);
        } else {
            self.mouse_release(user_data);
        }
    }

    pub fn event_modifiers(&mut self, modifier: ModifiersState) {
        self.modifier = modifier;
    }
}
