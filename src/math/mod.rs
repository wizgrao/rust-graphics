#[derive(Clone)]
pub struct V3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone)]
pub struct Ray {
    pub x: V3,
    pub d: V3,
}

#[derive(Clone)]
pub struct Plane {
    pub x: V3,
    pub n: V3,
    pub s: V3,
}

#[derive(Clone)]
pub struct Sphere {
    pub x: V3,
    pub r: f64,
}

#[derive(Clone)]
pub struct Intersection {
    pub x: V3,
    pub n: V3,
    pub s: V3,
    pub t: f64,
}

pub trait Intersectable {
    fn intersect(&self, r: &Ray) -> Option<Intersection>;
}

impl Intersectable for Plane {
    fn intersect(&self, r: &Ray) -> Option<Intersection> {
        let t = (dot(&self.n, &self.x) - dot(&self.n, &r.x)) / dot(&self.n, &r.d);
        if t < 0.0 {
            None
        } else {
            Some(Intersection {
                x: add(&r.x, &mul(t, &r.d)),
                n: self.n.clone(),
                s: self.s.clone(),
                t,
            })
        }
    }
}

impl Intersectable for 

pub fn sub(x: &V3, y: &V3) -> V3 {
    V3 {
        x: x.x - y.x,
        y: x.y - y.y,
        z: x.z - y.z,
    }
}

pub fn abs2(x: &V3) -> f64 {
    x.x * x.x + x.y * x.y + x.z * x.z
}

pub fn abs(x: &V3) -> f64 {
    abs2(x).sqrt()
}
pub fn v(x: f64, y: f64, z: f64) -> V3 {
    V3 { x, y, z }
}
pub fn mul(scalar: f64, x: &V3) -> V3 {
    V3 {
        x: x.x * scalar,
        y: x.y * scalar,
        z: x.z * scalar,
    }
}

pub fn add(x: &V3, y: &V3) -> V3 {
    V3 {
        x: x.x + y.x,
        y: x.y + y.y,
        z: x.z + y.z,
    }
}

pub fn dist(x: &V3, y: &V3) -> f64 {
    abs(&sub(x, y))
}

pub fn normalize(x: &V3) -> V3 {
    mul(1. / abs(x), x)
}

pub fn dot(x: &V3, y: &V3) -> f64 {
    x.x * y.x + x.y * y.y + x.z * y.z
}

pub fn cross(v1: &V3, v2: &V3) -> V3 {
    v(
        v1.y * v2.z - v1.z * v2.y,
        v1.z * v2.x - v1.x * v2.z,
        v1.x * v2.y - v1.y * v2.x,
    )
}

pub const B1: V3 = V3 {
    x: 1.,
    y: 0.,
    z: 0.,
};

pub const B2: V3 = V3 {
    x: 0.,
    y: 1.,
    z: 0.,
};

pub const B3: V3 = V3 {
    x: 0.,
    y: 0.,
    z: 1.,
};

pub const O: V3 = V3 {
    x: 0.,
    y: 0.,
    z: 0.,
};
