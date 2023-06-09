use crate::bih::BihState;
use crate::types::{Light, Material};
use crate::{aabb::Aabb, triaccel};
use ultraviolet::rotor::Rotor3;
use ultraviolet::vec::Vec3;
use wfront::loader::{Triangle as Tri, V3};

pub type Vertex = Vec3;

#[derive(Clone, Copy)]
pub struct Triangle {
    pub t0: u32,
    pub t1: u32,
    pub t2: u32,
    pub mat: u32,
}

pub struct Object {
    pub rot: Rotor3,
    pub pos: Vec3,
    pub tstart: usize, // inclusive
    pub tstop: usize,  // inclusive
}

pub struct Scene {
    pub ambient: Vec3,
    pub lights: Vec<Light>,
    pub materials: Vec<Material>,
    pub vbuffer: Vec<Vertex>,
    pub tbuffer: Vec<Triangle>,
    pub nbuffer: Vec<Vec3>,
    pub objects: Vec<Object>,
    bboxes: Vec<Aabb>,
    pub triaccels: Vec<triaccel::TriAccel>,
    pub global: Aabb,
}

pub struct TriangleIterator<'a> {
    pub current: usize,
    pub vbuffer: &'a Vec<Vertex>,
    pub tbuffer: &'a Vec<Triangle>,
}

impl Iterator for TriangleIterator<'_> {
    type Item = (Vertex, Vertex, Vertex);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.tbuffer.len() {
            None
        } else {
            let current = self.current;
            let vbuffer = self.vbuffer;
            let tri = &self.tbuffer[current];
            let v0 = vbuffer[tri.t0 as usize];
            let v1 = vbuffer[tri.t1 as usize];
            let v2 = vbuffer[tri.t2 as usize];
            self.current += 1;
            Some((v0, v1, v2))
        }
    }
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            ambient: Vec3::zero(),
            lights: Vec::new(),
            materials: Vec::new(),
            vbuffer: Vec::new(),
            tbuffer: Vec::new(),
            nbuffer: Vec::new(),
            objects: Vec::new(),
            triaccels: Vec::new(),
            bboxes: Vec::new(),
            global: crate::aabb::EMPTY,
        }
    }

    pub fn iter_triangles(&self) -> TriangleIterator {
        TriangleIterator {
            current: 0,
            vbuffer: &self.vbuffer,
            tbuffer: &self.tbuffer,
        }
    }

    fn add_object(
        &mut self,
        vbuffer: &mut Vec<Vertex>,
        tbuffer: &mut Vec<Triangle>,
        nbuffer: &mut Vec<Vec3>,
    ) -> Object {
        let vcount = self.vbuffer.len();

        self.vbuffer.append(vbuffer);
        self.nbuffer.append(nbuffer);

        let tstart = self.tbuffer.len();
        let tstop = tstart + tbuffer.len() - 1;

        for t in tbuffer.iter_mut() {
            t.t0 += vcount as u32;
            t.t1 += vcount as u32;
            t.t2 += vcount as u32;
            let aabb = triangle_aabb(&self.vbuffer, t);
            let p0 = self.vbuffer[t.t0 as usize];
            let p1 = self.vbuffer[t.t1 as usize];
            let p2 = self.vbuffer[t.t2 as usize];
            self.tbuffer.push(*t);
            self.global = crate::aabb::join(&aabb, &self.global);
            self.bboxes.push(aabb);
            self.triaccels.push(triaccel::precompute(p0, p1, p2));
        }

        Object {
            rot: Rotor3::identity(),
            pos: Vec3::zero(),
            tstart,
            tstop,
        }
    }

    pub fn add_wavefront(&mut self, shift: Vec3, fname: &str) -> Object {
        let mesh = wfront::loader::load(fname);

        let mut tbuffer: Vec<Triangle> = Vec::new();
        let mut vbuffer: Vec<Vec3> = Vec::new();

        println!("Loading {fname}");

        {
            println!(
                "vertices = {}; triangles = {}",
                mesh.vertices.len(),
                mesh.triangles.len()
            );

            let mut triangles: Vec<Triangle> = mesh
                .triangles
                .iter()
                .map(|Tri(t0, t1, t2)| Triangle {
                    t0: (*t0 - 1),
                    t1: (*t1 - 1),
                    t2: (*t2 - 1),
                    mat: 0,
                })
                .collect();

            tbuffer.append(&mut triangles);

            for V3(x, y, z) in mesh.vertices {
                let v = Vec3::new(x, y, z);
                vbuffer.push(shift + v);
            }
        }

        let mut nbuffer = tbuffer
            .iter()
            .map(|t| {
                let v0 = vbuffer[t.t0 as usize];
                let v1 = vbuffer[t.t1 as usize];
                let v2 = vbuffer[t.t2 as usize];
                (v1 - v0).cross(v2 - v0).normalized()
            })
            .collect();

        self.add_object(&mut vbuffer, &mut tbuffer, &mut nbuffer)
    }

    pub fn refresh_triaccel(&mut self, obj: &Object) {
        for i in obj.tstart..=obj.tstop {
            let t = self.tbuffer[i];
            let p0 = obj.pos + obj.rot * self.vbuffer[t.t0 as usize];
            let p1 = obj.pos + obj.rot * self.vbuffer[t.t1 as usize];
            let p2 = obj.pos + obj.rot * self.vbuffer[t.t2 as usize];
            self.triaccels[i] = triaccel::precompute(p0, p1, p2);
        }
    }
}

impl Object {
    pub fn set_orientation(&mut self, rot: Rotor3) {
        self.rot = rot;
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.pos = pos;
    }
}

pub fn triangle_aabb(vbuffer: &[Vertex], tri: &Triangle) -> Aabb {
    let p0 = vbuffer[tri.t0 as usize];
    let p1 = vbuffer[tri.t1 as usize];
    let p2 = vbuffer[tri.t2 as usize];
    let maxs = p0.max_by_component(p1).max_by_component(p2);
    let mins = p0.min_by_component(p1).min_by_component(p2);
    crate::aabb::make(mins, maxs)
}

impl crate::bih::Elt for Triangle {
    type T = Triangle;
    type State = Scene;

    fn extents(state: &Self::State, elt: &Self::T) -> Aabb {
        triangle_aabb(state.vbuffer.as_slice(), elt)
    }
}

pub fn compute_bih(scene: &Scene, leaf_bound: u32) -> BihState {
    crate::bih::alloc::<Triangle>(scene, &scene.tbuffer, leaf_bound)
}
