use clap::Parser;
use graphics::math;
use graphics::math::V3;
use graphics::path_tracer::CupLight;
use image::{ImageBuffer, Pixel};
use rayon::iter::IntoParallelIterator;
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 512)]
    size: usize,

    #[arg(short, long, default_value_t = 5)]
    antialias: u32,

    #[arg(short, long, default_value_t = false)]
    imp: bool,

    #[arg(short, long, default_value = "out.png")]
    out: String,
}
fn main() {
    let args = Args::parse();
    let w = args.size;
    println!("Starting image generation!");
    let start = Instant::now();
    let mut img2: ImageBuffer<image::Rgb<u8>, Vec<u8>> = ImageBuffer::new(w as u32, w as u32);
    let pixel_vec: Vec<V3> = (0usize..(w * w))
        .into_par_iter()
        .map(move |x| (x % w, x / w))
        .map(move |(x, y)| {
            let s = math::Sphere {
                x: math::v(-2.1, 0., 15.),
                r: 1.,
            };
            let e = graphics::path_tracer::Emissive {
                emission: math::v(8., 0., 8.),
            };
            let obj = graphics::path_tracer::Solid {
                bsdf: Arc::new(e),
                intersectable: Arc::new(s),
            };
            let s2 = math::Sphere {
                x: math::v(2.1, 0., 15.),
                r: 1.,
            };
            let e2 = graphics::path_tracer::Emissive {
                emission: math::v(0., 8., 8.),
            };
            let obj2 = graphics::path_tracer::Solid {
                bsdf: Arc::new(e2),
                intersectable: Arc::new(s2),
            };
            let s3 = math::Sphere {
                x: math::v(0., 0., 16.),
                r: 1.,
            };
            let e3 = graphics::path_tracer::Lambertian {
                reflectance: math::v(1., 1., 1.),
            };
            let obj4 = graphics::path_tracer::Solid {
                bsdf: Arc::new(e3),
                intersectable: Arc::new(s3),
            };
             let plane = math::Plane {
                x: math::v(0., 1.1, 0.),
                n: math::v(0., -1., 0.),
                s: math::v(1., 0., 0.),
            };
            let e4 = graphics::path_tracer::Lambertian {
                reflectance: math::v(1., 1., 1.),
            };
            let obj5 = graphics::path_tracer::Solid {
                bsdf: Arc::new(e4),
                intersectable: Arc::new(plane),
            };
            let plane2 = math::Plane {
                x: math::v(0., -3.1, 0.),
                n: math::v(0., 1., 0.),
                s: math::v(1., 0., 0.),
            };
            let obj6 = graphics::path_tracer::Solid {
                bsdf: Arc::new(e4),
                intersectable: Arc::new(plane2),
            };

            let l1 = graphics::path_tracer::SphereLight {
                sphere: s,
                e,
            };
            let l2 = graphics::path_tracer::SphereLight {
                sphere: s2,
                e: e2,
            };
            let obj3 = graphics::path_tracer::Cup {
                objects: vec![
                    Box::new(obj),
                    Box::new(obj2),
                    Box::new(obj4),
                    Box::new(obj5),
                    Box::new(obj6),
                ],
            };

            let scene = graphics::path_tracer::Scene {
                object: Box::new(obj3),
                light: Box::new(CupLight{ lights: vec![Box::new(l1), Box::new(l2)] }),
            };

            let pix_width = 2. / w as f64;
            let loc = math::V3 {
                x: (2. * x as f64) / w as f64 - 1.,
                y: (2. * y as f64) / w as f64 - 1.,
                z: 2.,
            };
            let anti_aliasing = args.antialias;
            let subpixel_width = pix_width / anti_aliasing as f64;
            let mut pix_sum = math::O;
            for x_jitter in 0..anti_aliasing {
                for y_jitter in 0..anti_aliasing {
                    let jitter = math::v(
                        x_jitter as f64 * subpixel_width,
                        y_jitter as f64 * subpixel_width,
                        0.,
                    );
                    let subpix_loc = loc + jitter;
                    pix_sum = pix_sum
                        + graphics::path_tracer::estimated_total_radiance(
                            &scene,
                            &math::Ray {
                                x: math::O,
                                d: math::normalize(&subpix_loc),
                            },
                            args.imp,
                        )
                }
            }
            tone_map((1.0 / (anti_aliasing as f64 * anti_aliasing as f64)) * pix_sum)
        })
        .collect();
    for (x, y, p) in img2.enumerate_pixels_mut() {
        let color = pixel_vec[(x + y * (w as u32)) as usize];
        p.channels_mut()[0] = (color.x.abs() * 255.) as u8;
        p.channels_mut()[1] = (color.y.abs() * 255.) as u8;
        p.channels_mut()[2] = (color.z.abs() * 255.) as u8;
    }
    println!("Render took {} s", start.elapsed().as_secs_f32());
    img2.save(args.out).unwrap()
}

fn tone_map1(x: f64) -> f64 {
    x / (1. + x)
}

fn tone_map(x: V3) -> V3 {
    math::v(tone_map1(x.x), tone_map1(x.y), tone_map1(x.z))
}
