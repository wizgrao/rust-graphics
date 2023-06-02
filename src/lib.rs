use crate::math::V3;

pub mod marcher;
pub mod math;
pub mod path_tracer;

pub trait Scene {
    fn sdf(&self, x: &V3) -> f32;
}
