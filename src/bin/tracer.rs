use clap::Parser;
use graphics::math::{Triangle, B1, B2, B3, V3};
use graphics::path_tracer::obj;
use graphics::path_tracer::obj::ObjLine;
use graphics::path_tracer::primitives::CupLight;
use graphics::{math, path_tracer};
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

    #[arg(short, long, default_value_t = 10)]
    min_leaf_size: usize,

    #[arg(short, long, default_value_t = 0.3)]
    termination_p: f64,

    #[arg(short, long, default_value_t = 10)]
    replicas: i32,

    #[arg(short, long, default_value = "monkey.obj")]
    file: String,

    #[arg(long, default_value_t = 0.0001)]
    lens_radius: f64,
}
fn main() {
    let args = Args::parse();
    let w = args.size;
    let h = args.size * 2 / 3;
    println!("initializing scene");
    let mut start = Instant::now();
    let grey_diffuse = path_tracer::primitives::Lambertian {
        reflectance: math::v(0.7, 0.7, 0.7),
    };
    let monke_obj = graphics::path_tracer::obj::read_obj_file(&args.file).unwrap();
    let monke_triangles = path_tracer::primitives::obj_to_triangles(&monke_obj);
    let mut monke_triangles_transformed: Vec<Vec<Triangle>> = Vec::new();
    for i in -args.replicas..=args.replicas {
        for j in -args.replicas..=args.replicas {
            let translate = 3.0 * i as f64 * B1 + 3. * j as f64 * B2;
            monke_triangles_transformed.push(
                monke_triangles
                    .iter()
                    .map(|math::Triangle { v0, v1, v2 }| Triangle {
                        v0: *v0 + translate,
                        v1: *v1 + translate,
                        v2: *v2 + translate,
                    })
                    .collect(),
            );
        }
    }
    let final_monke_triangles = monke_triangles_transformed
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    println!("init took {} s", start.elapsed().as_secs_f32());
    start = Instant::now();
    println!("building bvh tree");
    let monke_object = Arc::new(graphics::path_tracer::primitives::triangles_to_solid(
        final_monke_triangles,
        Arc::new(grey_diffuse),
        args.min_leaf_size,
    ));
    let transformed_monke_object =
        Arc::new(graphics::path_tracer::primitives::TransformedObject::new(
            monke_object,
            math::Transform {
                mat: math::M3::new(B1, -B2, -B3),
                trans: math::v(0., 0.15, 0.5),
            },
        ));
    println!("bvh took {} s", start.elapsed().as_secs_f32());
    start = Instant::now();
    let mut img2: ImageBuffer<image::Rgb<u8>, Vec<u8>> = ImageBuffer::new(w as u32, h as u32);
    println!("rendering image");
    let lens_width = 0.035;
    let lens_height = 0.035 * 2. / 3.;
    let focal_length = 0.035;
    let f_num = 64.0;
    let camera = path_tracer::Camera::new(
        math::O,
        B3,
        focal_length / f_num * 0.5,
        focal_length,
        0.5,
        0.5 * lens_width * B1,
        0.5 * lens_height * B2,
    );

    dbg!(&camera);
    let pixel_vec: Vec<V3> = (0usize..(w * h))
        .into_par_iter()
        .map(move |x| (x % w, x / w))
        .map(move |(x, y)| {
            let left_sphere_light = math::Sphere {
                x: math::v(-20.1, 0., -15.),
                r: 0.5,
            };
            let pink_light = graphics::path_tracer::primitives::Emissive {
                emission: math::v(6400., 0., 6400.),
            };
            let pink_ball_obj = graphics::path_tracer::primitives::Solid {
                bsdf: Arc::new(pink_light),
                intersectable: Arc::new(left_sphere_light),
            };
            let right_sphere_light = math::Sphere {
                x: math::v(20.1, 0., -15.),
                r: 0.5,
            };
            let turquoise_light = graphics::path_tracer::primitives::Emissive {
                emission: math::v(0., 6400., 6400.),
            };
            let turquoise_ball_obj = graphics::path_tracer::primitives::Solid {
                bsdf: Arc::new(turquoise_light),
                intersectable: Arc::new(right_sphere_light),
            };

            let top_plane = math::Plane {
                x: math::v(0., 1.1, 0.),
                n: math::v(0., -1., 0.),
                s: math::v(1., 0., 0.),
            };
            let top_plane_obj = graphics::path_tracer::primitives::Solid {
                bsdf: Arc::new(grey_diffuse),
                intersectable: Arc::new(top_plane),
            };
            let bottom_plane = math::Plane {
                x: math::v(0., -1.1, 0.),
                n: math::v(0., 1., 0.),
                s: math::v(1., 0., 0.),
            };
            let bottom_plane_obj = graphics::path_tracer::primitives::Solid {
                bsdf: Arc::new(grey_diffuse),
                intersectable: Arc::new(bottom_plane),
            };

            let l1 = graphics::path_tracer::primitives::SphereLight {
                sphere: left_sphere_light,
                e: pink_light,
            };
            let l2 = graphics::path_tracer::primitives::SphereLight {
                sphere: right_sphere_light,
                e: turquoise_light,
            };

            let combined_objects = graphics::path_tracer::primitives::Cup {
                objects: vec![
                    Arc::new(pink_ball_obj),
                    Arc::new(turquoise_ball_obj),
                    transformed_monke_object.clone(),
                    //Arc::new(top_plane_obj),
                    //Arc::new(bottom_plane_obj),
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
                y: (-2. * y as f64) / h as f64 + 1.,
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
                            &camera.sample_ray(subpix_loc.x, subpix_loc.y),
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
