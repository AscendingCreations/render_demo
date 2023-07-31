use crate::Allocator;

pub struct Layer {
    pub allocator: Allocator,
}

impl Layer {
    pub fn new(size: u32) -> Self {
        Self {
            allocator: Allocator::new(size),
        }
    }
}
