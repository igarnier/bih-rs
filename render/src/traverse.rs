use crate::bih::{BihState, Node};
use crate::scene::Scene;
use crate::triaccel::{triaccel_intersect, TriAccel};
use crate::types::{Hit, Ray};
use ultraviolet::vec as uv;
use uv::Vec3;

pub fn intersect_ray(
    scene: &Scene,
    index: &[u32],
    ray: &Ray,
    tmin: f32,
    tmax: f32,
    tri_start: &u32,
    tri_end: &u32,
) -> Option<Hit> {
    let mut min_hit = Hit {
        t: f32::INFINITY,
        u: 0.0,
        v: 0.0,
        dot: 0.0,
    };

    let abuffer: &[TriAccel] = &scene.triaccels;

    for tri in *tri_start..=*tri_end {
        let i = index[tri as usize] as usize;
        let triaccel = &abuffer[i];
        let mut hit = Hit {
            t: 0.0,
            u: 0.0,
            v: 0.0,
            dot: 0.0,
        };

        if triaccel_intersect(&triaccel, ray, tmin, tmax, &mut hit) && hit.t < min_hit.t {
            min_hit = hit
        }
    }

    if min_hit.t == f32::INFINITY {
        None
    } else {
        Some(min_hit)
    }
}

pub fn traverse(
    scene: &Scene,
    bih: &BihState,
    node_index: u32,
    ray: &Ray,
    tmin: f32,
    tmax: f32,
) -> Option<Hit> {
    let node = &bih.nodes[node_index as usize];
    if tmin >= tmax {
        return None;
    };
    match node {
        Node::Leaf { start, stop } => {
            intersect_ray(scene, &bih.index, ray, tmin, tmax, start, stop)
        }
        Node::Node {
            axis,
            leftclip,
            rightclip,
            left,
        } => {
            let right = *left + 1;
            let dim = *axis as usize;

            let ray_start = ray.origin[dim] + ray.normal[dim] * tmin;
            let ray_stop = ray.origin[dim] + ray.normal[dim] * tmax;

            if ray.normal[dim] >= 0.0 {
                // going left-to-right : first left, then right
                if ray_start <= *leftclip {
                    // ray intersects left subspace
                    let far_clip = f32::min((leftclip - ray.origin[dim]) * ray.inormal[dim], tmax);
                    // explore left
                    let left_hit = traverse(scene, bih, *left, ray, tmin, far_clip);

                    if leftclip <= rightclip {
                        // boxes do not overlap - we explore the right if
                        // we didn't hit anything in the left and the ray is nonempty in the right
                        match left_hit {
                            None => {
                                if *rightclip <= ray_stop {
                                    let near_clip = f32::max(
                                        (rightclip - ray.origin[dim]) * ray.inormal[dim],
                                        tmin,
                                    );
                                    traverse(scene, bih, right, ray, near_clip, tmax)
                                } else {
                                    None
                                }
                            }
                            Some(_) => left_hit,
                        }
                    } else if *rightclip <= ray_stop {
                        // boxes do overlap - we have to explore both boxes and pick the nearest hit
                        let near_clip =
                            f32::max((rightclip - ray.origin[dim]) * ray.inormal[dim], tmin);
                        let right_hit = traverse(scene, bih, right, ray, near_clip, tmax);
                        match (left_hit, right_hit) {
                            (None, None) => None,
                            (None, x) | (x, None) => x,
                            (Some(x), Some(y)) => {
                                if x.t < y.t {
                                    Some(x)
                                } else {
                                    Some(y)
                                }
                            }
                        }
                    } else {
                        // boxes do not overlap and ray stops before right subspace
                        left_hit
                    }
                } else if *rightclip <= ray_stop {
                    // ray does not intersect left subspace but intersects right one
                    let near_clip =
                        f32::max((rightclip - ray.origin[dim]) * ray.inormal[dim], tmin);
                    traverse(scene, bih, right, ray, near_clip, tmax)
                } else {
                    None
                }
            } else {
                // going right-to-left : first right, then left
                if *rightclip <= ray_start {
                    // ray intersects right subspace
                    let far_clip = f32::min((rightclip - ray.origin[dim]) * ray.inormal[dim], tmax);
                    // explore right
                    let right_hit = traverse(scene, bih, right, ray, tmin, far_clip);

                    if leftclip < rightclip {
                        // boxes do not overlap - we explore the right if
                        // we didn't hit anything in the left and the ray is nonempty in the right
                        match right_hit {
                            None => {
                                if ray_stop <= *leftclip {
                                    let near_clip = f32::max(
                                        (leftclip - ray.origin[dim]) * ray.inormal[dim],
                                        tmin,
                                    );
                                    traverse(scene, bih, *left, ray, near_clip, tmax)
                                } else {
                                    None
                                }
                            }
                            _ => right_hit,
                        }
                    } else if ray_stop <= *leftclip {
                        // boxes do overlap - we have to explore both boxes and pick the nearest hit
                        let near_clip =
                            f32::max((leftclip - ray.origin[dim]) * ray.inormal[dim], tmin);
                        let left_hit = traverse(scene, bih, *left, ray, near_clip, tmax);

                        match (right_hit, left_hit) {
                            (None, None) => None,
                            (None, x) | (x, None) => x,
                            (Some(x), Some(y)) => {
                                if x.t < y.t {
                                    Some(x)
                                } else {
                                    Some(y)
                                }
                            }
                        }
                    } else {
                        // boxes do not overlap and ray stops before right subspace
                        right_hit
                    }
                } else if ray_stop <= *leftclip {
                    // ray does not intersect left subspace but intersects right one
                    let near_clip = f32::max((leftclip - ray.origin[dim]) * ray.inormal[dim], tmin);
                    traverse(scene, bih, *left, ray, near_clip, tmax)
                } else {
                    None
                }
            }
        }
    }
}
