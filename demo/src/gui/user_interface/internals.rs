use crate::{
    Actions, FrameTime, Handle, Hidden, Identity, Parent, SystemEvent,
    UIBuffer, UiFlags, Widget, WidgetAny, UI,
};
use graphics::*;
use hecs::{With, Without, World};
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

impl<Message> UI<Message> {
    pub(crate) fn mouse_over_event(
        &mut self,
        renderer: &mut GpuRenderer,
        events: &mut Vec<Message>,
    ) {
        for &handle in self.zlist.clone().iter().rev() {
            let control = self.get_widget(handle);

            if control.ui.check_mouse_bounds(self.mouse_pos)
                && self.widget_usable(&control)
            {
                if control.actions.get(UiFlags::CanClickBehind) {
                    if let Some(parent_handle) = control.parent {
                        let parent = self.get_widget(parent_handle);

                        if !parent.actions.get(UiFlags::Moving) {
                            self.widget_mouse_over(
                                renderer, &parent, true, events,
                            );
                        }

                        return;
                    }
                } else {
                    self.widget_mouse_over(renderer, &control, true, events);
                    return;
                }
            } else if !control.actions.get(UiFlags::Moving) {
                self.widget_mouse_over(renderer, &control, false, events);
            }
        }
    }

    pub(crate) fn widget_mouse_over_callback(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        control: Handle,
        entered: bool,
        events: &mut Vec<Message>,
    ) {
        let mut control = control;
        let actions = control.actions;

        control.ui.event(
            actions,
            self.ui_buffer_mut(),
            renderer,
            SystemEvent::MousePresent(entered),
            events,
        );
    }

    pub(crate) fn widget_mouse_over(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        control: Handle,
        entered: bool,
        events: &mut Vec<Message>,
    ) {
        if let Some(over) = self.over {
            let mut actions = world
                .get::<&mut Actions>(over.get_key())
                .expect("Widget is missing its actions?");

            if entered {
                if over != control && self.widget_moving.is_none() {
                    actions.clear(UiFlags::MouseOver);

                    world
                        .get::<&mut Actions>(control.get_key())
                        .expect("Widget is missing its actions?")
                        .set(UiFlags::MouseOver);

                    self.widget_mouse_over_callback(
                        world, ui_buffer, renderer, over, false, events,
                    );
                    self.over = Some(control);
                    self.widget_mouse_over_callback(
                        world, ui_buffer, renderer, control, true, events,
                    );
                }
            } else {
                let mut ui = world
                    .get::<&mut WidgetAny<Message>>(over.get_key())
                    .expect("Widget is missing its inner UI Type?");

                if !ui.check_mouse_bounds(self.mouse_pos)
                    && actions.exists(UiFlags::MouseOver)
                    && self.widget_moving.is_none()
                {
                    actions.clear(UiFlags::MouseOver);
                    self.widget_mouse_over_callback(
                        world, ui_buffer, renderer, over, false, events,
                    );
                    self.over = None;
                }
            }
        } else if entered {
            self.over = Some(control);
            world
                .get::<&mut Actions>(control.get_key())
                .expect("Widget is missing its actions?")
                .set(UiFlags::MouseOver);
            self.widget_mouse_over_callback(
                world, ui_buffer, renderer, control, true, events,
            );
        }
    }

    pub(crate) fn widget_usable(
        &self,
        world: &mut World,
        control: Handle,
    ) -> bool {
        let actions: Actions =
            world.get(control).expect("Widget is missing its actions?");

        if actions.exists(UiFlags::AlwaysUseable) {
            return true;
        }

        if !actions.exists(UiFlags::IsFocused) {
            let mut parent_handle = match world.get::<Parent>(control.get_key())
            {
                Ok(parent) => Some(parent),
                Err(_) => return false,
            };

            while let Some(parent) = parent_handle {
                let parent_actions: Actions = world
                    .get(parent.get_key())
                    .expect("Widget is missing its actions?");

                if (parent_actions.exists(UiFlags::CanFocus)
                    && parent_actions.exists(UiFlags::IsFocused))
                    || parent_actions.exists(UiFlags::AlwaysUseable)
                {
                    return true;
                }

                parent_handle = match world.get::<Parent>(parent.get_key()) {
                    Ok(parent) => Some(parent),
                    Err(_) => return false,
                };
            }
        } else {
            true
        }
    }

    pub(crate) fn widget_manual_focus(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        control: Handle,
        events: &mut Vec<Message>,
    ) {
        let actions: Actions =
            world.get(control).expect("Widget is missing its actions?");

        if actions.exists(UiFlags::CanFocus) {
            if let Some(pos) = self.zlist.iter().position(|x| *x == control) {
                self.zlist.remove(pos);
                self.zlist.push_back(control);
            }

            self.widget_show_children(world, control);

            if let Some(focused_handle) = self.focused {
                self.widget_focused_callback(
                    world,
                    ui_buffer,
                    renderer,
                    focused_handle,
                    events,
                    false,
                );
            }

            actions.set(UiFlags::IsFocused);
            self.focused = Some(control);
            self.widget_focused_callback(
                world, ui_buffer, renderer, control, events, true,
            );
        }
    }

