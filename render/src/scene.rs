use crate::bih::BihState;
use crate::{aabb::Aabb, triaccel};
use ultraviolet::vec as uv;
use uv::Vec3;
use wfront::loader::{Triangle as Tri, V3};

// struct Light {
//     position: Vec3,
//     intensity: f32,
//     color: Vec3,
// }

// struct Material {
//     m_color: Vec3,
//     m_diffuse: f32,
//     m_specular: f32,
//     m_shininess: f32,
// }

pub type Vertex = Vec3;
pub type Triangle = [u32; 3];

pub struct Scene {
    ambient: Vec3,
    // lights: Vec<Light>,
    // materials: Vec<Material>,
    pub vbuffer: Vec<Vertex>,
    pub tbuffer: Vec<Triangle>,
    pub nbuffer: Vec<Vec3>,
    bboxes: Vec<Aabb>,
    triaccels: Vec<triaccel::TriAccel>,
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
            let vbuffer = &self.vbuffer;
            let tri = self.tbuffer[current];
            let v0 = vbuffer[tri[0] as usize];
            let v1 = vbuffer[tri[1] as usize];
            let v2 = vbuffer[tri[2] as usize];
            self.current += 1;
            Some((v0, v1, v2))
        }
    }
}

pub fn iter_triangles(scene: &Scene) -> TriangleIterator {
    TriangleIterator {
        current: 0,
        vbuffer: &scene.vbuffer,
        tbuffer: &scene.tbuffer,
    }
}

pub fn empty() -> Scene {
    Scene {
        ambient: Vec3::zero(),
        // lights: Vec::new(),
        // materials: Vec::new(),
        vbuffer: Vec::new(),
        tbuffer: Vec::new(),
        nbuffer: Vec::new(),
        triaccels: Vec::new(),
        bboxes: Vec::new(),
        global: crate::aabb::EMPTY,
    }
}

// pub fn triangle_aabb(vbuffer: &[Vertex], triangle: &Triangle) -> Aabb {
//     let x = triangle[0] as usize;
//     let y = triangle[1] as usize;
//     let z = triangle[2] as usize;
//     let p0 = vbuffer[x];
//     let p1 = vbuffer[y];
//     let p2 = vbuffer[z];
//     let mut aabb = crate::aabb::EMPTY;
//     aabb = crate::aabb::join_point(&aabb, &p0);
//     aabb = crate::aabb::join_point(&aabb, &p1);
//     aabb = crate::aabb::join_point(&aabb, &p2);
//     aabb
// }

pub fn triangle_aabb(vbuffer: &[Vertex], triangle: &Triangle) -> Aabb {
    let x = triangle[0] as usize;
    let y = triangle[1] as usize;
    let z = triangle[2] as usize;
    let p0 = vbuffer[x];
    let p1 = vbuffer[y];
    let p2 = vbuffer[z];
    let maxs = p0.max_by_component(p1).max_by_component(p2);
    let mins = p0.min_by_component(p1).min_by_component(p2);
    crate::aabb::make(mins, maxs)
}

pub fn add_object_to_scene(
    scene: &mut Scene,
    vbuffer: &mut Vec<Vertex>,
    tbuffer: &mut Vec<Triangle>,
    nbuffer: &mut Vec<Vec3>,
) {
    let vcount = scene.vbuffer.len();

    for t in tbuffer.iter_mut() {
        t[0] += vcount as u32;
        t[1] += vcount as u32;
        t[2] += vcount as u32;
        let aabb = triangle_aabb(vbuffer, t);
        let p0 = vbuffer[t[0] as usize];
        let p1 = vbuffer[t[1] as usize];
        let p2 = vbuffer[t[2] as usize];
        // println!(
        //     "tri {:?} with coords {:?} has aabb {}",
        //     t,
        //     (p0, p1, p2),
        //     aabb
        // );
        scene.tbuffer.push(*t);
        scene.global = crate::aabb::join(&aabb, &scene.global);
        scene.bboxes.push(aabb);
        scene.triaccels.push(triaccel::precompute(p0, p1, p2));
    }
    scene.vbuffer.append(vbuffer);
    scene.nbuffer.append(nbuffer);
}

pub fn add_wavefront_to_scene(scene: &mut Scene, fname: &str) {
    let mesh = wfront::loader::load(fname);

    let mut tbuffer: Vec<Triangle> = Vec::new();
    let mut vbuffer: Vec<Vec3> = Vec::new();
    // let mut nbuffer: Vec<Vec3> = Vec::new();

    println!("adding {fname}");

    {
        // let mut next_face = 0;

        println!(
            "vertices = {}; triangles = {}",
            mesh.vertices.len(),
            mesh.triangles.len()
        );

        let mut triangles: Vec<Triangle> = mesh
            .triangles
            .iter()
            .map(|Tri(t0, t1, t2)| [*t0 - 1, *t1 - 1, *t2 - 1])
            .collect();

        tbuffer.append(&mut triangles);

        for V3(v0, v1, v2) in mesh.vertices {
            let v = Vec3::new(200. * v0, 200. * v1, 800. + 200. * v2);
            vbuffer.push(v);
        }
    }

    let mut nbuffer = tbuffer
        .iter()
        .map(|t| {
            let v0 = vbuffer[t[0] as usize];
            let v1 = vbuffer[t[1] as usize];
            let v2 = vbuffer[t[2] as usize];
            let mut res = (v1 - v0).cross(v2 - v0);
            res.normalize();
            res
        })
        .collect();

    crate::scene::add_object_to_scene(scene, &mut vbuffer, &mut tbuffer, &mut nbuffer);
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
