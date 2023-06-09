use crate::types::Ray;
use ultraviolet::bivec::Bivec3;
use ultraviolet::rotor::Rotor3;
use ultraviolet::vec::Vec3;

#[derive(Clone)]
pub struct Camera {
    pos: Vec3,
    rot: Rotor3,
    eyedist: f32,
    screen_height: f32,
    screen_width: f32,
}

pub struct RayIterator<'a> {
    xres: u32,
    yres: u32,
    x: u32,
    y: u32,
    dx: f32,
    dy: f32,
    cam: &'a Camera,
}

pub fn new(screen_width: f32, screen_height: f32, eyedist: f32) -> Camera {
    assert!(eyedist > 0.0);
    assert!(screen_height > 0.0);
    assert!(screen_width > 0.0);
    Camera {
        pos: Vec3::new(0.0, 0.0, 0.0),
        eyedist,
        rot: Rotor3::identity(),
        screen_height,
        screen_width,
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
            let ray_x = self.dx * 0.5 + ((x as f32) - (self.xres / 2) as f32) * self.dx;
            let ray_y = self.dy * 0.5 + ((y as f32) - (self.yres / 2) as f32) * self.dy;
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
            dx: self.screen_width / xres as f32,
            dy: self.screen_height / yres as f32,
            cam: self,
        }
    }

    pub fn set_position(&self, position: Vec3) -> Self {
        let mut c = self.clone();
        c.pos = position;
        c
    }

    pub fn set_orientation_angle_axis(&self, angle: f32, axis: Vec3) -> Self {
        let mut c = self.clone();
        c.rot = Rotor3::from_angle_plane(angle, Bivec3::from_normalized_axis(axis));
        c
    }
}
