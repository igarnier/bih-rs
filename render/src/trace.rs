use crate::bih::BihState;
use crate::scene::{Scene, Triangle};
use crate::traverse::traverse;
use crate::types::{Hit, Light, Material, Ray};
use ultraviolet::vec::Vec3;

type Rgb = Vec3;

const BLACK: Rgb = Vec3::new(0.0, 0.0, 0.0);

fn shadow_ray(normal: Vec3, hit_pos: Vec3, light_pos: Vec3) -> (Ray, f32) {
    let shifted_hit_pos = hit_pos + 0.1 * normal;
    let vec = light_pos - shifted_hit_pos;
    let length = vec.mag();
    let ilength = 1. / length;
    let normal = ilength * vec;
    let inormal = normal.map(|x| 1. / x);
    (
        Ray {
            origin: shifted_hit_pos,
            normal,
            inormal,
        },
        ilength,
    )
}

pub fn raytrace(maxdepth: usize, scene: &Scene, bih: &BihState, ray: &Ray) -> Vec3 {
    if maxdepth <= 0 {
        // background shader ray
        return BLACK;
    }

    let tmin = 1.0;
    let tmax = f32::MAX;

    let hit = traverse(scene, bih, 0, ray, tmin, tmax);

    let nbuffer: &[Vec3] = &scene.nbuffer;
    let tbuffer: &[Triangle] = &scene.tbuffer;
    let materials: &[Material] = &scene.materials;

    match hit {
        None => {
            // Should be background shader ray
            BLACK
        }
        Some(Hit {
            t,
            u: _,
            v: _,
            dot,
            tri,
        }) => {
            // compute reflection and shadow rays
            let tri_norm = nbuffer[tri as usize];
            let material = &materials[tbuffer[tri as usize].mat as usize];
            let hitpoint = ray.origin + t * ray.normal;
            let dotprod = -2.0 * dot;
            let refl_dir = ray.normal + dotprod * tri_norm;
            let reciprocal = refl_dir.map(|x| 1. / x);
            let rray = Ray {
                origin: hitpoint,
                normal: refl_dir,
                inormal: reciprocal,
            };
            let lights = &scene.lights;

            let mut illumination = scene.ambient;
            for l in lights.into_iter() {
                let (sray, ilength) = shadow_ray(tri_norm, hitpoint, l.position);
                // let ilength = 1.0;
                // let shifted_hitpoint = hitpoint; // - 1. * tri_norm;
                // let normal = (ray.origin - shifted_hitpoint).normalized();
                // let inormal = normal.map(|x| 1. / x);
                // let sray = Ray {
                //     origin: shifted_hitpoint,
                //     normal,
                //     inormal,
                // };
                let hit = traverse(scene, bih, 0, &sray, 0., f32::MAX);
                match hit {
                    Some(_) => (),
                    None => {
                        // println!("ok");
                        // TODO: we use linear falloff instead of quadratic, not realistic
                        // let light_color = Vec3::new(0.5, 0.5, 0.5);
                        let light_color =
                            (tri_norm.dot(sray.normal).abs() * l.intensity * ilength) * l.color;
                        let result_color = material.m_color * (material.m_diffuse * light_color);
                        illumination += result_color;
                    }
                }
            }
            let reflected_color = raytrace(maxdepth - 1, scene, bih, &rray);
            let result_color = material.m_color * (material.m_diffuse * reflected_color);
            result_color + illumination
        }
    }
}
