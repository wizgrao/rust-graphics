use crate::math;
use crate::math::Intersection;
use rand::distributions::uniform::UniformFloat;
use rand::distributions::Standard;
use rand::{thread_rng, Rng};

const EPS: f64 = 1e-4;
const TERMINATION_P: f64 = 0.2;

pub trait BSDF {
    fn sample_wi(&self, wo: math::V3) -> (f64, math::V3);
    fn bsdf(&self, wo: math::V3, wi: math::V3) -> math::V3;
    fn radiance(&self, wo: math::V3) -> math::V3;
}
type IntersectionWithBSDF = Box<(Intersection, dyn BSDF)>;
pub trait Object {
    fn intersect(&self, r: &math::Ray) -> Option<IntersectionWithBSDF>;
}

pub fn estimated_total_radiance(o: &impl Object, r: &math::Ray) -> math::V3 {
    return estimated_at_least_one_bounce_radiance(o, r)
        + estimated_at_least_one_bounce_radiance(o, r);
}

pub fn estimated_zero_bounce_radiance(
    o: &impl Object,
    r: &math::Ray,
    p: IntersectionWithBSDF,
) -> math::V3 {
    return (*p).1.radiance(r.d);
}

pub fn bounce(r: &math::Ray, intersection: &Intersection, p: IntersectionWithBSDF) -> math::Ray {
    let o2w = math::M3 {
        v0: intersection.s,
        v1: math::cross(&intersection.n, &intersection.s),
        v2: intersection.n,
    };
    let w2o = o2w.t();
    let d_o = w2o * r.d;
    let do_bounce = math::v(d_o.x, d_o.y, -d_o.z);
    let d_bounce = o2w * do_bounce;
    math::Ray {
        x: intersection.x + EPS * d_bounce,
        d: d_bounce,
    }
}

pub fn estimated_one_bounce_radiance(
    o: &impl Object,
    r: &math::Ray,
    p: IntersectionWithBSDF,
) -> math::V3 {
}

pub fn estimated_at_least_one_bounce_radiance(o: &impl Object, r: &math::Ray) -> math::V3 {
    let one_bounce = estimated_one_bounce_radiance(o, r);
    let thresh: f64 = thread_rng().sample(Standard);
    if thresh < TERMINATION_P {
        return one_bounce;
    }
    let isect = o.intersect(r);
    match isect {
        None => one_bounce,
        Some(b) => {
            let intersection = (*b).0;
            let bounced = bounce(r, &intersection);
            return one_bounce
                + 1.0 / (1.0 - TERMINATION_P)
                    * estimated_at_least_one_bounce_radiance(o, &bounced);
        }
    }
}
