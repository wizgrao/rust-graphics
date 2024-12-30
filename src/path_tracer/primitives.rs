use crate::math;
use crate::math::{Intersectable, Ray, Sphere, Triangle, V3};
use crate::path_tracer::bvh;
use crate::path_tracer::bvh::BVHNode;
use crate::path_tracer::obj::{FaceVertex, ObjLine};
use crate::path_tracer::{
    sample_hemisphere, sample_sphere, IntersectionWithBSDF, Light, Object, Photon, BSDF,
};
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::f32::consts::PI;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct Lambertian {
    pub reflectance: V3,
}

#[derive(Clone, Copy, Debug)]
pub struct Emissive {
    pub emission: V3,
}

impl<B: BSDF> bvh::Bounded for Solid<B, Triangle> {
    fn get_bounds(&self) -> (V3, V3) {
        let t = &self.intersectable;
        let min = bvh::calculate_min(bvh::calculate_min(t.v1, t.v2), t.v0);
        let max = bvh::calculate_max(bvh::calculate_max(t.v1, t.v2), t.v0);
        (min, max)
    }
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

pub fn obj_to_solid<B: BSDF + 'static>(
    objs: &Vec<ObjLine>,
    bsdf: Arc<B>,
) -> BVHNode<Solid<B, Triangle>> {
    bvh::BVHNode::new(
        obj_to_triangles(objs)
            .into_iter()
            .map(|t| Solid {
                bsdf: bsdf.clone(),
                intersectable: Arc::new(t),
            })
            .collect(),
    )
}

pub fn obj_to_cup_solid<B: BSDF + 'static>(objs: &Vec<ObjLine>, bsdf: Arc<B>) -> Cup {
    Cup {
        objects: obj_to_triangles(objs)
            .into_iter()
            .map(|t| {
                Arc::new(Solid {
                    bsdf: bsdf.clone(),
                    intersectable: Arc::new(t),
                }) as Arc<dyn Object>
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

#[derive(Debug)]
pub struct Solid<B: BSDF, I: Intersectable> {
    pub bsdf: Arc<B>,
    pub intersectable: Arc<I>,
}
impl<B: BSDF + 'static, I: Intersectable> Object for Solid<B, I> {
    fn intersect(&self, r: &Ray) -> Option<IntersectionWithBSDF> {
        self.intersectable
            .intersect(r)
            .map(|intersection| (intersection, self.bsdf.clone() as Arc<dyn BSDF>))
    }
}

pub struct TransformedObject<O: Object> {
    pub wrapped: Arc<O>,
    pub transform: math::Transform,
}

impl<O: Object> TransformedObject<O> {
    pub fn new(wrapped: Arc<O>, transform: math::Transform) -> Self {
        Self { wrapped, transform }
    }
}

impl<O: Object> Object for TransformedObject<O> {
    fn intersect(&self, r: &Ray) -> Option<IntersectionWithBSDF> {
        self.wrapped
            .intersect(&math::transform_ray(self.transform, r))
    }
}

pub struct Cup {
    pub objects: Vec<Arc<dyn Object>>,
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
