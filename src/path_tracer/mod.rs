use crate::math::{Intersectable, Intersection, Ray};
use crate::math::{Sphere, Triangle};
use crate::{math, V3};
use rand::distributions::uniform::Uniform;
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use rand_distr::StandardNormal;
use std::f32::consts::PI;
use std::sync::Arc;
pub mod obj;

use obj::*;

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

pub fn obj_to_triangles(objs: &Vec<ObjLine>) -> Vec<Triangle> {
    let mut triangles: Vec<Triangle> = Vec::new();
    let mut vertices: Vec<V3> = Vec::new();
    for obj_line in objs {
        match obj_line {
            ObjLine::Vertex(x, y, z) => vertices.push(V3 {
                x: *x,
                y: *y,
                z: *z,
            }),
            ObjLine::Face(i, j, k) => triangles.push(math::Triangle {
                v0: vertices[get_index_from_face(*i) - 1],
                v1: vertices[get_index_from_face(*j) - 1],
                v2: vertices[get_index_from_face(*k) - 1],
            }),
            _ => {}
        }
    }
    triangles
}

pub fn obj_to_solid(objs: &Vec<ObjLine>, bsdf: Arc<dyn BSDF>) -> Cup {
    Cup {
        objects: obj_to_triangles(objs)
            .into_iter()
            .map(|t| {
                Box::new(Solid {
                    bsdf: bsdf.clone(),
                    intersectable: Arc::new(t),
                }) as Box<dyn Object>
            })
            .collect(),
    }
}

fn get_index_from_face(f: FaceVertex) -> usize {
    match f {
        FaceVertex::Vertex(i) => i as usize,
        FaceVertex::VertexNormal(i, _) => i as usize,
        FaceVertex::VertexTexture(i, _) => i as usize,
        FaceVertex::VertexTextureNormal(i, _, _) => i as usize,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Lambertian {
    pub reflectance: V3,
}

#[derive(Clone, Copy, Debug)]
pub struct Emissive {
    pub emission: V3,
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

fn sample_sphere() -> V3 {
    let x = thread_rng().sample(StandardNormal);
    let y = thread_rng().sample(StandardNormal);
    let z: f64 = thread_rng().sample(StandardNormal);
    math::normalize(&math::v(x, y, z))
}

impl BSDF for Lambertian {
    fn sample_wi(&self, _wo: V3) -> (f64, V3) {
        sample_hemisphere()
    }

    fn bsdf(&self, _wo: V3, _wi: V3) -> V3 {
        (1. / PI as f64) * self.reflectance
    }

    fn radiance(&self, _wo: V3) -> V3 {
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

pub struct Photon {
    pub d: Ray,
    pub radiance: V3,
}

pub trait Light {
    fn sample_rad(&self, p: V3) -> (f64, Photon);
}

impl Object for Solid {
    fn intersect(&self, r: &Ray) -> Option<IntersectionWithBSDF> {
        self.intersectable
            .intersect(r)
            .map(|intersection| (intersection, self.bsdf.clone()))
    }
}

pub struct Cup {
    pub objects: Vec<Box<dyn Object>>,
}

pub struct TransformedObject<O: Object> {
    pub wrapped: O,
    pub transform: math::Transform,
}

impl<O: Object> TransformedObject<O> {
    pub fn new(wrapped: O, transform: math::Transform) -> Self {
        Self { wrapped, transform }
    }
}

impl<O: Object> Object for TransformedObject<O> {
    fn intersect(&self, r: &Ray) -> Option<IntersectionWithBSDF> {
        self.wrapped
            .intersect(&math::transform_ray(self.transform, r))
    }
}

impl Object for Cup {
    fn intersect(&self, r: &Ray) -> Option<IntersectionWithBSDF> {
        let mut ret: Option<IntersectionWithBSDF> = None;
        for object in self.objects.iter() {
            let intersection = object.intersect(r);
            match (&ret, &intersection) {
                (Some((ret_intersection, _)), Some((new_intersection, _))) => {
                    if ret_intersection.t > new_intersection.t {
                        ret = intersection;
                    }
                }
                (None, Some(_)) => ret = intersection,
                _ => {}
            }
        }
        ret
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SphereLight {
    pub sphere: Sphere,
    pub e: Emissive,
}

impl Light for SphereLight {
    fn sample_rad(&self, p: V3) -> (f64, Photon) {
        let v = sample_sphere();
        let light_surface_point = self.sphere.r * v + self.sphere.x;
        let dir = math::normalize(&(p - light_surface_point));
        let mut cos_dir = math::dot(&dir, &v);
        if cos_dir < 0.0 {
            cos_dir = 0.0;
        }

        (
            1.0 / (4.0 * PI as f64 * self.sphere.r * self.sphere.r),
            Photon {
                d: math::Ray {
                    d: dir,
                    x: light_surface_point,
                },
                radiance: cos_dir * self.e.emission,
            },
        )
    }
}

pub struct CupLight {
    pub lights: Vec<Box<dyn Light>>,
}

impl Light for CupLight {
    fn sample_rad(&self, p: V3) -> (f64, Photon) {
        let num_lights = self.lights.len();
        if num_lights == 0 {
            return (
                0.,
                Photon {
                    d: Ray {
                        x: math::O,
                        d: math::O,
                    },
                    radiance: math::O,
                },
            );
        }
        let index = thread_rng().sample(Uniform::new(0, num_lights));
        let light = &self.lights[index];
        let (pdf, photon) = light.sample_rad(p);
        (pdf / (num_lights as f64), photon)
    }
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
