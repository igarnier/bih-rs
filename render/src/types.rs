use ultraviolet::{f32x8, vec as uv};
use uv::{Vec3, Vec3x8};

#[derive(Debug)]
pub struct Hit {
    pub t: f32,
    pub u: f32,
    pub v: f32,
    pub dot: f32,
    pub tri: u32,
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

pub struct Light {
    pub position: Vec3,
    pub intensity: f32,
    pub color: Vec3,
}

pub struct Material {
    pub m_color: Vec3,
    pub m_diffuse: f32, // Proportion of the light emitted by the actual light sources that is reflected by the surface
    pub m_specular: f32,
    pub m_shininess: f32,
}

pub fn new_hit() -> Hit {
    Hit {
        t: 0.0,
        u: 0.0,
        v: 0.0,
        dot: 0.0,
        tri: 0,
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

pub fn default_material(color: Vec3) -> Material {
    Material {
        m_color: color,
        m_diffuse: 1.0,
        m_specular: 1.0,
        m_shininess: 1.0,
    }
}
