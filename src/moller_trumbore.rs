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
