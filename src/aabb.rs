use ultraviolet::vec as uv;
use uv::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub struct Aabb {
    pub mins: Vec3,
    pub maxs: Vec3,
}

pub fn make(mins: Vec3, maxs: Vec3) -> Aabb {
    Aabb { mins, maxs }
}

pub const EMPTY: Aabb = Aabb {
    mins: uv::Vec3::broadcast(0.0),
    maxs: uv::Vec3::broadcast(0.0),
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
        && point[0] < aabb.maxs[0]
        && aabb.mins[1] <= point[1]
        && point[1] < aabb.maxs[1]
        && aabb.mins[2] <= point[2]
        && point[2] < aabb.maxs[2]
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    const TEST: Aabb = Aabb {
        mins: uv::Vec3::broadcast(0.0),
        maxs: uv::Vec3::broadcast(1.0),
    };

    #[test]
    fn test_extents_empty() {
        assert_eq!(extents(&EMPTY), Vec3::zero());
    }

    #[test]
    fn test_join_empty() {
        let right = join(&EMPTY, &TEST);
        let left = join(&TEST, &EMPTY);
        assert_eq!(&right, &TEST);
        assert_eq!(&left, &TEST);
    }

    #[test]
    fn test_mem_empty() {
        let point: Vec3 = Vec3::new(0.0, 0.0, 0.0);
        assert!(!(mem(&point, &EMPTY)));
    }

    #[test]
    fn test_mem_nonempty_oob() {
        let point: Vec3 = Vec3::new(1.0, 1.0, 1.0);
        assert!(!(mem(&point, &TEST)));
    }

    #[test]
    fn test_mem_nonempty_ib() {
        let point: Vec3 = Vec3::new(0.9, 0.9, 0.9);
        assert!((mem(&point, &TEST)));
    }
}
