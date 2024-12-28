use clap::Parser;
use graphics::marcher::{render, Cap, Sphere, Torus};
use graphics::math::{normalize, v, V3};
use image::buffer::ConvertBuffer;
use image::{ImageBuffer, Rgb};
use rayon::prelude::*;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 512)]
    size: u32,

    #[arg(short, long, default_value_t = 2)]
    antialias: u32,
}

fn main() {
    let args = Args::parse();

    println!("Starting image generation!");
    let start = Instant::now();
    let w = args.size;
    let mut img2: ImageBuffer<image::Luma<u16>, Vec<u16>> = ImageBuffer::new(w, w);
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

            let pix_width = 2. / w as f64;
            let loc = V3 {
                x: (2. * x as f64) / w as f64 - 1.,
                y: (2. * y as f64) / w as f64 - 1.,
                z: 2.,
            };
            let anti_aliasing = args.antialias;
            let subpixel_width = pix_width / anti_aliasing as f64;
            let mut pix_sum = 0.;
            for x_jitter in 0..anti_aliasing {
                for y_jitter in 0..anti_aliasing {
                    let jitter = v(
                        x_jitter as f64 * subpixel_width,
                        y_jitter as f64 * subpixel_width,
                        0.,
                    );
                    let subpix_loc = &(loc + jitter);
                    pix_sum += render(&f, subpix_loc);
                }
            }

            *p = (pix_sum / (anti_aliasing as f64 * anti_aliasing as f64) * 256.) as u16
        });
    println!("Render took {} s", start.elapsed().as_secs_f32());
    let a: ImageBuffer<Rgb<u16>, Vec<u16>> = img2.convert();
    a.save("out.png").unwrap()
}
