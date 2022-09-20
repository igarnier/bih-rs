use ultraviolet::vec as uv;
use uv::Vec3;

#[derive(Debug, Clone)]
pub struct Aabb {
    pub mins: Vec3,
    pub maxs: Vec3,
}

pub fn make(mins: Vec3, maxs: Vec3) -> Aabb {
    Aabb { mins, maxs }
}

pub const EMPTY: Aabb = Aabb {
    mins: uv::Vec3::broadcast(f32::INFINITY),
    maxs: uv::Vec3::broadcast(f32::NEG_INFINITY),
};

pub fn extents(aabb: &Aabb) -> Vec3 {
    aabb.maxs - aabb.mins
}

pub fn join(lhs: &Aabb, rhs: &Aabb) -> Aabb {
    Aabb {
        mins: Vec3::min_by_component(lhs.mins, rhs.mins),
        maxs: Vec3::max_by_component(lhs.maxs, rhs.maxs),
    }
}

pub fn join_point(aabb: &Aabb, point: &Vec3) -> Aabb {
    Aabb {
        mins: Vec3::min_by_component(aabb.mins, *point),
        maxs: Vec3::max_by_component(aabb.maxs, *point),
    }
}

pub fn mem(point: &Vec3, aabb: &Aabb) -> bool {
    aabb.mins[0] <= point[0]
        && point[0] <= aabb.maxs[0]
        && aabb.mins[1] <= point[1]
        && point[1] <= aabb.maxs[1]
        && aabb.mins[2] <= point[2]
        && point[2] <= aabb.maxs[2]
}
