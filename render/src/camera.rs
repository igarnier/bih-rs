use crate::types::Ray;
use ultraviolet::bivec::Bivec3;
use ultraviolet::rotor::Rotor3;
use ultraviolet::vec::Vec3;

pub struct Camera {
    pub pos: Vec3,
    pub rot: Rotor3,
    pub eyedist: f32,
}

pub struct RayIterator<'a> {
    xres: u32,
    yres: u32,
    x: u32,
    y: u32,
    cam: &'a Camera,
}

// let init ~position ~eyedist ~angle ~axis =
//   {pos = position; rot = M3.rot3_axis axis angle; eyedist}

pub fn from_angle_axis(pos: Vec3, eyedist: f32, angle: f32, axis: Vec3) -> Camera {
    Camera {
        pos,
        eyedist,
        rot: Rotor3::from_angle_plane(angle, Bivec3::from_normalized_axis(axis)),
    }
}

impl Iterator for RayIterator<'_> {
    type Item = (u32, u32, Ray);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y == self.yres {
            None
        } else {
            let x = self.x;
            let y = self.y;
            let ray_x = (x as f32) - (self.xres / 2) as f32;
            let ray_y = (y as f32) - (self.yres / 2) as f32;
            let ray_vec = Vec3::new(ray_x, ray_y, self.cam.eyedist).normalized();
            let normal = self.cam.rot * ray_vec;
            let inormal = normal.map(|x| 1. / x);
            self.x += 1;
            if self.x == self.xres {
                self.x = 0;
                self.y += 1;
            };
            Some((
                x,
                y,
                Ray {
                    origin: self.cam.pos,
                    normal,
                    inormal,
                },
            ))
        }
    }
}

impl Camera {
    pub fn iter_rays<'a>(&'a self, xres: u32, yres: u32) -> RayIterator<'a> {
        RayIterator {
            xres,
            yres,
            x: 0,
            y: 0,
            cam: self,
        }
    }
}
