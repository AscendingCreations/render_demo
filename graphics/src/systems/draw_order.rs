use crate::{BufferDetails, Vec3};
use generational_array::GenerationalIndex;
use std::cmp::Ordering;

pub type Index = GenerationalIndex;

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct DrawOrder {
    pub layer: u32, // lowest to highest. for spliting different types into layers.
    pub alpha: bool, // alpha always is highest
    pub x: u32,     // Lower is lower
    pub y: u32,     // higher is lower
    pub z: u32,     // lower is higher
}

impl PartialOrd for DrawOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DrawOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        self.layer
            .cmp(&other.layer)
            .then(self.alpha.cmp(&other.alpha))
            .then(self.x.cmp(&other.x))
            .then(self.y.cmp(&other.y).reverse())
            .then(self.z.cmp(&other.z).reverse())
    }
}

impl DrawOrder {
    pub fn new(alpha: bool, pos: &Vec3, layer: u32) -> Self {
        Self {
            layer,
            alpha,
            x: (pos.x * 100.0) as u32,
            y: (pos.y * 100.0) as u32,
            z: (pos.z * 100.0) as u32,
        }
    }
}

#[derive(Copy, Clone)]
pub struct OrderedIndex {
    pub(crate) order: DrawOrder,
    pub(crate) index: Index,
    pub(crate) render_details: BufferDetails,
}

impl PartialOrd for OrderedIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for OrderedIndex {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order
    }
}

impl Eq for OrderedIndex {}

impl Ord for OrderedIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order.cmp(&other.order)
    }
}

impl OrderedIndex {
    pub fn new(order: DrawOrder, index: Index) -> Self {
        Self {
            order,
            index,
            render_details: Default::default(),
        }
    }
}
