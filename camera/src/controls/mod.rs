mod first_person;
mod flying;
mod orbit;
mod flat;

pub trait Controls {
    /// Retrieves the view matrix.
    fn view(&self) -> mint::ColumnMatrix4<f32>;

    /// Retrieves the eye position.
    fn eye(&self) -> [f32; 3];

    /// Processes the inputs and recalculates the view matrix and eye position if the state
    /// changed. Returns `true` if anything was updated, otherwise returns `false`.
    fn update(&mut self, delta: f32) -> bool;
}

pub use first_person::{FirstPersonControls, FirstPersonInputs, FirstPersonSettings};
pub use flying::{FlyingControls, FlyingInputs, FlyingSettings};
pub use orbit::{OrbitControls, OrbitInputs, OrbitSettings};
pub use flat::{FlatControls, FlatSettings, FlatInputs};