    pub(crate) fn widget_show(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        control: Handle,
        events: &mut Vec<Message>,
    ) {
        if let Some(pos) = self.zlist.iter().position(|x| *x == control) {
            self.zlist.remove(pos);
        }

        self.zlist.push_back(control);
        self.widget_show_children(world, control);

        if !self.widget_is_focused(world, control) && self.focused.is_some() {
            let focused = self.focused.unwrap();

            self.widget_manual_focus(
                world, ui_buffer, renderer, focused, events,
            );
        }
    }

    pub(crate) fn widget_hide(&mut self, world: &mut World, control: Handle) {
        if let Some(pos) = self.zlist.iter().position(|x| *x == control) {
            self.zlist.remove(pos);
        }

        self.widget_hide_children(world, control);

        if self.focused == Some(control) {
            self.focused = None;
        }
    }

    pub(crate) fn widget_clear_self(
        &mut self,
        world: &mut World,
        control: Handle,
    ) {
        if self.focused == Some(control) {
            self.focused = None;
        }

        if self.clicked == Some(control) {
            self.clicked = None;
        }

        if self.over == Some(control) {
            self.over = None;
        }

        if let Some(pos) = self.zlist.iter().position(|x| *x == control) {
            self.zlist.remove(pos);
        }

        self.name_map.remove(
            world
                .get::<WidgetAny<Message>>(control.get_key())
                .expect("Widget is missing its inner UI Type?")
                .get_id(),
        );

        for child in world
            .query::<Without<(&Widget, &Parent), &Hidden>>()
            .iter()
            .filter(|(entity, (_, parent))| *parent == Parent(control))
            .map(|(entity, _)| entity)
        {
            self.widget_clear_self(world, child);
        }

        for child in world
            .query::<With<(&Widget, &Parent), &Hidden>>()
            .iter()
            .filter(|(entity, (_, parent))| *parent == Parent(control))
            .map(|(entity, _)| entity)
        {
            self.widget_clear_self(world, child);
        }
    }

    // This will remove the children from the Zlist, focused, over and clicked.
    // This does not move the children into the controls hidden Vec.
    // This is because we want to be able to reshow All the visible children
    // when we unhide the control.
    pub(crate) fn widget_hide_children(
        &mut self,
        world: &mut World,
        control: Handle,
    ) {
        for child in world
            .query::<Without<(&Widget, &Parent), &Hidden>>()
            .iter()
            .filter(|(entity, (_, parent))| *parent == Parent(control))
            .map(|(entity, _)| entity)
        {
            let actions: Actions =
                world.get(child).expect("Widget is missing its actions?");
            let child_handle = Handle(child);

            if let Some(pos) =
                self.zlist.iter().position(|x| *x == child_handle)
            {
                self.zlist.remove(pos);
            }

            if actions.exists(UiFlags::AllowChildren) {
                self.widget_hide_children(world, child_handle);
            }

            if self.focused == Some(child_handle) {
                self.focused = None;
            }

            if self.clicked == Some(child_handle) {
                self.clicked = None;
            }

            if self.over == Some(child_handle) {
                self.over = None;
            }
        }
    }

    //This will Advance the children into the Back of the Zlist allowing them to
    //render on top.
    pub(crate) fn widget_show_children(
        &mut self,
        world: &mut World,
        control: Handle,
    ) {
        for child in world
            .query::<Without<(&Widget, &Parent), &Hidden>>()
            .iter()
            .filter(|(entity, (_, parent))| *parent == Parent(control))
            .map(|(entity, _)| entity)
        {
            let actions: Actions =
                world.get(child).expect("Widget is missing its actions?");
            let child_handle = Handle(child);

            if let Some(pos) =
                self.zlist.iter().position(|x| *x == child_handle)
            {
                self.zlist.remove(pos);
            }

            self.zlist.push_back(child_handle);

            if actions.exists(UiFlags::AllowChildren) {
                self.widget_show_children(world, child_handle);
            }
        }
    }

    pub(crate) fn widget_focused_callback(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        control: Handle,
        focused: bool,
        events: &mut Vec<Message>,
    ) {
        let mut ui: WidgetAny<Message> = world
            .get(control.get_key())
            .expect("Widget is missing its inner UI Type?");

        let action: Actions = world
            .get(control.get_key())
            .expect("Widget is missing its actions?");

        ui.event(
            action.0,
            ui_buffer,
            renderer,
            SystemEvent::FocusChange(focused),
            events,
        );
    }

