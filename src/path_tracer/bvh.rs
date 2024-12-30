use crate::math::{v, Ray, V3};
use crate::path_tracer;
#[derive(Debug)]
pub enum BVHItem<T: Bounded> {
    Leaf(T),
    Branch {
        left: Box<BVHNode<T>>,
        right: Box<BVHNode<T>>,
    },
}

#[derive(Debug)]
pub struct BVHNode<T: Bounded> {
    min: V3,
    max: V3,
    item: BVHItem<T>,
}

pub trait Bounded {
    fn get_bounds(&self) -> (V3, V3);
    fn get_midpoint(&self) -> V3 {
        let (min, max) = self.get_bounds();
        0.5 * (max + min)
    }
}

impl<T: Bounded> BVHNode<T> {
    pub fn new(items: Vec<T>) -> Self {
        let mut bvec = items as BoundedVec<T>;
        let (min, max) = bvec.get_bounds();
        if bvec.len() == 1 {
            BVHNode {
                min,
                max,
                item: BVHItem::Leaf(bvec.remove(0)),
            }
        } else {
            let size = max - min;
            let vec_indexer: fn(V3) -> f64 =
                match (size.x > size.y, size.x > size.z, size.y > size.z) {
                    (true, true, _) => |x| x.x,
                    (_, _, true) => |x| x.y,
                    _ => |x| x.z,
                };
            let mut left = Vec::new();
            let mut right = Vec::new();
            let mid = 0.5 * (max + min);
            for item in bvec.into_iter() {
                if vec_indexer(item.get_midpoint()) < vec_indexer(mid) {
                    left.push(item);
                } else {
                    right.push(item);
                }
            }
            if left.len() == 0 {
                left.push(right.pop().unwrap())
            }
            if right.len() == 0 {
                right.push(left.pop().unwrap())
            }
            BVHNode {
                min,
                max,
                item: BVHItem::Branch {
                    left: Box::new(BVHNode::new(left)),
                    right: Box::new(BVHNode::new(right)),
                },
            }
        }
    }
}

type BoundedVec<T> = Vec<T>;
impl<T: Bounded> Bounded for BoundedVec<T> {
    fn get_bounds(&self) -> (V3, V3) {
        let mut state: Option<(V3, V3)> = None;
        for item in self.iter() {
            match state {
                None => {
                    state = Some(item.get_bounds());
                }
                Some((min, max)) => {
                    let (new_min, new_max) = item.get_bounds();
                    state = Some((calculate_min(min, new_min), calculate_max(max, new_max)));
                }
            }
        }
        state.unwrap()
    }
}

impl<T: Bounded + path_tracer::Object> BVHNode<T> {
    fn inner_intersect(&self, r: &Ray) -> Option<path_tracer::IntersectionWithBSDF> {
        match &self.item {
            BVHItem::Leaf(t) => t.intersect(r),
            BVHItem::Branch { left, right } => {
                match (left.inner_intersect(r), right.inner_intersect(r)) {
                    (None, None) => None,
                    (None, Some(i)) => Some(i),
                    (Some(i), None) => Some(i),
                    (Some(i), Some(j)) => {
                        let ti = i.0.t;
                        let tj = j.0.t;
                        if ti < tj {
                            Some(i)
                        } else {
                            Some(j)
                        }
                    }
                }
            }
        }
    }
}

impl<T: Bounded + path_tracer::Object> path_tracer::Object for BVHNode<T> {
    fn intersect(&self, r: &Ray) -> Option<path_tracer::IntersectionWithBSDF> {
        //dbg!("in intersect");
        let x_interval = get_interval_from_linear(r.d.x, r.x.x, self.min.x, self.max.x);
        let y_interval = get_interval_from_linear(r.d.y, r.x.y, self.min.y, self.max.y);
        let z_interval = get_interval_from_linear(r.d.z, r.x.z, self.min.z, self.max.z);
        match intersect(intersect(x_interval, y_interval), z_interval) {
            Interval::Empty => {
                /*dbg!("empty interval!");*/
                None
            }
            _ => self.inner_intersect(r),
        }
    }
}

enum Interval {
    Empty,
    Bounds(f64, f64),
}

fn intersect(a: Interval, b: Interval) -> Interval {
    match (a, b) {
        (Interval::Bounds(amin, amax), Interval::Bounds(bmin, bmax)) => {
            let new_min = amin.max(bmin);
            let new_max = amax.min(bmax);
            if new_min > new_max {
                Interval::Empty
            } else {
                Interval::Bounds(new_min, new_max)
            }
        }
        _ => return Interval::Empty,
    }
}

fn get_interval_from_linear(m: f64, b: f64, min: f64, max: f64) -> Interval {
    match m {
        0. => {
            if b <= max && b >= min {
                Interval::Bounds(f64::NEG_INFINITY, f64::INFINITY)
            } else {
                Interval::Empty
            }
        }
        _ => {
            let tmin = (min - b) / m;
            let tmax = (max - b) / m;
            Interval::Bounds(tmin.min(tmax), tmin.max(tmax))
        }
    }
}

pub fn calculate_min(a: V3, b: V3) -> V3 {
    v(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z))
}
pub fn calculate_max(a: V3, b: V3) -> V3 {
    v(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z))
}
