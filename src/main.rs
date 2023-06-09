use clap::Parser;
use raylib::prelude::*;
use render::types::Ray;
use render::{scene, trace, types};

use std::str::FromStr;
use ultraviolet::Vec3;
use wfront::loader::{load, Mesh};

fn parse_r(arg: &str) -> Result<WindowResolution, std::io::Error> {
    let mut cs = arg.split('x');
    let Some(xres) = cs.next() else
    { return Err(std::io::Error::new(std::io::ErrorKind::Other, "not a complex")) };
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

    scene
        .materials
        .push(types::default_material(Vec3::new(1.0, 1.0, 1.0)));

    scene.lights.push(types::Light {
        position: Vec3::new(0.0, 0.0, 0.0),
        intensity: 100.0,
        color: Vec3::new(1.0, 1.0, 1.0),
    });

    // scene.ambient = Vec3::new(0.5, 0.5, 0.5);

    use std::time::Instant;

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

                let color_vec = render::trace::raytrace(1, &scene, &bih, &ray);
                let r = (color_vec.x.clamp(0.0, 1.) * 255.) as u8;
                let g = (color_vec.y.clamp(0.0, 1.) * 255.) as u8;
                let b = (color_vec.z.clamp(0.0, 1.) * 255.) as u8;
                let color = Color { r, g, b, a: 255 };

                d.draw_pixel(x as i32, y as i32, color)
            }
        }

        let elapsed = now.elapsed().as_millis();

        println!("Rendering time: {elapsed} ms");
    }
}
