use std::time::Instant;
use image::{ImageBuffer, Pixel, Rgb};
use image::buffer::ConvertBuffer;
use rayon::iter::IntoParallelIterator;
use rayon::prelude::*;
use graphics::math;
use graphics::math::{Intersectable, Intersection};
use clap::Parser;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value_t = 512)]
    size: usize,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 5)]
    antialias: u32,
}
fn main() {
    let args = Args::parse();
    let s = math::Sphere {
        x: math::v(0., 0., 5.),
        r: 1.,
    };
    let w = args.size;
    println!("Starting image generation!");
    let start = Instant::now();
    let mut img2: ImageBuffer<image::Rgb<u8>, Vec<u8>> = ImageBuffer::new(w as u32, w as u32);
    let yeet = 0usize..(w*w) as usize;
    let beet = yeet.into_par_iter();
    let ceet = beet.map(move |x| {(x%w, x/w)});
    let fin: Vec<math::V3> = ceet.map(move |(x, y)| {
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
                pix_sum = pix_sum + match s.intersect(&math::Ray{ x: math::O, d: math::normalize(&subpix_loc) }) {
                    None => math::O,
                    Some(intersection) => intersection.s,
                };
                //dbg!(pix_sum);
            }
        }
        return (1.0/ (anti_aliasing as f64 * anti_aliasing as f64))*pix_sum;
    }).collect();
    for (x, y, p) in img2.enumerate_pixels_mut() {
        let color = fin[(x + y*(w as u32)) as usize];
        p.channels_mut()[0] = (color.x.abs()*255.) as u8;
        p.channels_mut()[1] = (color.y.abs()*255.) as u8;
        p.channels_mut()[2] = (color.z.abs()*255.) as u8;
    }
    println!("Render took {} s", start.elapsed().as_secs_f32());
    img2.save("out.png").unwrap()
}