use crate::{
    CallBack, CallBackKey, CallBacks, FrameTime, GpuRenderer, Handle, Identity,
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
use winit::{
    dpi::PhysicalPosition,
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, ModifiersState,
        MouseButton, MouseScrollDelta, WindowEvent,
    },
    window::Window,
};

impl<T> UI<T> {
    pub fn event_draw(
        &mut self,
        renderer: &mut GpuRenderer,
        time: &FrameTime,
        user_data: &mut T,
    ) {
        for handle in &self.zlist.clone() {
            let widget = self.get_widget(*handle);

            let key = widget.borrow().callback_key(CallBack::Draw);
            let mut mut_wdgt = widget.borrow_mut();

            if let Some(callback) = self.get_inner_callback(&key) {
                if let InternalCallBacks::Draw(draw) = callback {
                    draw(&mut mut_wdgt, self, renderer, time);
                }
            }

            if let Some(callback) = self.get_user_callback(&key) {
                if let CallBacks::Draw(draw) = callback.as_ref() {
                    draw(&mut mut_wdgt, self, renderer, time, user_data);
                }
            }
        }

        self.ui_buffer_mut().ui_buffer.finalize(renderer);
        self.ui_buffer_mut().text_renderer.finalize(renderer);
    }

    pub fn event_mouse_position(
        &mut self,
        renderer: &mut GpuRenderer,
        position: Vec2,
        screensize: Vec2,
        user_data: &mut T,
    ) {
        self.new_mouse_pos = position;

        if self.moving {
            if let Ok(win_pos) = renderer.window().outer_position() {
                let mut win_pos = Vec2::new(win_pos.x as f32, win_pos.y as f32);
                win_pos.x = position[0] + win_pos.x - self.mouse_clicked[0];
                win_pos.y = position[1] + win_pos.y - self.mouse_clicked[1];
                renderer.window_mut().set_outer_position(
                    PhysicalPosition::new(win_pos.x, win_pos.y),
                );
            } else {
                panic!("Not Supported. This will be a Soft warning via log later on.")
            }
        } else {
            if let Some(handle) = self.focused {
                let focused = self.get_widget(handle);

                if focused.borrow().actions.get(UiFlags::Moving) {
                    let pos = [
                        position.x - self.mouse_pos[0],
                        position.y - self.mouse_pos[1],
                    ];
                    let mut bounds = focused.borrow().ui.get_bounds();

                    if bounds.x + pos[0] <= 0.0
                        || bounds.y + pos[1] <= 0.0
                        || bounds.x + bounds.z + pos[0] >= screensize[0]
                        || bounds.y + bounds.w + pos[1] >= screensize[1]
                    {
                        return;
                    }

                    bounds.x += pos[0];
                    bounds.y += pos[1];
                    let control_pos = focused.borrow_mut().ui.get_position();
                    focused.borrow_mut().ui.set_position(Vec3::new(
                        bounds.x,
                        bounds.y,
                        control_pos.z,
                    ));
                    self.widget_position_update(
                        renderer,
                        &mut focused.borrow_mut(),
                    );
                }
            }

            self.mouse_over_event(renderer, user_data);
        }

        self.mouse_pos = position;
    }

    pub fn event_mouse_button(
        &mut self,
        renderer: &mut GpuRenderer,
        button: MouseButton,
        pressed: bool,
        user_data: &mut T,
    ) {
        self.button = button;
        self.mouse_clicked = self.mouse_pos;

        if pressed {
            self.mouse_press(renderer, user_data);
        } else {
            self.mouse_release(renderer, user_data);
        }
    }

    pub fn event_modifiers(&mut self, modifier: ModifiersState) {
        self.modifier = modifier;
    }

    pub fn handle_events(
        &mut self,
        renderer: &mut GpuRenderer,
        event: &Event<()>,
        hidpi: f32,
        user_data: &mut T,
    ) {
        match *event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == renderer.window().id() => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: _,
                            virtual_keycode: Some(_key_code),
                            scancode: _,
                            ..
                        },
                    ..
                } => {}
                WindowEvent::MouseInput { state, button, .. } => {
                    let pressed = *state == ElementState::Pressed;
                    self.event_mouse_button(
                        renderer, *button, pressed, user_data,
                    );
                }
                WindowEvent::CursorMoved {
                    position: PhysicalPosition { x, y },
                    ..
                } => {
                    let size = renderer.size();
                    let pos = Vec2::new(
                        (*x as f32) * hidpi,
                        size.height - ((*y as f32) * hidpi),
                    );
                    self.event_mouse_position(
                        renderer,
                        pos,
                        Vec2::new(size.width, size.height),
                        user_data,
                    );
                }
                _ => (),
            },
            Event::DeviceEvent { ref event, .. } => match *event {
                DeviceEvent::MouseMotion { delta: _ } => {}
                DeviceEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(_dx, _dy),
                } => {
                    /* if dx != 0.0 {
                        self.mouse_wheel.0 = dx.signum();
                    }

                    if dy != 0.0 {
                        self.mouse_wheel.1 = dy.signum();
                    }*/
                }
                DeviceEvent::MouseWheel {
                    delta:
                        MouseScrollDelta::PixelDelta(PhysicalPosition {
                            x: _,
                            y: _,
                        }),
                } => {
                    /*if x != 0.0 {
                        self.mouse_wheel.0 = x.signum() as f32;
                    }

                    if y != 0.0 {
                        self.mouse_wheel.1 = y.signum() as f32;
                    }*/
                }
                _ => (),
            },
            _ => (),
        }
    }
}