    pub(crate) fn widget_mouse_press_callbacks(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        control: Handle,
        pressed: bool,
        events: &mut Vec<Message>,
    ) {
        let mut ui: WidgetAny<Message> = world
            .get(control.get_key())
            .expect("Widget is missing its inner UI Type?");

        let action: Actions = world
            .get(control.get_key())
            .expect("Widget is missing its actions?");

        let btn = self.button;
        let modifier = self.modifier;

        ui.event(
            action.0,
            ui_buffer,
            renderer,
            SystemEvent::MousePress(btn, pressed, modifier),
            events,
        );
    }

    pub(crate) fn widget_set_clicked(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        control: Handle,
        events: &mut Vec<Message>,
    ) {
        let mut ui: WidgetAny<Message> = world
            .get(control.get_key())
            .expect("Widget is missing its inner UI Type?");

        let action: Actions = world
            .get(control.get_key())
            .expect("Widget is missing its actions?");

        let in_bounds = ui.check_mouse_bounds(self.mouse_clicked);

        if action.exists(UiFlags::CanMoveWindow) && in_bounds {
            self.moving = true;
        }

        if action.exists(UiFlags::CanClickBehind) {
            if let Ok(parent) = world.get::<Parent>(control.get_key()) {
                let mut parent_ui: WidgetAny<Message> = world
                    .get(parent.get_key())
                    .expect("Widget is missing its inner UI Type?");

                let parent_action: Actions = world
                    .get(parent.get_key())
                    .expect("Widget is missing its actions?");

                if parent_action.exists(UiFlags::CanMoveWindow)
                    && parent_ui.check_mouse_bounds(self.mouse_clicked)
                {
                    self.moving = true;
                }

                self.clicked = Some(parent);
                self.widget_mouse_press_callbacks(
                    world, ui_buffer, renderer, parent, true, events,
                );
            }

            return;
        }

        if action.exists(UiFlags::MoveAble) && in_bounds {
            action.set(UiFlags::Moving);
            self.widget_moving = Some(control);
        }

        self.widget_mouse_press_callbacks(
            world, ui_buffer, renderer, control, true, events,
        );
    }

    pub(crate) fn widget_set_focus(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        control: Handle,
        events: &mut Vec<Message>,
    ) {
        if let Some(pos) = self.zlist.iter().position(|x| *x == control) {
            self.zlist.remove(pos);
            self.zlist.push_back(control);
        }

        //This will basically append the children after the parent since they render first.
        self.widget_show_children(world, control);

        if let Some(focused) = self.focused {
            self.widget_focused_callback(
                world, ui_buffer, renderer, focused, false, events,
            );
        }

        self.widget_focused_callback(
            world, ui_buffer, renderer, control, true, events,
        );
        self.widget_set_clicked(world, ui_buffer, renderer, control, events);
    }

    pub(crate) fn is_parent_focused(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        control: Handle,
        events: &mut Vec<Message>,
    ) -> bool {
        let action: Actions = world
            .get(control.get_key())
            .expect("Widget is missing its actions?");

        if action.exists(UiFlags::AlwaysUseable) {
            return true;
        }

        let mut control_parent: Parent = match world.get(control.get_key()) {
            Ok(parent) => parent,
            Err(_) => return false,
        };

        let mut parent_opt = Some(control_parent.get_id());

        while let Some(parent) = parent_opt {
            let parent_action: Actions = world
                .get(parent.get_key())
                .expect("Widget is missing its actions?");

            if parent_action.exists(UiFlags::CanFocus) {
                if parent_action.exists(UiFlags::IsFocused) {
                    return true;
                } else {
                    self.widget_manual_focus(
                        world, ui_buffer, renderer, parent, events,
                    );

                    if parent_action.exists(UiFlags::FocusClick) {
                        self.widget_set_clicked(
                            world, ui_buffer, renderer, parent, events,
                        );
                    }

                    return true;
                }
            } else if parent_action.exists(UiFlags::AlwaysUseable)
                && parent_action.exists(UiFlags::ClickAble)
                && control_parent == Some(parent)
                && action.exists(UiFlags::CanClickBehind)
            {
                return true;
            }

            parent_opt = match world.get::<Parent>(control.get_key()) {
                Ok(parent) => parent.get_id(),
                Err(_) => return false,
            };
        }

        false
    }

