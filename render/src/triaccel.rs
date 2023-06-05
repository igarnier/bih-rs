use crate::types::{Hit, Ray};
use ultraviolet::vec;
use ultraviolet::Vec3;

#[derive(Default)]
pub struct TriAccel {
    n_u: f32, // n.u / n.k
    n_v: f32, // n.v / n.k
    n_d: f32, // voir au dessus
    k: u32,   // la dimension sur laquelle on projete

    // constante de l'equation de la droite ac
    b_nu: f32,
    b_nv: f32,
    b_d: f32,

    // Les plans de projection.
    u: u16,
    v: u16,

    // equation ab
    c_nu: f32,
    c_nv: f32,
    c_d: f32,
    sign: i32, // le signe de la normale
}

const FAST_MODULO: [usize; 5] = [0, 1, 2, 0, 1];

pub fn precompute(a: Vec3, b: Vec3, c: Vec3) -> TriAccel {
    let mut out = TriAccel::default();
    let ab = b - a;
    let ac = c - a;

    let n = Vec3::cross(&ab, ac);
    let na = n.as_array();

    let max_dim: usize = if n.x > n.y {
        // x > y
        if n.x.abs() > n.z.abs() {
            // x > y; x > z
            0
        } else {
            // z >= x > y
            2
        }
    } else {
        // y >= x
        if n.y.abs() > n.z.abs() {
            // y >= x; y > z
            1
        } else {
            // z > y >= x
            2
        }
    };
    out.k = max_dim as u32;

    out.sign = if na[max_dim] > 0.0 { 1 } else { -1 };

    out.u = FAST_MODULO[max_dim + 1] as u16;
    out.v = FAST_MODULO[max_dim + 2] as u16;
    let u = out.u as usize;
    let v = out.v as usize;

    out.n_u = na[u] / na[max_dim];
    out.n_v = na[v] / na[max_dim];

    // out->n_d = A->m[out->k] + out->n_u * A->m[out->u] + out->n_v * A->m[out->v];
    let a = a.as_array();

    out.n_d = a[max_dim] + out.n_u * a[u] + out.n_v * a[v];

    let ax = a[u];
    let ay = a[v];

    let b = b.as_array();
    let bx = b[u] - ax;
    let by = b[v] - ay;

    let c = c.as_array();
    let cx = c[u] - ax;
    let cy = c[v] - ay;

    let denom = 1.0 / (bx * cy - by * cx);
    out.b_nu = -by * denom;
    out.b_nv = bx * denom;
    out.b_d = (by * ax - bx * ay) * denom;
    out.c_nu = cy * denom;
    out.c_nv = -cx * denom;
    out.c_d = (cx * ay - cy * ax) * denom;

    out
}

// Calcule l'intersection s'il y a intersection
// Wald powered
pub fn triaccel_intersect(tri: &TriAccel, ray: &Ray, tmin: f32, tmax: f32, hit: &mut Hit) -> bool {
    let dir = ray.normal.as_array();
    let nd: f32 =
        1. / (dir[tri.k as usize] + tri.n_u * dir[tri.u as usize] + tri.n_v * dir[tri.v as usize]);

    let pos = ray.origin.as_array();
    let f = (tri.n_d
        - pos[tri.k as usize]
        - tri.n_u * pos[tri.u as usize]
        - tri.n_v * pos[tri.v as usize])
        * nd;

    if f < tmin || f > tmax {
        return false;
    }

    // calcule le point d'intersection avec le plan de projection.
    let hu = pos[tri.u as usize] + f * dir[tri.u as usize];
    let hv = pos[tri.v as usize] + f * dir[tri.v as usize];

    // calcule les parametres du barycentre :
    let lambda = hu * tri.b_nu + hv * tri.b_nv + tri.b_d;

    if lambda < 0. {
        return false;
    }

    let mu = hu * tri.c_nu + hv * tri.c_nv + tri.c_d;

    if mu < 0. {
        return false;
    }

    if lambda + mu > 1. {
        return false;
    }

    hit.t = f;
    hit.dot = nd;
    return true;
}
