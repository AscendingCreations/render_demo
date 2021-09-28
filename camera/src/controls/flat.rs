use super::Controls;
use ultraviolet::{Mat4, Vec3};

#[derive(Clone, Debug, Default)]
pub struct FlatInputs {
    /// move in this direction.
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
}

#[derive(Clone, Debug)]
pub struct FlatSettings {
    pub scrollspeed: f32,
}

impl Default for FlatSettings {
    fn default() -> Self {
        Self { scrollspeed: 1.0 }
    }
}

#[derive(Clone, Debug)]
pub struct FlatControls {
    inputs: FlatInputs,
    settings: FlatSettings,
    view: Mat4,
    eye: Vec3,
    changed: bool,
}

impl FlatControls {
    pub fn new(settings: FlatSettings) -> Self {
        Self {
            inputs: FlatInputs::default(),
            settings,
            view: Mat4::identity(),
            eye: Vec3::zero(),
            changed: true,
        }
    }

    pub fn inputs(&self) -> &FlatInputs {
        &self.inputs
    }

    pub fn set_inputs(&mut self, inputs: FlatInputs) {
        self.inputs = inputs;
        self.changed = true;
    }
}

impl Controls for FlatControls {
    fn view(&self) -> mint::ColumnMatrix4<f32> {
        self.view.into()
    }

    fn eye(&self) -> [f32; 3] {
        self.eye.into()
    }

    fn update(&mut self, delta: f32) -> bool {
        let mut changed = self.changed;

        if changed {
            self.view = Mat4::identity();
        }

        self.changed = false;
        changed
    }
}