    pub(crate) fn widget_is_focused(
        &mut self,
        world: &mut World,
        control: Handle,
    ) -> bool {
        let action: Actions = world
            .get(control.get_key())
            .expect("Widget is missing its actions?");

        if action.exists(UiFlags::IsFocused) {
            return true;
        }

        let mut control_parent: Parent = match world.get(control.get_key()) {
            Ok(parent) => parent,
            Err(_) => return false,
        };

        let mut parent_opt = Some(control_parent.get_id());

        while let Some(parent_handle) = parent_opt {
            let parent_action: Actions = world
                .get(parent_handle.get_key())
                .expect("Widget is missing its actions?");

            if parent_action.exists(UiFlags::IsFocused) {
                return true;
            }

            parent_opt = match world.get::<Parent>(control.get_key()) {
                Ok(parent) => parent.get_id(),
                Err(_) => return false,
            };
        }

        false
    }

    pub(crate) fn mouse_press_event(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        control: Handle,
        events: &mut Vec<Message>,
    ) {
        let action: Actions = world
            .get(control.get_key())
            .expect("Widget is missing its actions?");

        if action.exists(UiFlags::CanFocus) {
            if self.focused != Some(control) {
                self.widget_set_focus(
                    world, ui_buffer, renderer, control, events,
                );
            } else {
                self.widget_set_clicked(
                    world, ui_buffer, renderer, control, events,
                );
            }
        } else if self
            .is_parent_focused(world, ui_buffer, renderer, control, events)
        {
            self.widget_set_clicked(
                world, ui_buffer, renderer, control, events,
            );
        }
    }

    pub(crate) fn mouse_press(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        events: &mut Vec<Message>,
    ) {
        for handle in self.zlist.clone().iter().rev() {
            let action: Actions = world
                .get(handle.get_key())
                .expect("Widget is missing its actions?");

            let mut ui: WidgetAny<Message> = world
                .get(handle.get_key())
                .expect("Widget is missing its inner UI Type?");

            if action.exists(UiFlags::ClickAble)
                && ui.check_mouse_bounds(self.mouse_clicked)
            {
                if action.exists(UiFlags::MoveAble) {
                    action.clear(UiFlags::Moving);
                }

                self.mouse_press_event(
                    world, ui_buffer, renderer, handle, events,
                );
                return;
            }

            if action.exists(UiFlags::MoveAble)
                && ui.check_mouse_bounds(self.mouse_clicked)
            {
                action.clear(UiFlags::Moving);
            }
        }
    }

    pub(crate) fn mouse_release(
        &mut self,
        world: &mut World,
        ui_buffer: &mut UIBuffer,
        renderer: &mut GpuRenderer,
        events: &mut Vec<Message>,
    ) {
        if let Some(focused_handle) = self.focused {
            let action: Actions = world
                .get(focused_handle.get_key())
                .expect("Widget is missing its actions?");

            if action.exists(UiFlags::Moving)
                && self.widget_moving == Some(focused_handle)
            {
                action.clear(UiFlags::Moving);
                self.widget_moving = None;
            }
        }

        for handle in self.zlist.clone().iter().rev() {
            let action: Actions = world
                .get(handle.get_key())
                .expect("Widget is missing its actions?");

            let mut ui: WidgetAny<Message> = world
                .get(handle.get_key())
                .expect("Widget is missing its inner UI Type?");

            if action.exists(UiFlags::ClickAble)
                && ui.check_mouse_bounds(self.mouse_clicked)
            {
                if action.exists(UiFlags::CanMoveWindow) {
                    self.moving = false;
                }

                if action.exists(UiFlags::Moving)
                    && self.widget_moving == Some(*handle)
                {
                    action.clear(UiFlags::Moving);
                    self.widget_moving = None;
                }

                self.widget_mouse_press_callbacks(
                    world, ui_buffer, renderer, handle, false, events,
                );
                return;
            }
        }
    }

    pub(crate) fn widget_position_update(
        &mut self,
        _renderer: &mut GpuRenderer,
        _parent: &mut Widget,
    ) {
        //TODO Find good way to handle position updates for widgets being dragged around.
        /*let mut control = control;

        control.ui.event(
            control.actions,
            self.ui_buffer_mut(),
            renderer,
            SystemEvent::PositionChange,
            &mut vec![],
        );

        let key = parent.callback_key(Event::PositionChange);

        if let Some(callback) = self.get_inner_callback(&key) {
            if let InternalCallBacks::PositionChange(internal_update_pos) =
                callback
            {
                internal_update_pos(parent, self, renderer);
            }
        }

        for handle in &parent.visible {
            let widget = self.get_widget(*handle);

            if !widget.visible.is_empty() {
                self.widget_position_update(renderer, &mut widget);
            } else {
                let key = widget.callback_key(Event::PositionChange);
                let mut mut_wdgt = widget;

                if let Some(callback) = self.get_inner_callback(&key) {
                    if let InternalCallBacks::PositionChange(
                        internal_update_pos,
                    ) = callback
                    {
                        internal_update_pos(&mut mut_wdgt, self, renderer);
                    }
                }
            }
        }*/
    }
}
