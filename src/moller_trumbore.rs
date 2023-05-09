use ultraviolet::vec as uv;
use uv::Vec3;

const EPS: f32 = 0.0001;

#[derive(Debug)]
pub struct Hit {
    pub t: f32,
    pub u: f32,
    pub v: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub normal: Vec3,
    pub inormal: Vec3,
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
        if ucoord < 0.0 || ucoord > 1.0 {
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

// pub fn test_intersection_nobranch(
//     ray: &Ray,
//     p0: &Vec3,
//     p1: &Vec3,
//     p2: &Vec3,
//     out: &mut Hit,
// ) -> bool {
//     let mut edge1 = crate::vec3::zero();
//     let mut edge2 = crate::vec3::zero();
//     let mut pvec = crate::vec3::zero();
//     crate::vec3::sub(p1, p0, &mut edge1);
//     crate::vec3::sub(p2, p0, &mut edge2);
//     crate::vec3::cross(&ray.normal, &edge2, &mut pvec);
//     let det = crate::vec3::dot(&edge1, &pvec);
//     let test1 = !(det > -EPS && det < EPS);
//     let inv_det = 1.0 / det;
//     let mut tvec = crate::vec3::zero();
//     crate::vec3::sub(&ray.origin, p0, &mut tvec);
//     let ucoord = crate::vec3::dot(&tvec, &pvec) * inv_det;
//     let test2 = !(ucoord < 0.0 || ucoord > 1.0);
//     crate::vec3::cross(&tvec, &edge1, &mut pvec);
//     let vcoord = crate::vec3::dot(&ray.normal, &pvec) * inv_det;
//     let test3 = !(vcoord < 0.0 || ucoord + vcoord > 1.0);
//     let hit_dist = crate::vec3::dot(&edge2, &pvec) * inv_det;
//     out.t = hit_dist;
//     out.u = ucoord;
//     out.v = vcoord;
//     (test1 as i32 * test2 as i32 * test3 as i32) != 0
// }

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
        let normal: Vec3 = (P2.clone() + Vec3::new(0.0, EPS, 0.0)).normalized();
        let ray: Ray = new_ray(origin, normal);
        let mut hit: Hit = new_hit();
        let res: bool = test_intersection(&ray, P0, P1, P2, &mut hit);
        assert!(!res);
        // Shoot the ray just above the typ of the triangle.
        let normal: Vec3 = (P2.clone() + Vec3::new(0.0, -EPS, 0.0)).normalized();
        let ray: Ray = new_ray(origin, normal);
        let res: bool = test_intersection(&ray, P0, P1, P2, &mut hit);
        assert!(res);
    }
}
