use std::any::Any;

pub enum Value {
    Float32(f32),
    Float64(f64),
    UnsignedInteger(usize),
    Integer(isize),
    String(String),
    Float32Range(f32, f32),
    Float64Range(f64, f64),
    UnsignedIntegerRange(usize, usize),
    IntegerRange(isize, isize),
    Generic(Box<dyn Any>),
}
