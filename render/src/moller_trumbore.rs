use crate::types::{Hit, Hit8, Ray, Ray8};
use ultraviolet::{f32x8, vec as uv};
use uv::{Vec3, Vec3x8};

const EPS: f32 = 0.0001;

pub fn test_intersection(ray: &Ray, p0: Vec3, p1: Vec3, p2: Vec3, out: &mut Hit) -> bool {
    let edge1 = p1 - p0;
    let edge2 = p2 - p0;
    let mut pvec = Vec3::cross(&ray.normal, edge2);
    let det = Vec3::dot(&edge1, pvec);
    if det > -EPS && det < EPS {
        false
    } else {
        let inv_det = 1.0 / det;
        let tvec = ray.origin - p0;
        let ucoord = Vec3::dot(&tvec, pvec) * inv_det;
        if !(0.0..=1.0).contains(&ucoord) {
            false
        } else {
            pvec = Vec3::cross(&tvec, edge1);
            let vcoord = Vec3::dot(&pvec, ray.normal) * inv_det;
            if vcoord < 0.0 || ucoord + vcoord > 1.0 {
                false
            } else {
                let hit_dist = Vec3::dot(&pvec, edge2) * inv_det;
                out.t = hit_dist;
                out.u = ucoord;
                out.v = vcoord;
                true
            }
        }
    }
}

pub fn test_intersection_branchless(
    ray: &Ray,
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
    out: &mut Hit,
) -> bool {
    let edge1 = p1 - p0;
    let edge2 = p2 - p0;
    let mut pvec = Vec3::cross(&ray.normal, edge2);
    let det = Vec3::dot(&edge1, pvec);
    let det_cond = det > -EPS && det < EPS;
    let inv_det = 1.0 / det;
    let tvec = ray.origin - p0;
    let ucoord = Vec3::dot(&tvec, pvec) * inv_det;
    let ucoord_cond = !(0.0..=1.0).contains(&ucoord);
    pvec = Vec3::cross(&tvec, edge1);
    let vcoord = Vec3::dot(&pvec, ray.normal) * inv_det;
    let vcoord_cond = vcoord < 0.0 || ucoord + vcoord > 1.0;
    let hit_dist = Vec3::dot(&pvec, edge2) * inv_det;
    out.t = hit_dist;
    out.u = ucoord;
    out.v = vcoord;
    !(det_cond || ucoord_cond || vcoord_cond)
}

pub fn test_intersection_branchless8(
    ray: &Ray8,
    p0: Vec3x8,
    p1: Vec3x8,
    p2: Vec3x8,
    out: &mut Hit8,
) -> f32x8 {
    use wide::{CmpGe, CmpLe};
    let edge1 = p1 - p0;
    let edge2 = p2 - p0;
    let mut pvec = Vec3x8::cross(&ray.normal, edge2);
    let det: f32x8 = Vec3x8::dot(&edge1, pvec);
    let det_mask = det.cmp_le(f32x8::splat(-EPS)) | det.cmp_ge(f32x8::splat(EPS));
    let inv_det = 1.0 / det;
    let tvec = ray.origin - p0;
    let ucoord = Vec3x8::dot(&tvec, pvec) * inv_det;
    let ucoord_mask = ucoord.cmp_ge(f32x8::splat(0.0)) & ucoord.cmp_le(f32x8::splat(1.0));
    pvec = Vec3x8::cross(&tvec, edge1);
    let vcoord = Vec3x8::dot(&pvec, ray.normal) * inv_det;
    let vcoord_mask =
        vcoord.cmp_ge(f32x8::splat(0.0)) | (ucoord + vcoord).cmp_le(f32x8::splat(1.0));
    let hit_dist = Vec3x8::dot(&pvec, edge2) * inv_det;
    out.t = hit_dist;
    out.u = ucoord;
    out.v = vcoord;
    det_mask & ucoord_mask & vcoord_mask
}

#[cfg(test)]
mod tests {
    use super::*;

    const P0: Vec3 = Vec3::new(-1.0, 0.0, 1.0);
    const P1: Vec3 = Vec3::new(1.0, 0.0, 1.0);
    const P2: Vec3 = Vec3::new(0.0, 1.5, 1.0);

    #[test]
    fn test_hit() {
        let origin: Vec3 = Vec3::zero();
        let normal: Vec3 = Vec3::new(0.0, 0.0, 1.0);
        let ray: Ray = new_ray(origin, normal);
        let mut hit: Hit = new_hit();
        let res: bool = test_intersection(&ray, P0, P1, P2, &mut hit);
        assert!(res);
        assert!(hit.t == 1.0);
    }

    #[test]
    fn test_hit_neg() {
        let origin: Vec3 = Vec3::zero();
        let normal: Vec3 = Vec3::new(0.0, 0.0, -1.0);
        let ray: Ray = new_ray(origin, normal);
        let mut hit: Hit = new_hit();
        let res: bool = test_intersection(&ray, P0, P1, P2, &mut hit);
        assert!(res);
    }

    #[test]
    fn test_epsilon() {
        let origin: Vec3 = Vec3::zero();
        // Shoot the ray just above the typ of the triangle.
        let normal: Vec3 = (P2 + Vec3::new(0.0, EPS, 0.0)).normalized();
        let ray: Ray = new_ray(origin, normal);
        let mut hit: Hit = new_hit();
        let res: bool = test_intersection(&ray, P0, P1, P2, &mut hit);
        assert!(!res);
        // Shoot the ray just above the typ of the triangle.
        let normal: Vec3 = (P2 + Vec3::new(0.0, -EPS, 0.0)).normalized();
        let ray: Ray = new_ray(origin, normal);
        let res: bool = test_intersection(&ray, P0, P1, P2, &mut hit);
        assert!(res);
    }
}
