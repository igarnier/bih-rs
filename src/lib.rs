pub mod aabb;
pub mod bih;

pub mod camera;
pub mod moller_trumbore;
pub mod scene;
pub mod traverse;

#[cfg(test)]
mod tests {

    use crate::moller_trumbore::{test_intersection, Ray};

    use ultraviolet::vec as uv;
    use uv::Vec3;

    use raylib::prelude::*;

    #[test]
    fn main() {
        let (mut rl, thrd) = raylib::init().size(800, 600).title("BIH").build();

        let dummy_ray = Ray {
            origin: Vec3::zero(),
            normal: Vec3::zero(),
            inormal: Vec3::zero(),
        };

        let mut scene = crate::scene::empty();
        crate::scene::add_wavefront_to_scene(&mut scene, "./cube.obj");

        println!("scene aabb {:?}", scene.global);

        let bih = crate::scene::compute_bih(&scene, 5);

        let mut rays = vec![dummy_ray; 800 * 600];
        for x in 0..800 {
            for y in 0..600 {
                let index = y * 800 + x;
                let pixx = x as i32;
                let pixy = y as i32;
                let origin = Vec3::new(0.0, 0.0, -100.0);
                let mut normal = Vec3::new((pixx - 400) as f32, (pixy - 300) as f32, 100.0);
                normal.normalize();
                let inormal = normal.map(|x| 1. / x);
                let ray: crate::moller_trumbore::Ray = crate::moller_trumbore::Ray {
                    origin,
                    normal,
                    inormal,
                };
                rays[index] = ray;
            }
        }

        let mut hit = crate::moller_trumbore::Hit {
            t: 0.0,
            u: 0.0,
            v: 0.0,
        };

        let mut iter = 0;

        println!("{}", bih);

        'running: while !rl.window_should_close() {
            if iter > 300 {
                break 'running;
            };
            let mut d = rl.begin_drawing(&thrd);
            d.clear_background(Color::WHITE);

            let mut hit = crate::moller_trumbore::Hit {
                t: 0.0,
                u: 0.0,
                v: 0.0,
            };

            for x in 0..800 {
                for y in 0..600 {
                    let index = y * 800 + x;
                    let ray = rays[index];

                    let mut put_pixel = false;

                    // for (v0, v1, v2) in crate::scene::iter_triangles (&scene) {
                    //     if test_intersection (&ray, v0, v1, v2, &mut hit) {
                    //         put_pixel = true;
                    //     }
                    // }

                    // if put_pixel {
                    //     d.draw_pixel (x as i32, y as i32, Color::RED);
                    // }

                    let hit = crate::traverse::traverse(&scene, &bih, 0, &ray, 0.0, 1000.0);

                    match hit {
                        None => (),
                        Some(_) => d.draw_pixel(x as i32, y as i32, Color::RED),
                    }
                }
            }
        }
    }
}
