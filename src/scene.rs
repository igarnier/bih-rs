use crate::aabb::Aabb;
use crate::bih::BihState;
use ultraviolet::vec as uv;
use uv::Vec3;

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
        bboxes: Vec::new(),
        global: crate::aabb::EMPTY,
    }
}

pub fn triangle_aabb(vbuffer: &[Vertex], triangle: &Triangle) -> Aabb {
    let x = triangle[0] as usize;
    let y = triangle[1] as usize;
    let z = triangle[2] as usize;
    let p0 = vbuffer[x];
    let p1 = vbuffer[y];
    let p2 = vbuffer[z];
    let mut aabb = crate::aabb::EMPTY;
    aabb = crate::aabb::join_point(&aabb, &p0);
    aabb = crate::aabb::join_point(&aabb, &p1);
    aabb = crate::aabb::join_point(&aabb, &p2);
    aabb
}

pub fn add_object_to_scene(
    scene: &mut Scene,
    vbuffer: &mut Vec<Vertex>,
    tbuffer: &mut Vec<Triangle>,
    nbuffer: &mut Vec<Vec3>,
) {
    let vcount = scene.vbuffer.len();

    for t in tbuffer.iter_mut() {
        let aabb = triangle_aabb(vbuffer, t);
        t[0] += vcount as u32;
        t[1] += vcount as u32;
        t[2] += vcount as u32;
        scene.tbuffer.push(*t);
        scene.global = crate::aabb::join(&aabb, &scene.global);
        scene.bboxes.push(aabb);
    }
    scene.vbuffer.append(vbuffer);
    scene.nbuffer.append(nbuffer);
}

pub fn add_wavefront_to_scene(scene: &mut Scene, fname: &str) {
    let (models, _materials) =
        tobj::load_obj(fname, &tobj::LoadOptions::default()).expect("Failed to OBJ load file");

    let mut tbuffer: Vec<Triangle> = Vec::new();
    let mut vbuffer: Vec<Vec3> = Vec::new();
    // let mut nbuffer: Vec<Vec3> = Vec::new();

    println!("adding {fname}");

    for (_i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;
        // let mut next_face = 0;

        println!(
            "vertices = {}; triangles = {}",
            mesh.positions.len(),
            mesh.indices.len() / 3
        );

        let mut triangles: Vec<Triangle> =
            mesh.indices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();

        tbuffer.append(&mut triangles);

        // for face in 0..mesh.face_arities.len() {
        //     let arity = mesh.face_arities[face] as usize;
        //     assert!(arity == 3);
        //     let end = next_face + mesh.face_arities[face] as usize;

        //     let tri = &mesh.indices[next_face..end];

        //     tbuffer.push([
        //         tri[0].try_into().unwrap(),
        //         tri[1].try_into().unwrap(),
        //         tri[2].try_into().unwrap(),
        //     ]);

        //     next_face = end;
        // }

        assert!(mesh.positions.len() % 3 == 0);

        for vtx in 0..mesh.positions.len() / 3 {
            let v = Vec3::new(
                200. * mesh.positions[3 * vtx],
                200. * mesh.positions[3 * vtx + 1],
                800. + 200. * mesh.positions[3 * vtx + 2],
            );
            vbuffer.push(v);
        }

        // for vtx in 0..mesh.positions.len() / 3 {
        //     let v = Vec3::new(
        //         3. * mesh.positions[3 * vtx],
        //         3. * mesh.positions[3 * vtx + 1],
        //         3. * mesh.positions[3 * vtx + 2],
        //     );
        //     vbuffer.push(v);
        // }
    }

    // let mut tbuffer: Vec<Triangle> = obj
    //     .indices
    //     .chunks(3)
    //     .map(|c| c.try_into().expect("slice with incorrect length"))
    //     .collect();

    // let mut vbuffer = obj
    //     .vertices
    //     .iter()
    //     .map(|v| {
    //         let p = v.position;
    //         Vec3::new(p[0], p[1], p[2])
    //     })
    //     .collect();
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

pub fn compute_bih(scene: &Scene, leaf_bound: usize) -> BihState {
    crate::bih::alloc::<Triangle>(scene, &scene.tbuffer, leaf_bound)
}
