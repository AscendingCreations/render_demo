use std::any::Any;

pub enum Value {
    Float32(f32),
    Float64(f64),
    UnsignedInteger(usize),
    Integer(isize),
    String(String),
    Generic(Box<dyn Any>),
}
