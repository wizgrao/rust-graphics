use crate::math::{Intersectable, Intersection, Ray};
use crate::{math, V3};
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use rand_distr::StandardNormal;
use std::f32::consts::PI;
use std::sync::Arc;

const EPS: f64 = 1e-4;
const TERMINATION_P: f64 = 0.2;

pub trait BSDF {
    fn sample_wi(&self, wo: V3) -> (f64, V3);
    fn bsdf(&self, wo: math::V3, wi: math::V3) -> math::V3;
    fn radiance(&self, wo: math::V3) -> math::V3;
}

pub struct Lambertian {
    pub reflectance: V3,
}

pub struct Emissive {
    pub emission: V3,
}

fn sample_hemisphere() -> (f64, V3) {
    let x = thread_rng().sample(StandardNormal);
    let y = thread_rng().sample(StandardNormal);
    let z: f64 = thread_rng().sample(StandardNormal);
    let z_pos = if z > 0.0 { z } else { -z };
    return (
        (1. / (2. * PI)) as f64,
        math::normalize(&math::v(x, y, z_pos)),
    );
}

impl BSDF for Lambertian {
    fn sample_wi(&self, _wo: V3) -> (f64, V3) {
        sample_hemisphere()
    }

    fn bsdf(&self, _wo: V3, _wi: V3) -> V3 {
        (1. / PI as f64) * self.reflectance
    }

    fn radiance(&self, wo: V3) -> V3 {
        math::O
    }
}

impl BSDF for Emissive {
    fn sample_wi(&self, _wo: V3) -> (f64, V3) {
        sample_hemisphere()
    }

    fn bsdf(&self, _wo: V3, _wi: V3) -> V3 {
        math::O
    }

    fn radiance(&self, _wo: V3) -> V3 {
        self.emission
    }
}

type IntersectionWithBSDF = (Intersection, Arc<dyn BSDF>);
pub trait Object {
    fn intersect(&self, r: &Ray) -> Option<IntersectionWithBSDF>;
}

pub struct Solid {
    pub bsdf: Arc<dyn BSDF>,
    pub intersectable: Arc<dyn Intersectable>,
}

impl Object for Solid {
    fn intersect(&self, r: &Ray) -> Option<IntersectionWithBSDF> {
        match self.intersectable.intersect(r) {
            None => None,
            Some(intersection) => Some((intersection, self.bsdf.clone())),
        }
    }
}

pub fn estimated_total_radiance(o: &impl Object, r: &Ray) -> V3 {
    match o.intersect(r) {
        Some(p) => {
            estimated_zero_bounce_radiance(r, &p) + estimated_at_least_one_bounce_radiance(o, r, &p)
        }
        None => math::O,
    }
}

fn estimated_zero_bounce_radiance(r: &Ray, p: &IntersectionWithBSDF) -> V3 {
    let (_o2w, w2o) = object_world_matrices_from_intersection(&p.0);
    return p.1.radiance(w2o * r.d);
}

fn object_world_matrices_from_intersection(intersection: &Intersection) -> (math::M3, math::M3) {
    let o2w = math::M3 {
        v0: intersection.s,
        v1: math::cross(&intersection.n, &intersection.s),
        v2: intersection.n,
    };
    let w2o = o2w.t();
    return (o2w, w2o);
}

fn bounce(r: &math::Ray, intersection: &Intersection) -> Ray {
    let (o2w, w2o) = object_world_matrices_from_intersection(intersection);
    let d_o = w2o * r.d;
    let do_bounce = math::v(d_o.x, d_o.y, -d_o.z);
    let d_bounce = o2w * do_bounce;
    Ray {
        x: intersection.x + EPS * d_bounce,
        d: d_bounce,
    }
}

fn estimated_one_bounce_radiance(o: &impl Object, r: &Ray, p: &IntersectionWithBSDF) -> V3 {
    let (intersection, bsdf) = p;
    let o2w = math::M3 {
        v0: intersection.s,
        v1: math::cross(&intersection.n, &intersection.s),
        v2: intersection.n,
    };
    let w2o = o2w.t();
    let d_o = w2o * r.d;

    let (pdf, wi_o) = (*bsdf).sample_wi(d_o);
    let wi_w = o2w * wi_o;
    let starting_point = intersection.x + EPS * wi_w;
    let new_ray = Ray {
        x: starting_point,
        d: wi_w,
    };
    match o.intersect(&new_ray) {
        None => math::O,
        Some(new_p) => 1. / pdf * wi_o.z * estimated_zero_bounce_radiance(&new_ray, &new_p),
    }
}

fn estimated_at_least_one_bounce_radiance(
    o: &impl Object,
    r: &Ray,
    p: &IntersectionWithBSDF,
) -> V3 {
    let one_bounce = estimated_one_bounce_radiance(o, r, p);
    let thresh: f64 = thread_rng().sample(Standard);
    if thresh < TERMINATION_P {
        return one_bounce;
    }
    let (intersection, bsdf) = p;
    let o2w = math::M3 {
        v0: intersection.s,
        v1: math::cross(&intersection.n, &intersection.s),
        v2: intersection.n,
    };
    let w2o = o2w.t();
    let d_o = w2o * r.d;

    let (pdf, wi_o) = (**bsdf).sample_wi(d_o);
    let wi_w = o2w * wi_o;
    let starting_point = intersection.x + EPS * wi_w;
    let new_ray = Ray {
        x: starting_point,
        d: wi_w,
    };
    match o.intersect(&new_ray) {
        None => one_bounce,
        Some(new_p) => {
            1. / pdf / (1. - TERMINATION_P)
                * wi_o.z
                * estimated_at_least_one_bounce_radiance(o, &new_ray, &new_p)
                + one_bounce
        }
    }
}
