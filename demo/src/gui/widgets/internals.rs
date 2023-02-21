use crate::{
    CallBack, CallBackKey, CallBacks, FrameTime, GuiRender, Handle, Identity,
    InternalCallBacks, UiFlags, Widget, WidgetRef, Widgets,
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

impl<T> Widgets<T> {
    pub(crate) fn get_widget(&self, handle: Handle) -> WidgetRef {
        self.widgets
            .get(handle.get_key())
            .expect("ID Existed but widget does not exist?")
            .clone()
    }

    pub(crate) fn get_user_callback(
        &self,
        key: &CallBackKey,
    ) -> Option<Rc<CallBacks<T>>> {
        self.user_callbacks.get(key).cloned()
    }

    pub(crate) fn get_inner_callback(
        &self,
        key: &CallBackKey,
    ) -> Option<Rc<InternalCallBacks<T>>> {
        self.callbacks.get(key).cloned()
    }

    pub(crate) fn mouse_over_event(&mut self, user_data: &mut T) {
        for &handle in self.zlist.clone().iter().rev() {
            let control = self.get_widget(handle);

            if control.borrow().ui.check_mouse_bounds(self.mouse_pos)
                && self.widget_usable(&control)
            {
                if control.borrow().actions.get(UiFlags::CanClickBehind) {
                    if let Some(parent_handle) = control.borrow().parent {
                        let parent = self.get_widget(parent_handle);

                        if !parent.borrow().actions.get(UiFlags::Moving) {
                            self.widget_mouse_over(&parent, true, user_data);
                        }

                        return;
                    }
                } else {
                    self.widget_mouse_over(&control, true, user_data);
                    return;
                }
            } else if !control.borrow().actions.get(UiFlags::Moving) {
                self.widget_mouse_over(&control, false, user_data);
            }
        }
    }

    pub(crate) fn widget_mouse_over_callback(
        &mut self,
        control: &WidgetRef,
        entered: bool,
        user_data: &mut T,
    ) {
        let key = control.borrow().callback_key(CallBack::MousePresent);

        if let Some(callback) = self.get_inner_callback(&key) {
            if let InternalCallBacks::MousePresent(present) = callback.as_ref()
            {
                present(&mut control.borrow_mut(), self, entered);
            }
        }

        if let Some(callback) = self.get_user_callback(&key) {
            if let CallBacks::MousePresent(present) = callback.as_ref() {
                present(&mut control.borrow_mut(), self, entered, user_data);
            }
        }
    }

    pub(crate) fn widget_mouse_over(
        &mut self,
        control: &WidgetRef,
        entered: bool,
        user_data: &mut T,
    ) {
        if entered {
            if self.over.is_some()
                && !self.over.contains(&control.borrow().id)
                && self.widget_moving.is_none()
            {
                let over = self.get_widget(self.over.unwrap());

                over.borrow_mut().actions.clear(UiFlags::MouseOver);
                control.borrow_mut().actions.set(UiFlags::MouseOver);
                self.widget_mouse_over_callback(&over, false, user_data);
                self.over = Some(control.borrow().id);
                self.widget_mouse_over_callback(control, true, user_data);
            } else if self.over.is_none() {
                self.over = Some(control.borrow().id);
                control.borrow_mut().actions.set(UiFlags::MouseOver);
                self.widget_mouse_over_callback(control, true, user_data);
            }
        } else if let Some(over_handle) = self.over {
            let over = self.get_widget(over_handle);

            if !over.borrow().ui.check_mouse_bounds(self.mouse_pos)
                && over.borrow().actions.get(UiFlags::MouseOver)
                && self.widget_moving.is_none()
            {
                self.over = None;
                control.borrow_mut().actions.clear(UiFlags::MouseOver);
                self.widget_mouse_over_callback(control, false, user_data);
            }
        }
    }

    pub(crate) fn widget_usable(&self, control: &WidgetRef) -> bool {
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

    pub(crate) fn widget_manual_focus(&mut self, control: &WidgetRef) {
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
                    &self.get_widget(focused_handle),
                    false,
                );
            }

            control.borrow_mut().actions.set(UiFlags::IsFocused);
            self.focused = Some(handle);
            self.widget_focused_callback(control, true);
        }
    }

    pub(crate) fn widget_show(&mut self, control: &WidgetRef) {
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

            self.widget_manual_focus(&focused);
        }
    }

    pub(crate) fn widget_hide(&mut self, control: &WidgetRef) {
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
        parent: Option<&WidgetRef>,
        control: WidgetRef,
    ) {
        if self.name_map.contains_key(&control.borrow().identity) {
            panic!("You can not use the same Identity for multiple widgets");
        }

        let handle = Handle(self.widgets.insert(control));
        let control = self.get_widget(handle);

        control.borrow_mut().id = handle;
        self.name_map
            .insert(control.borrow().identity.clone(), handle);

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
        parent: Option<&WidgetRef>,
        control: WidgetRef,
    ) {
        if self.name_map.contains_key(&control.borrow().identity) {
            panic!("You can not use the same Identity for multiple widgets even if hidden");
        }

        let handle = Handle(self.widgets.insert(control));
        let control = self.get_widget(handle);

        control.borrow_mut().id = handle;
        self.name_map
            .insert(control.borrow().identity.clone(), handle);

        if parent.is_none() {
            self.hidden.push(handle);
        } else if let Some(parent) = parent {
            parent.borrow_mut().hidden.push(handle);
        }
    }

    pub(crate) fn widget_clear_self(&mut self, control: &WidgetRef) {
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

        let identity = control.borrow().identity.clone();
        if let Some(identity) = self.name_map.remove(&identity) {
            self.widgets.remove(identity.get_key());
        }

        let callbacks = [
            CallBack::MousePress,
            CallBack::BoundsChange,
            CallBack::Draw,
            CallBack::FocusChange,
            CallBack::KeyPress,
            CallBack::MousePresent,
            CallBack::MouseScroll,
            CallBack::PositionChange,
            CallBack::ValueChanged,
        ];

        for callback in callbacks {
            let key = control.borrow().callback_key(callback);
            self.callbacks.remove(&key);
            self.user_callbacks.remove(&key);
        }

        self.widget_clear_visible(control);
        self.widget_clear_hidden(control);
    }

    pub(crate) fn widget_clear_visible(&mut self, control: &WidgetRef) {
        for child_handle in &control.borrow().visible {
            self.widget_clear_self(&self.get_widget(*child_handle));
        }

        control.borrow_mut().hidden.clear();
        control.borrow_mut().visible.clear();
    }

    pub(crate) fn widget_clear_hidden(&mut self, control: &WidgetRef) {
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
    pub(crate) fn widget_hide_children(&mut self, control: &WidgetRef) {
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
    pub(crate) fn widget_show_children(&mut self, control: &WidgetRef) {
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
        control: &WidgetRef,
        focused: bool,
    ) {
        let mut mut_wdgt = control.borrow_mut();
        let key = mut_wdgt.callback_key(CallBack::MousePress);

        mut_wdgt.actions.set(UiFlags::IsFocused);
        self.focused = Some(mut_wdgt.id);

        if let Some(callback) = self.get_inner_callback(&key) {
            if let InternalCallBacks::FocusChange(focus_changed) =
                callback.as_ref()
            {
                focus_changed(&mut mut_wdgt, self, focused);
            }
        }
    }

    pub(crate) fn widget_mouse_press_callbacks(
        &mut self,
        control: &WidgetRef,
        pressed: bool,
        user_data: &mut T,
    ) {
        let mut mut_wdgt = control.borrow_mut();
        let key = mut_wdgt.callback_key(CallBack::MousePress);

        if let Some(callback) = self.get_inner_callback(&key) {
            if let InternalCallBacks::MousePress(mouse_press) =
                callback.as_ref()
            {
                mouse_press(
                    &mut mut_wdgt,
                    self,
                    self.button,
                    pressed,
                    self.modifier,
                );
            }
        }

        if let Some(callback) = self.get_user_callback(&key) {
            if let CallBacks::MousePress(mouse_press) = callback.as_ref() {
                mouse_press(
                    &mut mut_wdgt,
                    self,
                    self.button,
                    pressed,
                    self.modifier,
                    user_data,
                );
            }
        }
    }

    pub(crate) fn widget_set_clicked(
        &mut self,
        control: &WidgetRef,
        user_data: &mut T,
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
                    self.widget_mouse_press_callbacks(&parent, true, user_data);
                }

                return;
            }

            if refctrl.actions.get(UiFlags::MoveAble) && in_bounds {
                refctrl.actions.set(UiFlags::Moving);
                self.widget_moving = Some(refctrl.id);
            }
        }

        self.widget_mouse_press_callbacks(control, true, user_data);
    }

    pub(crate) fn widget_set_focus(
        &mut self,
        control: &WidgetRef,
        user_data: &mut T,
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
            self.widget_focused_callback(&focused, false);
        }

        self.widget_focused_callback(control, true);
        self.widget_set_clicked(control, user_data);
    }

    pub(crate) fn is_parent_focused(
        &mut self,
        control: &WidgetRef,
        user_data: &mut T,
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
                    self.widget_manual_focus(&parent);

                    if parent.borrow().actions.get(UiFlags::FocusClick) {
                        self.widget_set_clicked(&parent, user_data);
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

    pub(crate) fn widget_is_focused(&mut self, control: &WidgetRef) -> bool {
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
        control: &WidgetRef,
        user_data: &mut T,
    ) {
        if control.borrow().actions.get(UiFlags::CanFocus) {
            if self.focused != Some(control.borrow().id) {
                self.widget_set_focus(control, user_data);
            } else {
                self.widget_set_clicked(control, user_data);
            }
        } else if self.is_parent_focused(control, user_data) {
            self.widget_set_clicked(control, user_data);
        }
    }

    pub(crate) fn mouse_press(&mut self, user_data: &mut T) {
        for handle in self.zlist.clone().iter().rev() {
            let child = self.get_widget(*handle);

            if child.borrow().actions.get(UiFlags::ClickAble)
                && child.borrow().ui.check_mouse_bounds(self.mouse_clicked)
            {
                if child.borrow().actions.get(UiFlags::MoveAble) {
                    child.borrow_mut().actions.clear(UiFlags::Moving);
                }

                self.mouse_press_event(&child, user_data);
                return;
            }

            if child.borrow().actions.get(UiFlags::MoveAble)
                && child.borrow().ui.check_mouse_bounds(self.mouse_clicked)
            {
                child.borrow_mut().actions.clear(UiFlags::Moving);
            }
        }
    }

    pub(crate) fn mouse_release(&mut self, user_data: &mut T) {
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

                self.widget_mouse_press_callbacks(&control, false, user_data);
                return;
            }
        }
    }

    pub(crate) fn widget_position_update(&mut self, parent: &mut Widget) {
        let key = parent.callback_key(CallBack::PositionChange);

        if let Some(callback) = self.get_inner_callback(&key) {
            if let InternalCallBacks::PositionChange(internal_update_pos) =
                callback.as_ref()
            {
                internal_update_pos(parent, self);
            }
        }

        for handle in &parent.visible {
            let widget = self.get_widget(*handle);

            if !widget.borrow().visible.is_empty() {
                self.widget_position_update(&mut widget.borrow_mut());
            } else {
                let key =
                    widget.borrow().callback_key(CallBack::PositionChange);
                let mut mut_wdgt = widget.borrow_mut();

                if let Some(callback) = self.get_inner_callback(&key) {
                    if let InternalCallBacks::PositionChange(
                        internal_update_pos,
                    ) = callback.as_ref()
                    {
                        internal_update_pos(&mut mut_wdgt, self);
                    }
                }
            }
        }
    }
}
