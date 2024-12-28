use clap::Parser;
use graphics::math;
use graphics::math::{B1, B2, B3, V3};
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

    #[arg(short, long, default_value_t = true)]
    imp: bool,

    #[arg(short, long, default_value_t = false)]
    preview: bool,

    #[arg(short, long, default_value = "out.png")]
    out: String,

    #[arg(short, long, default_value_t = 6)]
    bounces: i32,

    #[arg(short, long, default_value_t = 100)]
    light_samples: i32,

    #[arg(short, long, default_value_t = 0.3)]
    termination_p: f64,
}
fn main() {
    let args = Args::parse();
    let w = args.size;
    println!("Starting image generation!");
    let start = Instant::now();
    let monke_obj = Arc::new(graphics::path_tracer::obj::read_obj_file("monkey.obj").unwrap());
    let mut img2: ImageBuffer<image::Rgb<u8>, Vec<u8>> = ImageBuffer::new(w as u32, w as u32);
    let pixel_vec: Vec<V3> = (0usize..(w * w))
        .into_par_iter()
        .map(move |x| (x % w, x / w))
        .map(move |(x, y)| {
            let left_sphere_light = math::Sphere {
                x: math::v(-20.1, 0., -15.),
                r: 0.5,
            };
            let pink_light = graphics::path_tracer::Emissive {
                emission: math::v(6400., 0., 6400.),
            };
            let pink_ball_obj = graphics::path_tracer::Solid {
                bsdf: Arc::new(pink_light),
                intersectable: Arc::new(left_sphere_light),
            };
            let right_sphere_light = math::Sphere {
                x: math::v(20.1, 0., -15.),
                r: 0.5,
            };
            let turquoise_light = graphics::path_tracer::Emissive {
                emission: math::v(0., 6400., 6400.),
            };
            let turquoise_ball_obj = graphics::path_tracer::Solid {
                bsdf: Arc::new(turquoise_light),
                intersectable: Arc::new(right_sphere_light),
            };
            let middle_sphere = math::Sphere {
                x: math::v(0., 0., 16.),
                r: 1.,
            };

            let middle_triangle = math::Triangle {
                v1: math::v(1., -1., 16.),
                v0: math::v(-1.0, -1., 16.),
                v2: math::v(0., 1., 16.),
            };
            let grey_diffuse = graphics::path_tracer::Lambertian {
                reflectance: math::v(0.5, 0.5, 0.5),
            };
            let middle_ball_obj = graphics::path_tracer::Solid {
                bsdf: Arc::new(grey_diffuse),
                intersectable: Arc::new(middle_sphere),
            };
            let middle_triangle_obj = graphics::path_tracer::Solid {
                bsdf: Arc::new(grey_diffuse),
                intersectable: Arc::new(middle_triangle),
            };
            let top_plane = math::Plane {
                x: math::v(0., 5.1, 0.),
                n: math::v(0., -1., 0.),
                s: math::v(1., 0., 0.),
            };
            let top_plane_obj = graphics::path_tracer::Solid {
                bsdf: Arc::new(grey_diffuse),
                intersectable: Arc::new(top_plane),
            };
            let bottom_plane = math::Plane {
                x: math::v(0., -6.1, 0.),
                n: math::v(0., 1., 0.),
                s: math::v(1., 0., 0.),
            };
            let bottom_plane_obj = graphics::path_tracer::Solid {
                bsdf: Arc::new(grey_diffuse),
                intersectable: Arc::new(bottom_plane),
            };
            let monke_object =
                graphics::path_tracer::obj_to_solid(&monke_obj.clone(), Arc::new(grey_diffuse));
            let transformed_monke_object = graphics::path_tracer::TransformedObject::new(
                monke_object,
                math::Transform {
                    mat: math::M3::new(B1, -B2, -B3),
                    trans: math::v(0., 0., 8.),
                },
            );
            let l1 = graphics::path_tracer::SphereLight {
                sphere: left_sphere_light,
                e: pink_light,
            };
            let l2 = graphics::path_tracer::SphereLight {
                sphere: right_sphere_light,
                e: turquoise_light,
            };

            let combined_objects = graphics::path_tracer::Cup {
                objects: vec![
                    Box::new(pink_ball_obj),
                    Box::new(turquoise_ball_obj),
                    Box::new(transformed_monke_object),
                    //Box::new(middle_triangle_obj),
                    //Box::new(middle_ball_obj),
                    //Box::new(top_plane_obj),
                    //Box::new(bottom_plane_obj),
                ],
            };

            let scene = graphics::path_tracer::Scene {
                object: Box::new(combined_objects),
                light: Box::new(CupLight {
                    lights: vec![Box::new(l1), Box::new(l2)],
                }),
            };

            let pix_width = 2. / w as f64;
            let loc = math::V3 {
                x: (2. * x as f64) / w as f64 - 1.,
                y: (2. * y as f64) / w as f64 - 1.,
                z: 4.,
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
                            &graphics::path_tracer::RenderContext {
                                imp: args.imp,
                                max_bounces: args.bounces,
                                termination_p: args.termination_p,
                                light_samples: args.light_samples,
                                preview: args.preview,
                            },
                            &scene,
                            &math::Ray {
                                x: math::O,
                                d: math::normalize(&subpix_loc),
                            },
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
