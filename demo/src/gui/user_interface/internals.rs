use crate::{
    FrameTime, Handle, Identity, SystemEvent, UIBuffer, UiFlags, Widget,
    WidgetRef, UI,
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

impl<Message> UI<Message> {
    pub(crate) fn mouse_over_event(
        &mut self,
        renderer: &mut GpuRenderer,
        events: &mut Vec<Message>,
    ) {
        for &handle in self.zlist.clone().iter().rev() {
            let control = self.get_widget(handle);

            if control.borrow().ui.check_mouse_bounds(self.mouse_pos)
                && self.widget_usable(&control)
            {
                if control.borrow().actions.get(UiFlags::CanClickBehind) {
                    if let Some(parent_handle) = control.borrow().parent {
                        let parent = self.get_widget(parent_handle);

                        if !parent.borrow().actions.get(UiFlags::Moving) {
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
            } else if !control.borrow().actions.get(UiFlags::Moving) {
                self.widget_mouse_over(renderer, &control, false, events);
            }
        }
    }

    pub(crate) fn widget_mouse_over_callback(
        &mut self,
        renderer: &mut GpuRenderer,
        control: &WidgetRef<Message>,
        entered: bool,
        events: &mut Vec<Message>,
    ) {
        let mut control = control.borrow_mut();
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
        renderer: &mut GpuRenderer,
        control: &WidgetRef<Message>,
        entered: bool,
        events: &mut Vec<Message>,
    ) {
        if entered {
            if self.over.is_some()
                && self.over != Some(control.borrow().id)
                && self.widget_moving.is_none()
            {
                let over = self.get_widget(self.over.unwrap());

                over.borrow_mut().actions.clear(UiFlags::MouseOver);
                control.borrow_mut().actions.set(UiFlags::MouseOver);
                self.widget_mouse_over_callback(renderer, &over, false, events);
                self.over = Some(control.borrow().id);
                self.widget_mouse_over_callback(
                    renderer, control, true, events,
                );
            } else if self.over.is_none() {
                self.over = Some(control.borrow().id);
                control.borrow_mut().actions.set(UiFlags::MouseOver);
                self.widget_mouse_over_callback(
                    renderer, control, true, events,
                );
            }
        } else if let Some(over_handle) = self.over {
            let over = self.get_widget(over_handle);

            if !over.borrow().ui.check_mouse_bounds(self.mouse_pos)
                && over.borrow().actions.get(UiFlags::MouseOver)
                && self.widget_moving.is_none()
            {
                over.borrow_mut().actions.clear(UiFlags::MouseOver);
                self.widget_mouse_over_callback(renderer, &over, false, events);
                self.over = None;
            }
        }
    }

    pub(crate) fn widget_usable(&self, control: &WidgetRef<Message>) -> bool {
        if control.borrow().actions.get(UiFlags::AlwaysUseable) {
            return true;
        }

        if !control.borrow().actions.get(UiFlags::IsFocused) {
            let mut parent_handle = control.borrow().parent;

            while let Some(handle) = parent_handle {
                let parent = self.get_widget(handle);

                if (parent.borrow().actions.get(UiFlags::CanFocus)
                    && parent.borrow().actions.get(UiFlags::IsFocused))
                    || parent.borrow().actions.get(UiFlags::AlwaysUseable)
                {
                    return true;
                }

                parent_handle = parent.borrow().parent;
            }

            false
        } else {
            true
        }
    }

    pub(crate) fn widget_manual_focus(
        &mut self,
        renderer: &mut GpuRenderer,
        control: &WidgetRef<Message>,
    ) {
        let handle = control.borrow().id;

        if control.borrow().actions.get(UiFlags::CanFocus) {
            if let Some(pos) = self.zlist.iter().position(|x| *x == handle) {
                self.zlist.remove(pos);
                self.zlist.push_back(handle);
            }

            self.widget_show_children(control);

            if let Some(parent_handle) = control.borrow().parent {
                let wdgt = self.get_widget(parent_handle);
                let mut parent = wdgt.borrow_mut();

                if let Some(pos) =
                    parent.visible.iter().position(|x| *x == handle)
                {
                    parent.visible.remove(pos);
                    parent.visible.push_back(handle);
                }
            } else if let Some(pos) =
                self.visible.iter().position(|x| *x == handle)
            {
                self.visible.remove(pos);
                self.visible.push_back(handle);
            }

            if let Some(focused_handle) = self.focused {
                self.widget_focused_callback(
                    renderer,
                    &self.get_widget(focused_handle),
                    false,
                );
            }

            control.borrow_mut().actions.set(UiFlags::IsFocused);
            self.focused = Some(handle);
            self.widget_focused_callback(renderer, control, true);
        }
    }

    pub(crate) fn widget_show(
        &mut self,
        renderer: &mut GpuRenderer,
        control: &WidgetRef<Message>,
    ) {
        let handle = control.borrow().id;

        if control.borrow().parent.is_none() {
            self.visible.push_back(handle);

            if let Some(pos) = self.hidden.iter().position(|x| *x == handle) {
                self.hidden.remove(pos);
            }
        }

        self.zlist.push_back(handle);
        self.widget_show_children(control);

        if !self.widget_is_focused(control) && self.focused.is_some() {
            let focused = self.get_widget(self.focused.unwrap());

            self.widget_manual_focus(renderer, &focused);
        }
    }

    pub(crate) fn widget_hide(&mut self, control: &WidgetRef<Message>) {
        let handle = control.borrow().id;

        if control.borrow().parent.is_none() {
            if let Some(pos) = self.visible.iter().position(|x| *x == handle) {
                self.visible.remove(pos);
            }

            self.hidden.push(handle);
        }

        if let Some(pos) = self.zlist.iter().position(|x| *x == handle) {
            self.zlist.remove(pos);
        }

        self.widget_hide_children(control);

        if self.focused == Some(handle) {
            self.focused = None;
        }
    }

    pub(crate) fn widget_add(
        &mut self,
        parent: Option<&WidgetRef<Message>>,
        control: WidgetRef<Message>,
    ) {
        let id = control.borrow().get_identity();
        if self.name_map.contains_key(&id) {
            panic!("You can not use the same Identity for multiple widgets");
        }

        let handle = Handle(self.widgets.insert(control));
        let control = self.get_widget(handle);

        control.borrow_mut().id = handle;
        self.name_map.insert(id, handle);

        if parent.is_none() {
            self.visible.push_back(handle);
            self.zlist.push_back(handle);
            self.widget_show_children(&control)
        } else if let Some(parent) = parent {
            parent.borrow_mut().visible.push_back(handle);
            self.widget_show_children(parent);
        }
    }

    pub(crate) fn widget_add_hidden(
        &mut self,
        parent: Option<&WidgetRef<Message>>,
        control: WidgetRef<Message>,
    ) {
        let id = control.borrow().get_identity();

        if self.name_map.contains_key(&id) {
            panic!("You can not use the same Identity for multiple widgets even if hidden");
        }

        //let callbacks = control.borrow().ui
        let handle = Handle(self.widgets.insert(control));
        let control = self.get_widget(handle);

        control.borrow_mut().id = handle;
        self.name_map.insert(id, handle);

        if parent.is_none() {
            self.hidden.push(handle);
        } else if let Some(parent) = parent {
            parent.borrow_mut().hidden.push(handle);
        }
    }

    pub(crate) fn widget_clear_self(&mut self, control: &WidgetRef<Message>) {
        let handle = control.borrow().id;

        if control.borrow().parent.is_none() {
            if let Some(pos) = self.visible.iter().position(|x| *x == handle) {
                self.visible.remove(pos);
            }

            if let Some(pos) = self.hidden.iter().position(|x| *x == handle) {
                self.hidden.remove(pos);
            }
        }

        if self.focused == Some(handle) {
            self.focused = None;
        }

        if self.clicked == Some(handle) {
            self.clicked = None;
        }

        if self.over == Some(handle) {
            self.over = None;
        }

        if let Some(pos) = self.zlist.iter().position(|x| *x == handle) {
            self.zlist.remove(pos);
        }

        let identity = control.borrow().get_identity();
        if let Some(identity) = self.name_map.remove(&identity) {
            self.widgets.remove(identity.get_key());
        }

        self.widget_clear_visible(control);
        self.widget_clear_hidden(control);
    }

    pub(crate) fn widget_clear_visible(
        &mut self,
        control: &WidgetRef<Message>,
    ) {
        for child_handle in &control.borrow().visible {
            self.widget_clear_self(&self.get_widget(*child_handle));
        }

        control.borrow_mut().hidden.clear();
        control.borrow_mut().visible.clear();
    }

    pub(crate) fn widget_clear_hidden(&mut self, control: &WidgetRef<Message>) {
        for child_handle in &control.borrow().hidden {
            self.widget_clear_self(&self.get_widget(*child_handle));
        }

        control.borrow_mut().hidden.clear();
        control.borrow_mut().visible.clear();
    }

    // This will remove the children from the Zlist, focused, over and clicked.
    // This does not move the children into the controls hidden Vec.
    // This is because we want to be able to reshow All the visible children
    // when we unhide the control.
    pub(crate) fn widget_hide_children(
        &mut self,
        control: &WidgetRef<Message>,
    ) {
        for child_handle in &control.borrow().visible {
            let child = self.get_widget(*child_handle);

            if let Some(pos) =
                self.zlist.iter().position(|x| *x == *child_handle)
            {
                self.zlist.remove(pos);
            }

            self.widget_hide_children(&child);

            if self.focused == Some(*child_handle) {
                self.focused = None;
            }

            if self.clicked == Some(*child_handle) {
                self.clicked = None;
            }

            if self.over == Some(*child_handle) {
                self.over = None;
            }
        }
    }

    //This will Advance the children into the Back of the Zlist allowing them to
    //render on top.
    pub(crate) fn widget_show_children(
        &mut self,
        control: &WidgetRef<Message>,
    ) {
        for child_handle in &control.borrow().visible {
            let child = self.get_widget(*child_handle);

            if let Some(pos) =
                self.zlist.iter().position(|x| *x == *child_handle)
            {
                self.zlist.remove(pos);
                self.zlist.push_back(*child_handle);
            } else {
                self.zlist.push_back(*child_handle);
            }

            self.widget_show_children(&child);
        }
    }

    pub(crate) fn widget_focused_callback(
        &mut self,
        renderer: &mut GpuRenderer,
        control: &WidgetRef<Message>,
        focused: bool,
    ) {
        let mut control = control.borrow_mut();
        let actions = control.actions;
        control.ui.event(
            actions,
            self.ui_buffer_mut(),
            renderer,
            SystemEvent::FocusChange(focused),
            &mut vec![],
        );
    }

    pub(crate) fn widget_mouse_press_callbacks(
        &mut self,
        renderer: &mut GpuRenderer,
        control: &WidgetRef<Message>,
        pressed: bool,
        events: &mut Vec<Message>,
    ) {
        let mut control = control.borrow_mut();
        let actions = control.actions;
        let btn = self.button;
        let modifier = self.modifier;

        control.ui.event(
            actions,
            self.ui_buffer_mut(),
            renderer,
            SystemEvent::MousePress(btn, pressed, modifier),
            events,
        );
    }

    pub(crate) fn widget_set_clicked(
        &mut self,
        renderer: &mut GpuRenderer,
        control: &WidgetRef<Message>,
        events: &mut Vec<Message>,
    ) {
        {
            let mut refctrl = control.borrow_mut();
            let in_bounds = refctrl.ui.check_mouse_bounds(self.mouse_clicked);

            if refctrl.actions.get(UiFlags::CanMoveWindow) && in_bounds {
                self.moving = true;
            }

            if refctrl.actions.get(UiFlags::CanClickBehind) {
                if let Some(parent_handle) = refctrl.parent {
                    let parent = self.get_widget(parent_handle);

                    if parent.borrow().actions.get(UiFlags::CanMoveWindow)
                        && parent
                            .borrow()
                            .ui
                            .check_mouse_bounds(self.mouse_clicked)
                    {
                        self.moving = true;
                    }

                    self.clicked = Some(parent_handle);
                    self.widget_mouse_press_callbacks(
                        renderer, &parent, true, events,
                    );
                }

                return;
            }

            if refctrl.actions.get(UiFlags::MoveAble) && in_bounds {
                refctrl.actions.set(UiFlags::Moving);
                self.widget_moving = Some(refctrl.id);
            }
        }

        self.widget_mouse_press_callbacks(renderer, control, true, events);
    }

    pub(crate) fn widget_set_focus(
        &mut self,
        renderer: &mut GpuRenderer,
        control: &WidgetRef<Message>,
        events: &mut Vec<Message>,
    ) {
        let handle = control.borrow().id;

        if let Some(pos) = self.zlist.iter().position(|x| *x == handle) {
            self.zlist.remove(pos);
            self.zlist.push_back(handle);
        }
        //This will basically append the children after the parent since they render first.
        self.widget_show_children(control);

        if let Some(parent_handle) = control.borrow().parent {
            let wdgt = self.get_widget(parent_handle);
            let mut parent = wdgt.borrow_mut();

            if let Some(pos) = parent.visible.iter().position(|x| *x == handle)
            {
                parent.visible.remove(pos);
                parent.visible.push_back(handle);
            }
        }

        if let Some(focused_handle) = self.focused {
            let focused = self.get_widget(focused_handle);
            self.widget_focused_callback(renderer, &focused, false);
        }

        self.widget_focused_callback(renderer, control, true);
        self.widget_set_clicked(renderer, control, events);
    }

    pub(crate) fn is_parent_focused(
        &mut self,
        renderer: &mut GpuRenderer,
        control: &WidgetRef<Message>,
        events: &mut Vec<Message>,
    ) -> bool {
        if control.borrow().actions.get(UiFlags::AlwaysUseable) {
            return true;
        }

        let mut parent_opt = control.borrow().parent;

        while let Some(parent_handle) = parent_opt {
            let parent = self.get_widget(parent_handle);

            if parent.borrow().actions.get(UiFlags::CanFocus) {
                if parent.borrow().actions.get(UiFlags::IsFocused) {
                    return true;
                } else {
                    self.widget_manual_focus(renderer, &parent);

                    if parent.borrow().actions.get(UiFlags::FocusClick) {
                        self.widget_set_clicked(renderer, &parent, events);
                    }

                    return true;
                }
            } else if parent.borrow().actions.get(UiFlags::AlwaysUseable)
                && parent.borrow().actions.get(UiFlags::ClickAble)
                && control.borrow().parent == Some(parent_handle)
                && control.borrow().actions.get(UiFlags::CanClickBehind)
            {
                return true;
            }

            parent_opt = parent.borrow().parent;
        }

        false
    }

    pub(crate) fn widget_is_focused(
        &mut self,
        control: &WidgetRef<Message>,
    ) -> bool {
        if control.borrow().actions.get(UiFlags::IsFocused) {
            return true;
        }

        let mut parent_opt = control.borrow().parent;

        while let Some(parent_handle) = parent_opt {
            let parent = self.get_widget(parent_handle);

            if parent.borrow().actions.get(UiFlags::IsFocused) {
                return true;
            }

            parent_opt = parent.borrow().parent;
        }

        false
    }

    pub(crate) fn mouse_press_event(
        &mut self,
        renderer: &mut GpuRenderer,
        control: &WidgetRef<Message>,
        events: &mut Vec<Message>,
    ) {
        if control.borrow().actions.get(UiFlags::CanFocus) {
            if self.focused != Some(control.borrow().id) {
                self.widget_set_focus(renderer, control, events);
            } else {
                self.widget_set_clicked(renderer, control, events);
            }
        } else if self.is_parent_focused(renderer, control, events) {
            self.widget_set_clicked(renderer, control, events);
        }
    }

    pub(crate) fn mouse_press(
        &mut self,
        renderer: &mut GpuRenderer,
        events: &mut Vec<Message>,
    ) {
        for handle in self.zlist.clone().iter().rev() {
            let child = self.get_widget(*handle);

            if child.borrow().actions.get(UiFlags::ClickAble)
                && child.borrow().ui.check_mouse_bounds(self.mouse_clicked)
            {
                if child.borrow().actions.get(UiFlags::MoveAble) {
                    child.borrow_mut().actions.clear(UiFlags::Moving);
                }

                self.mouse_press_event(renderer, &child, events);
                return;
            }

            if child.borrow().actions.get(UiFlags::MoveAble)
                && child.borrow().ui.check_mouse_bounds(self.mouse_clicked)
            {
                child.borrow_mut().actions.clear(UiFlags::Moving);
            }
        }
    }

    pub(crate) fn mouse_release(
        &mut self,
        renderer: &mut GpuRenderer,
        events: &mut Vec<Message>,
    ) {
        if let Some(focused_handle) = self.focused {
            let focused = self.get_widget(focused_handle);

            if focused.borrow().actions.get(UiFlags::Moving)
                && self.widget_moving == Some(focused_handle)
            {
                focused.borrow_mut().actions.clear(UiFlags::Moving);
                self.widget_moving = None;
            }
        }

        for handle in self.zlist.clone().iter().rev() {
            let control = self.get_widget(*handle);

            if control.borrow().actions.get(UiFlags::ClickAble)
                && control.borrow().ui.check_mouse_bounds(self.mouse_clicked)
            {
                if control.borrow().actions.get(UiFlags::CanMoveWindow) {
                    self.moving = false;
                }

                if control.borrow().actions.get(UiFlags::Moving)
                    && self.widget_moving == Some(*handle)
                {
                    control.borrow_mut().actions.clear(UiFlags::Moving);
                    self.widget_moving = None;
                }

                self.widget_mouse_press_callbacks(
                    renderer, &control, false, events,
                );
                return;
            }
        }
    }

    pub(crate) fn widget_position_update(
        &mut self,
        _renderer: &mut GpuRenderer,
        _parent: &mut Widget<Message>,
    ) {
        //TODO Find good way to handle position updates for widgets being dragged around.
        /*let mut control = control.borrow_mut();

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

            if !widget.borrow().visible.is_empty() {
                self.widget_position_update(renderer, &mut widget.borrow_mut());
            } else {
                let key = widget.borrow().callback_key(Event::PositionChange);
                let mut mut_wdgt = widget.borrow_mut();

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
