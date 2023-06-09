use clap::Parser;
use raylib::prelude::*;
use render::types::Ray;
use render::{camera, scene, trace, types};

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

    let mut scene = render::scene::Scene::new();
    let _obj = scene.add_wavefront(Vec3::new(3.5, 0.0, 0.0), &args.filename);
    let _obj = scene.add_wavefront(Vec3::new(-3.5, 0.0, 0.0), &args.filename);
    let _obj = scene.add_wavefront(Vec3::new(0.0, -5.0, 0.0), "plane.obj");

    // need proper BIH construction
    // obj.set_position(Vec3::new(1., 1., 0.));
    // scene.refresh_triaccel(&obj);

    scene
        .materials
        .push(types::default_material(Vec3::new(1.0, 1.0, 1.0)));

    scene.lights.push(types::Light {
        position: Vec3::new(5.0, 5.0, -10.0),
        intensity: 5.0,
        color: Vec3::new(1.0, 0.0, 0.0),
    });

    scene.lights.push(types::Light {
        position: Vec3::new(-5.0, 5.0, -10.0),
        intensity: 5.0,
        color: Vec3::new(0.0, 0.0, 1.0),
    });

    use std::time::Instant;

    let now = Instant::now();

    let bih = scene::compute_bih(&scene, 6);

    let elapsed = now.elapsed().as_nanos();

    println!("Construction time: {elapsed} ns");

    let camera = camera::new(8., 6., 5.)
        .set_position(Vec3::new(0.0, 0.0, -10.))
        .set_orientation_angle_axis(0.0, Vec3::new(0.0, 1.0, 0.0));

    let mut iter = 0;

    'running: while !rl.window_should_close() {
        if iter > 300 {
            break 'running;
        };
        iter += 1;
        let mut d = rl.begin_drawing(&thrd);
        d.clear_background(Color::BLACK);

        let now = Instant::now();

        camera.iter_rays(800, 600).for_each(|(x, y, ray)| {
            let color_vec = render::trace::raytrace(2, &scene, &bih, &ray);
            let r = (color_vec.x.clamp(0., 1.) * 255.) as u8;
            let g = (color_vec.y.clamp(0., 1.) * 255.) as u8;
            let b = (color_vec.z.clamp(0., 1.) * 255.) as u8;
            let color = Color { r, g, b, a: 255 };

            d.draw_pixel(x as i32, y as i32, color)
        });

        let elapsed = now.elapsed().as_millis();

        println!("Rendering time: {elapsed} ms");
    }
}
