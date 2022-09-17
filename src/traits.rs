use crate::system::System;

pub trait GetPos {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}

macro_rules! impl_get_pos {
    ($x:ty) => {
        impl GetPos for $x {
            fn x(&self) -> f32 {
                self.0
            }
            fn y(&self) -> f32 {
                self.1
            }
        }
    };
}

impl_get_pos!((f32, f32));
impl_get_pos!(&(f32, f32));

pub trait GetDimension {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

macro_rules! impl_get_dimension {
    ($x:ty) => {
        /// This is a tuple of position. Therefore there is no width or height.
        impl GetDimension for $x {
            fn width(&self) -> u32 {
                0
            }
            fn height(&self) -> u32 {
                0
            }
        }
    };
}

impl_get_dimension!((f32, f32));
impl_get_dimension!(&(f32, f32));
impl_get_dimension!((u32, u32));
impl_get_dimension!(&(u32, u32));

pub trait Draw: GetPos {
    fn draw(&mut self, system: &mut System, debug: bool);
}
