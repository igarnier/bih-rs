use ultraviolet::{f32x8, vec as uv};
use uv::{Vec3, Vec3x8};

const EPS: f32 = 0.0001;

#[derive(Debug)]
pub struct Hit {
    pub t: f32,
    pub u: f32,
    pub v: f32,
}

#[derive(Debug)]
pub struct Hit8 {
    pub t: f32x8,
    pub u: f32x8,
    pub v: f32x8,
}

#[derive(Copy, Clone, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub normal: Vec3,
    pub inormal: Vec3,
}

pub struct Ray8 {
    pub origin: Vec3x8,
    pub normal: Vec3x8,
    pub inormal: Vec3x8,
}

pub fn new_hit() -> Hit {
    Hit {
        t: 0.0,
        u: 0.0,
        v: 0.0,
    }
}

pub fn new_ray(origin: Vec3, normal: Vec3) -> Ray {
    let inormal = normal.map(|x| 1. / x);
    Ray {
        origin,
        normal,
        inormal,
    }
}

pub fn new_ray8(origin: Vec3x8, normal: Vec3x8) -> Ray8 {
    let inormal = normal.map(|x| 1. / x);
    Ray8 {
        origin,
        normal,
        inormal,
    }
}

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
