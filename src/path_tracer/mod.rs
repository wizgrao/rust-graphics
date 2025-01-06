use crate::math::{Intersection, Ray, M3};
use crate::{math, V3};
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use rand_distr::StandardNormal;
use std::f32::consts::PI;
use std::sync::Arc;
pub mod bvh;
pub mod obj;
pub mod primitives;

pub struct RenderContext {
    pub imp: bool,
    pub max_bounces: i32,
    pub termination_p: f64,
    pub light_samples: i32,
    pub preview: bool,
}

pub trait BSDF {
    fn sample_wi(&self, wo: V3) -> (f64, V3);
    fn bsdf(&self, wo: math::V3, wi: math::V3) -> math::V3;
    fn radiance(&self, wo: math::V3) -> math::V3;
}

pub struct Scene {
    pub object: Box<dyn Object>,
    pub light: Box<dyn Light>,
}

fn sample_hemisphere() -> (f64, V3) {
    let x = thread_rng().sample(StandardNormal);
    let y = thread_rng().sample(StandardNormal);
    let z: f64 = thread_rng().sample(StandardNormal);
    let z_pos = if z > 0.0 { z } else { -z };
    (
        (1. / (2. * PI)) as f64,
        math::normalize(&math::v(x, y, z_pos)),
    )
}

fn sample_disk() -> V3 {
    loop {
        let x: f64 = thread_rng().sample(Standard);
        let y: f64 = thread_rng().sample(Standard);
        let v = math::v(x, y, 0.);
        if math::abs2(&v) <= 1. {
            return v;
        }
    }
}

fn sample_sphere() -> V3 {
    let x = thread_rng().sample(StandardNormal);
    let y = thread_rng().sample(StandardNormal);
    let z: f64 = thread_rng().sample(StandardNormal);
    math::normalize(&math::v(x, y, z))
}

type IntersectionWithBSDF = (Intersection, Arc<dyn BSDF>);
pub trait Object {
    fn intersect(&self, r: &Ray) -> Option<IntersectionWithBSDF>;
}

pub struct Photon {
    pub d: Ray,
    pub radiance: V3,
}

#[derive(Clone, Debug)]
pub struct Camera {
    pub lens_origin: V3,
    pub sensor_origin: V3,
    pub sensor_x: V3,
    pub sensor_y: V3,
    pub lens_direction: V3,
    pub lens_radius: f64,
    pub focal_length: f64,
}

impl Camera {
    pub fn new(
        lens_origin: V3,
        lens_direction: V3,
        lens_radius: f64,
        focal_length: f64,
        focus_distance: f64,
        sensor_x: V3,
        sensor_y: V3,
    ) -> Self {
        Self {
            lens_origin,
            sensor_origin: lens_origin
                - (1.0 / (1. / focal_length - 1. / focus_distance)) * lens_direction,
            sensor_x,
            sensor_y,
            lens_direction,
            lens_radius,
            focal_length,
        }
    }
    fn get_lens_basis(&self) -> M3 {
        let basis = if math::dot(&math::B1, &self.lens_direction).abs() < 0.9 {
            math::B1
        } else {
            math::B2
        };
        let camera_b1 = math::normalize(
            &(basis - math::dot(&basis, &self.lens_direction) * self.lens_direction),
        );
        let camera_b2 = math::cross(&camera_b1, &self.lens_direction);
        M3::new(camera_b1, camera_b2, self.lens_direction)
    }
    fn sample_lens_point(&self) -> V3 {
        let disk_point = sample_disk();
        self.lens_radius * (self.get_lens_basis() * disk_point) + self.lens_origin
    }

    fn get_lens_ray(&self, sensor_point: V3, lens_point: V3) -> Ray {
        let l2w = self.get_lens_basis();
        let w2l = l2w.t();
        let normalized_sensor_point = sensor_point - self.lens_origin;
        let normalized_lens_point = lens_point - self.lens_origin;
        let sensor_point_lens_space = w2l * normalized_sensor_point;
        let lens_point_lens_space = w2l * normalized_lens_point;
        let sensor_lens_dir = lens_point_lens_space - sensor_point_lens_space;
        let focal_lens_dir = self.focal_length / sensor_lens_dir.z * sensor_lens_dir;
        let focal_plane_point = sensor_point_lens_space - focal_lens_dir;
        let ray_dir = l2w * math::normalize(&(-focal_plane_point));
        Ray {
            x: lens_point,
            d: ray_dir,
        }
    }

    fn get_sensor_point(&self, x: f64, y: f64) -> V3 {
        self.sensor_origin + x * self.sensor_x + y * self.sensor_y
    }

    pub fn sample_ray(&self, x: f64, y: f64) -> Ray {
        self.get_lens_ray(self.get_sensor_point(x, y), self.sample_lens_point())
    }
}

pub trait Light {
    fn sample_rad(&self, p: V3) -> (f64, Photon);
}

pub fn estimated_total_radiance(ctx: &RenderContext, o: &Scene, r: &Ray) -> V3 {
    match o.object.intersect(r) {
        Some(p) => {
            if ctx.preview {
                normalize_elems(math::normalize(&p.0.n))
            } else {
                estimated_zero_bounce_radiance(r, &p)
                    + estimated_at_least_one_bounce_radiance(ctx, o, r, &p, 0)
            }
        }
        None => math::O,
    }
}

