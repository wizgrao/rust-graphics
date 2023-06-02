use clap::Parser;
use graphics::linalg::{abs, add, dist, mul, normalize, sub, v, V3};
use graphics::marcher::{render, Cap};
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
