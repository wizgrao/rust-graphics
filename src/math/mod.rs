use std::ops;
use std::ops::Neg;

pub const EPS: f64 = 1e-4;

#[derive(Clone, Copy, Debug)]
pub struct V3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct M3 {
    pub v0: V3,
    pub v1: V3,
    pub v2: V3,
}

impl M3 {
    pub fn t(&self) -> M3 {
        M3 {
            v0: v(self.v0.x, self.v1.x, self.v2.x),
            v1: v(self.v0.y, self.v1.y, self.v2.y),
            v2: v(self.v0.z, self.v1.z, self.v2.z),
        }
    }

    pub fn new(v0: V3, v1: V3, v2: V3) -> M3 {
        M3 { v0, v1, v2 }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub mat: M3,
    pub trans: V3,
}

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub x: V3,
    pub d: V3,
}

pub fn transform_ray(t: Transform, r: &Ray) -> Ray {
    Ray {
        x: t.mat * r.x + t.trans,
        d: t.mat * r.d,
    }
}

pub fn jitter_ray(r: Ray) -> Ray {
    Ray {
        x: r.x + 1. * EPS * r.d,
        d: r.d,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Plane {
    pub x: V3,
    pub n: V3,
    pub s: V3,
}

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub v0: V3,
    pub v1: V3,
    pub v2: V3,
}

impl Intersectable for Triangle {
    // https://cs184.eecs.berkeley.edu/sp24/lecture/9-20/ray-tracing-and-acceleration-str
    fn intersect(&self, r: &Ray) -> Option<Intersection> {
        //dbg!("in intersection method!");
        let e1 = self.v1 - self.v0;
        let e2 = self.v2 - self.v0;
        let s = r.x - self.v0;
        let s1 = cross(&r.d, &e2);
        let s2 = cross(&s, &e1);
        let coeff = 1.0 / dot(&s1, &e1);
        let t = coeff * dot(&s2, &e2);
        let b1 = coeff * dot(&s1, &s);
        let b2 = coeff * dot(&s2, &r.d);
        if b1 < 0.0 || b2 < 0.0 || b1 + b2 > 1.0 || t < 0. {
            return None;
        }
        Some(Intersection {
            x: r.x + t * r.d,
            n: normalize(&cross(&e1, &e2)),
            s: normalize(&e1),
            t,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub x: V3,
    pub r: f64,
}

#[derive(Clone, Debug)]
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
                n: self.n,
                s: self.s,
                t,
            })
        }
    }
}

impl Intersectable for Sphere {
    fn intersect(&self, r: &Ray) -> Option<Intersection> {
        let l = self.x - r.x;
        let tc = dot(&l, &r.d);

        if tc < 0.0 {
            return None;
        }

        let d2 = -(tc * tc) + dot(&l, &l);

        let radius2 = self.r * self.r;
        if d2 > radius2 {
            return None;
        }

        //solve for t1c
        let t1c = (radius2 - d2).sqrt();

        //solve for intersection points
        let t1 = match tc - t1c > 0. {
            true => tc - t1c,
            _ => tc + t1c,
        };

        let new_x = r.x + t1 * r.d;
        let n_unnormalized = new_x - self.x;
        let n_normalized = normalize(&n_unnormalized);
        let s_unnormalized = if n_normalized.z * n_normalized.z < 0.95 {
            v(n_unnormalized.y, -n_unnormalized.x, 0.0)
        } else {
            v(0.0, n_unnormalized.z, -n_unnormalized.y)
        };

        Some(Intersection {
            x: new_x,
            n: n_normalized,
            s: normalize(&s_unnormalized),
            t: t1,
        })
    }
}

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

impl ops::Add<V3> for V3 {
    type Output = V3;

    fn add(self, rhs: V3) -> V3 {
        add(&self, &rhs)
    }
}

impl ops::Sub<V3> for V3 {
    type Output = V3;

    fn sub(self, rhs: V3) -> V3 {
        sub(&self, &rhs)
    }
}

impl ops::Mul<V3> for f64 {
    type Output = V3;

    fn mul(self, rhs: V3) -> Self::Output {
        mul(self, &rhs)
    }
}

impl ops::Mul<V3> for V3 {
    type Output = V3;

    fn mul(self, rhs: V3) -> Self::Output {
        v(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl ops::Mul<V3> for M3 {
    type Output = V3;

    fn mul(self, rhs: V3) -> Self::Output {
        rhs.x * self.v0 + rhs.y * self.v1 + rhs.z * self.v2
    }
}

impl ops::Mul<M3> for M3 {
    type Output = M3;

    fn mul(self, rhs: M3) -> Self::Output {
        M3 {
            v0: self * rhs.v0,
            v1: self * rhs.v1,
            v2: self * rhs.v2,
        }
    }
}

impl Neg for V3 {
    type Output = Self;
    fn neg(self) -> V3 {
        -1. * self
    }
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

pub const I: M3 = M3 {
    v0: B1,
    v1: B2,
    v2: B3,
};

pub const O: V3 = V3 {
    x: 0.,
    y: 0.,
    z: 0.,
};
