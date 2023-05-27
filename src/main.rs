use clap::Parser;
use image::buffer::ConvertBuffer;
use image::error::ImageFormatHint::Exact;
use image::GrayImage;
use image::{GenericImage, GenericImageView, ImageBuffer, Rgb, RgbImage};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value_t = 512)]
    size: u32,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 2)]
    antialias: u32,
}

#[derive(Clone)]
pub struct V3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Default)]
pub struct Cup {
    renderables: Vec<Box<dyn Renderable>>,
}

pub struct Cap {
    renderables: Vec<Box<dyn Renderable>>,
}

fn sub(x: &V3, y: &V3) -> V3 {
    V3 {
        x: x.x - y.x,
        y: x.y - y.y,
        z: x.z - y.z,
    }
}

fn abs2(x: &V3) -> f32 {
    x.x * x.x + x.y * x.y + x.z * x.z
}

fn abs(x: &V3) -> f32 {
    abs2(x).sqrt()
}
fn v(x: f32, y: f32, z: f32) -> V3 {
    V3 { x, y, z }
}
fn mul(scalar: f32, x: &V3) -> V3 {
    V3 {
        x: x.x * scalar,
        y: x.y * scalar,
        z: x.z * scalar,
    }
}

fn add(x: &V3, y: &V3) -> V3 {
    V3 {
        x: x.x + y.x,
        y: x.y + y.y,
        z: x.z + y.z,
    }
}

struct Sphere {
    center: V3,
    radius: f32,
}

struct Torus {
    center: V3,
    axis: V3,
    big_radius: f32,
    small_radius: f32,
}

struct Plane {
    point: V3,
    axis: V3,
}

pub trait Renderable {
    fn sdf(&self, x: &V3) -> f32;
}

impl Renderable for Sphere {
    fn sdf(&self, x: &V3) -> f32 {
        abs(&sub(&x, &self.center)) - self.radius
    }
}

impl Renderable for Cup {
    fn sdf(&self, x: &V3) -> f32 {
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
    fn sdf(&self, x: &V3) -> f32 {
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

fn dist(x: &V3, y: &V3) -> f32 {
    abs(&sub(x, y))
}

impl Renderable for Torus {
    fn sdf(&self, x: &V3) -> f32 {
        let rel_pos = sub(x, &self.center); //v relPos = sub(x, t->center);
        let axis_proj = mul(dot(&rel_pos, &self.axis), &self.axis); // axisProj = proj(relPos, t->axis);
        let plane_proj = sub(&rel_pos, &axis_proj); //v planeProj = sub(relPos, axisProj);
        let circle_proj = mul(self.big_radius, &normalize(&plane_proj)); // v circleProj = scale(t->br, vnormalize(planeProj));
        dist(&circle_proj, &rel_pos) - self.small_radius
    }
}

impl Renderable for Plane {
    fn sdf(&self, x: &V3) -> f32 {
        dot(&sub(x, &self.point), &self.axis)
    }
}

const B1: V3 = V3 {
    x: 1.,
    y: 0.,
    z: 0.,
};

const B2: V3 = V3 {
    x: 0.,
    y: 1.,
    z: 0.,
};

const B3: V3 = V3 {
    x: 0.,
    y: 0.,
    z: 1.,
};

const EPS: f32 = 1e-5;

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

fn normalize(x: &V3) -> V3 {
    mul(1. / abs(x), x)
}

struct IntersectionResult {
    intersection: V3,
    normal: V3,
}

fn intersect(r: &impl Renderable, x: &V3, dir: &V3) -> Option<IntersectionResult> {
    let mut y = x.clone();
    for _ in 0..1000 {
        let sdf = r.sdf(&y);
        if sdf < EPS {
            return Some(IntersectionResult {
                normal: normalize(&dsdf(r, &y)),
                intersection: y,
            });
        }
        y = add(&y, &mul(sdf, dir));
        if abs(&y) > 10. {
            return None;
        }
    }
    None
}

const O: V3 = V3 {
    x: 0.,
    y: 0.,
    z: 0.,
};

fn dot(x: &V3, y: &V3) -> f32 {
    x.x * y.x + x.y * y.y + x.z * y.z
}

fn main() {
    let args = Args::parse();

    println!("Starting image generation!");
    let start = Instant::now();
    let w = args.size;
    let mut img2: ImageBuffer<image::Luma<f32>, Vec<f32>> = ImageBuffer::new(w, w);
    img2.par_iter_mut()
        .enumerate()
        .map(|(i, p)| (i as u32 % w, i as u32 / w, p))
        .for_each(move |(x, y, p)| {
            let s = Torus {
                center: V3 {
                    x: 0.,
                    y: 0.,
                    z: 6.,
                },
                axis: normalize(&V3 {
                    x: 1.,
                    y: -1.,
                    z: -1.3,
                }),
                big_radius: 1.2,
                small_radius: 0.9,
            };
            let t = Sphere {
                center: v(0., 0., 6.),
                radius: 0.6,
            };
            let f = Cap {
                renderables: vec![Box::new(s), Box::new(t)],
            };

            let pix_width = 2. / w as f32;
            let loc = V3 {
                x: (2. * x as f32) / w as f32 - 1.,
                y: (2. * y as f32) / w as f32 - 1.,
                z: 2.,
            };
            let anti_aliasing = args.antialias;
            let subpixel_width = pix_width / anti_aliasing as f32;
            let mut pix_sum = 0.;
            for x_jitter in 0..anti_aliasing {
                for y_jitter in 0..anti_aliasing {
                    let jitter = v(
                        x_jitter as f32 * subpixel_width,
                        y_jitter as f32 * subpixel_width,
                        0.,
                    );
                    let subpix_loc = &add(&loc, &jitter);
                    pix_sum += render(&f, subpix_loc);
                }
            }

            *p = pix_sum / (anti_aliasing as f32 * anti_aliasing as f32)
        });
    println!("Render took {} s", start.elapsed().as_secs_f32());
    let a: ImageBuffer<Rgb<u16>, Vec<u16>> = img2.convert();
    a.save("out.png").unwrap()
}

fn render(s: &impl Renderable, loc: &V3) -> f32 {
    let normalized_loc = normalize(&loc);
    let light = normalize(&V3 {
        x: 1.,
        y: 1.,
        z: 1.5,
    });

    match intersect(s, &O, &normalized_loc) {
        Some(IntersectionResult { normal, .. }) => (-dot(&normal, &light)).clamp(0.1, 0.995),
        _ => 0.,
    }
}
