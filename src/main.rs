use clap::Parser;
use raylib::prelude::*;
use render::types::Ray;
use render::*;
use scene;
use std::str::FromStr;
use ultraviolet::Vec3;
use wfront::loader::{load, Mesh};

fn parse_r(arg: &str) -> Result<WindowResolution, std::io::Error> {
    let mut cs = arg.split("x");
    let Some(xres) = cs.next() else { return Err(std::io::Error::new(std::io::ErrorKind::Other, "not a complex")) };
    let Ok (xres) = u32::from_str(xres) else { return Err(std::io::Error::new(std::io::ErrorKind::Other, "not a complex")) };
    let xres = xres - (xres % 10);
    let Some(yres) = cs.next() else { return Err(std::io::Error::new(std::io::ErrorKind::Other, "not a complex")) };
    let Ok (yres) = u32::from_str(yres) else { return Err(std::io::Error::new(std::io::ErrorKind::Other, "not a complex")) };
    let yres = yres - (yres % 10);
    let None = cs.next() else { return Err(std::io::Error::new(std::io::ErrorKind::Other, "not a complex")) };
    return Ok(WindowResolution { xres, yres });
}

const DEFAULT_WINDOW_RESOLUTION: WindowResolution = WindowResolution {
    xres: 800,
    yres: 600,
};

#[derive(Copy, Clone)]
pub struct WindowResolution {
    pub xres: u32,
    pub yres: u32,
}

impl std::fmt::Display for WindowResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("{}x{}", self.xres, self.yres))
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value_t = DEFAULT_WINDOW_RESOLUTION, value_parser = parse_r)]
    pub resolution: WindowResolution,
    pub filename: String,
}

pub fn main() {
    let args = Args::parse();

    let xres = args.resolution.xres;
    let yres = args.resolution.yres;

    let (mut rl, thrd) = raylib::init()
        .size(xres as i32, yres as i32)
        .title("BIH")
        .build();

    let mut scene = render::scene::empty();
    render::scene::add_wavefront_to_scene(&mut scene, &args.filename);

    use std::time::{Duration, Instant};

    let now = Instant::now();

    let bih = scene::compute_bih(&scene, 5);

    let elapsed = now.elapsed().as_nanos();

    println!("Construction time: {elapsed} ns");

    let dummy_ray = Ray {
        origin: Vec3::zero(),
        normal: Vec3::zero(),
        inormal: Vec3::zero(),
    };

    let mut rays = vec![dummy_ray; (xres * yres) as usize];

    for x in 0..xres {
        for y in 0..yres {
            let index = (y * xres + x) as usize;
            let origin = Vec3::new(0.0, 0.0, -(yres as f32) / 2.0);
            let mut normal = Vec3::new(
                x as f32 - (xres as f32) / 2.,
                y as f32 - (yres as f32) / 2.,
                yres as f32 / 2.0,
            );
            normal.normalize();
            let inormal = normal.map(|x| 1. / x);
            let ray = Ray {
                origin,
                normal,
                inormal,
            };
            rays[index] = ray;
        }
    }

    let mut iter = 0;

    'running: while !rl.window_should_close() {
        if iter > 300 {
            break 'running;
        };
        iter += 1;
        let mut d = rl.begin_drawing(&thrd);
        d.clear_background(Color::BLACK);

        let now = Instant::now();

        for x in 0..800 {
            for y in 0..600 {
                let index = y * 800 + x;
                let ray = rays[index];

                let hit = render::traverse::traverse(&scene, &bih, 0, &ray, 0.0, f32::MAX);

                match hit {
                    None => (),
                    Some(_) => d.draw_pixel(x as i32, y as i32, Color::RED),
                }
            }
        }

        let elapsed = now.elapsed().as_millis();

        println!("Rendering time: {elapsed} ms");
    }
}
