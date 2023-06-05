use ultraviolet::{f32x8, vec as uv};
use uv::{Vec3, Vec3x8};

#[derive(Debug)]
pub struct Hit {
    pub t: f32,
    pub u: f32,
    pub v: f32,
    pub dot: f32,
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
        dot: 0.0,
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
