use crate::math::{abs, add, dist, dot, mul, normalize, sub, Ray, B1, B2, B3, O, V3};
#[derive(Default)]
pub struct Cup {
    renderables: Vec<Box<dyn Renderable>>,
}

pub struct Cap {
    pub renderables: Vec<Box<dyn Renderable>>,
}

pub struct Sphere {
    pub center: V3,
    pub radius: f64,
}

pub struct Torus {
    pub center: V3,
    pub axis: V3,
    pub big_radius: f64,
    pub small_radius: f64,
}

pub struct Plane {
    pub point: V3,
    pub axis: V3,
}

pub trait Renderable {
    fn sdf(&self, x: &V3) -> f64;
}

impl Renderable for Sphere {
    fn sdf(&self, x: &V3) -> f64 {
        abs(&sub(&x, &self.center)) - self.radius
    }
}

impl Renderable for Cup {
    fn sdf(&self, x: &V3) -> f64 {
        match self
            .renderables
            .iter()
            .min_by(|xx, yy| (*xx).sdf(x).total_cmp(&(*yy).sdf(x)))
        {
            None => 0.,
            Some(y) => (*y).sdf(x),
        }
    }
}

impl Renderable for Cap {
    fn sdf(&self, x: &V3) -> f64 {
        match self
            .renderables
            .iter()
            .max_by(|xx, yy| (*xx).sdf(x).total_cmp(&(*yy).sdf(x)))
        {
            None => 0.,
            Some(y) => (*y).sdf(x),
        }
    }
}

impl Renderable for Torus {
    fn sdf(&self, x: &V3) -> f64 {
        let rel_pos = sub(x, &self.center); //v relPos = sub(x, t->center);
        let axis_proj = mul(dot(&rel_pos, &self.axis), &self.axis); // axisProj = proj(relPos, t->axis);
        let plane_proj = sub(&rel_pos, &axis_proj); //v planeProj = sub(relPos, axisProj);
        let circle_proj = mul(self.big_radius, &normalize(&plane_proj)); // v circleProj = scale(t->br, vnormalize(planeProj));
        dist(&circle_proj, &rel_pos) - self.small_radius
    }
}

impl Renderable for Plane {
    fn sdf(&self, x: &V3) -> f64 {
        dot(&sub(x, &self.point), &self.axis)
    }
}

pub fn render(s: &impl Renderable, loc: &V3) -> f64 {
    let normalized_loc = normalize(&loc);
    let light = normalize(&V3 {
        x: 1.,
        y: 1.,
        z: 1.5,
    });

    match intersect(s, &O, &normalized_loc) {
        Some(Ray { d, .. }) => (-dot(&d, &light)).clamp(0.1, 0.995),
        _ => 0.,
    }
}

fn intersect(r: &impl Renderable, x: &V3, dir: &V3) -> Option<Ray> {
    let mut y = x.clone();
    for _ in 0..1000 {
        let sdf = r.sdf(&y);
        if sdf < EPS {
            return Some(Ray {
                d: normalize(&dsdf(r, &y)),
                x: y,
            });
        }
        y = add(&y, &mul(sdf, dir));
        if abs(&y) > 10. {
            return None;
        }
    }
    None
}
const EPS: f64 = 1e-5;

fn dsdf(r: &impl Renderable, x: &V3) -> V3 {
    let dx = mul(EPS, &B1);
    let dy = mul(EPS, &B2);
    let dz = mul(EPS, &B3);

    let x_plus_dx = add(&dx, &x);
    let x_plus_dy = add(&dy, &x);
    let x_plus_dz = add(&dz, &x);

    let dsdx = (r.sdf(&x_plus_dx) - r.sdf(x)) / EPS;
    let dsdy = (r.sdf(&x_plus_dy) - r.sdf(x)) / EPS;
    let dsdz = (r.sdf(&x_plus_dz) - r.sdf(x)) / EPS;

    V3 {
        x: dsdx,
        y: dsdy,
        z: dsdz,
    }
}