fn normalize_elems(s: V3) -> V3 {
    math::v(abs1(s.x), abs1(s.y), abs1(s.z))
}

fn abs1(x: f64) -> f64 {
    if x > 0. {
        x
    } else {
        -x
    }
}

fn estimated_zero_bounce_radiance(r: &Ray, p: &IntersectionWithBSDF) -> V3 {
    let (_o2w, w2o) = object_world_matrices_from_intersection(&p.0);
    p.1.radiance(w2o * r.d)
}

fn object_world_matrices_from_intersection(intersection: &Intersection) -> (math::M3, math::M3) {
    let o2w = math::M3 {
        v0: intersection.s,
        v1: math::cross(&intersection.n, &intersection.s),
        v2: intersection.n,
    };
    let w2o = o2w.t();
    (o2w, w2o)
}

fn _bounce(r: &Ray, intersection: &Intersection) -> Ray {
    let (o2w, w2o) = object_world_matrices_from_intersection(intersection);
    let d_o = w2o * r.d;
    let do_bounce = math::v(d_o.x, d_o.y, -d_o.z);
    let d_bounce = o2w * do_bounce;
    Ray {
        x: intersection.x + math::EPS * d_bounce,
        d: d_bounce,
    }
}

fn estimated_one_bounce_radiance(
    _ctx: &RenderContext,
    s: &Scene,
    r: &Ray,
    p: &IntersectionWithBSDF,
) -> V3 {
    let o = &s.object;
    let (intersection, bsdf) = p;
    let o2w = math::M3 {
        v0: intersection.s,
        v1: math::cross(&intersection.n, &intersection.s),
        v2: intersection.n,
    };
    let w2o = o2w.t();
    let d_o = w2o * r.d;

    let (pdf, wi_o) = (*bsdf).sample_wi(d_o);
    let reflection = (*bsdf).bsdf(d_o, wi_o);
    let wi_w = o2w * wi_o;
    let starting_point = intersection.x + math::EPS * wi_w;
    let new_ray = Ray {
        x: starting_point,
        d: wi_w,
    };
    match o.intersect(&new_ray) {
        None => math::O,
        Some(new_p) => {
            1. / pdf * wi_o.z * estimated_zero_bounce_radiance(&new_ray, &new_p) * reflection
        }
    }
}

fn estimated_one_bounce_radiance_imp(
    ctx: &RenderContext,
    s: &Scene,
    r: &Ray,
    p: &IntersectionWithBSDF,
) -> V3 {
    let (intersection, bsdf) = p;
    let o2w = math::M3 {
        v0: intersection.s,
        v1: math::cross(&intersection.n, &intersection.s),
        v2: intersection.n,
    };
    let w2o = o2w.t();
    let d_o = w2o * r.d;

    let mut light_sum = math::O;

    for _ in 0..ctx.light_samples {
        let (light_pdf, photon_sample) = s.light.sample_rad(intersection.x);
        let shadow_ray = math::jitter_ray(photon_sample.d);

        let mut obj_cos = math::dot(&intersection.n, &(-1.0 * photon_sample.d.d));
        if obj_cos < 0.0 {
            obj_cos = -obj_cos
        }

        if let Some((i, _)) = s.object.intersect(&shadow_ray) {
            if math::dist(&i.x, &intersection.x) > 1. * math::EPS {
                continue;
            }
        }

        let reflection = (*bsdf).bsdf(d_o, -1.0 * (w2o * photon_sample.d.d));
        let d2 = math::abs2(&(intersection.x - photon_sample.d.x));
        light_sum = light_sum + (obj_cos / (light_pdf * d2)) * photon_sample.radiance * reflection;
    }
    1.0 / (ctx.light_samples as f64) * light_sum
}

fn estimated_at_least_one_bounce_radiance(
    ctx: &RenderContext,
    s: &Scene,
    r: &Ray,
    p: &IntersectionWithBSDF,
    bounce: i32,
) -> V3 {
    if bounce >= ctx.max_bounces {
        return math::O;
    }
    let o = &s.object;
    let one_bounce = (if ctx.imp {
        estimated_one_bounce_radiance_imp
    } else {
        estimated_one_bounce_radiance
    })(ctx, s, r, p);

    let thresh: f64 = thread_rng().sample(Standard);
    if thresh < ctx.termination_p {
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
    let reflection = (*bsdf).bsdf(d_o, wi_o);
    let wi_w = o2w * wi_o;
    let starting_point = intersection.x + math::EPS * wi_w;
    let new_ray = Ray {
        x: starting_point,
        d: wi_w,
    };
    match o.intersect(&new_ray) {
        None => one_bounce,
        Some(new_p) => {
            1. / pdf / (1. - ctx.termination_p)
                * wi_o.z
                * estimated_at_least_one_bounce_radiance(ctx, s, &new_ray, &new_p, bounce + 1)
                * reflection
                + one_bounce
        }
    }
}
