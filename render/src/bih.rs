use crate::aabb::Aabb;
use std::fmt;
use ultraviolet::vec as uv;
use uv::Vec3;

pub type ObjIndex = usize;

pub type Dim = u8;

pub type NodeIndex = usize;

#[derive(Debug)]
pub enum Node {
    Leaf {
        start: ObjIndex,
        stop: ObjIndex,
    },
    Node {
        axis: Dim,
        leftclip: f32,
        rightclip: f32,
        left: NodeIndex,
        right: NodeIndex,
    },
}

pub trait Elt {
    type T;
    type State;

    fn extents(state: &Self::State, elt: &Self::T) -> Aabb;
}

pub struct BihState {
    pub index: Vec<ObjIndex>, // The BIH indexes into this array
    pub nodes: Vec<Node>,     // Nodes of the tree
    pub boxes: Vec<Aabb>,
    global: Aabb,
}

pub fn sort_objects(
    bboxes: &[Aabb],
    index: &mut [usize],
    half_dim: f32,
    dim: usize,
    left_obj: usize,
    right_obj: usize,
) -> (ObjIndex, f32, f32, f32, f32) {
    assert!(left_obj < right_obj);
    let mut left_obj: usize = left_obj;
    let mut right_obj: usize = right_obj;
    let mut lclip: f32 = -f32::MAX;
    let mut rclip: f32 = f32::MAX;
    let mut lmin: f32 = f32::MAX;
    let mut rmax: f32 = -f32::MAX;

    while right_obj >= left_obj {
        let left_box = &bboxes[index[left_obj]];
        let box_min = left_box.mins[dim];
        let box_max = left_box.maxs[dim];
        let middle = (box_min + box_max) * 0.5;
        lmin = f32::min(lmin, box_min);
        rmax = f32::max(rmax, box_max);
        if middle <= half_dim {
            left_obj += 1;
            lclip = f32::max(lclip, box_max);
        } else {
            index.swap(left_obj, right_obj);
            right_obj -= 1;
            rclip = f32::min(rclip, box_min);
        }
    }

    (left_obj, lclip, rclip, lmin, rmax)
}

#[derive(Debug)]
struct StackFrame {
    start: usize,
    stop: usize,
    bbox: Aabb,
    node_index: ObjIndex,
}

pub fn index_of_max(vec: &Vec3) -> u8 {
    if vec[0] >= vec[1] {
        if vec[0] >= vec[2] {
            0
        } else {
            2
        }
    } else if vec[1] >= vec[2] {
        1
    } else {
        2
    }
}

pub fn compute_bih(
    leaf_bound: usize,
    bboxes: &[Aabb],
    global: &Aabb,
    index: &mut [usize],
    nodes: &mut Vec<Node>,
) {
    let mut stack: Vec<StackFrame> = Vec::new();
    let size = bboxes.len();
    *nodes = Vec::with_capacity(2 * size);
    unsafe { nodes.set_len(size) };
    let init_frame = StackFrame {
        start: 0,
        stop: index.len() - 1,
        bbox: global.clone(),
        node_index: 0,
    };
    let mut cursor: usize = &init_frame.node_index + 1;
    stack.push(init_frame);

    'construction: loop {
        match stack.pop() {
            None => break 'construction,
            Some(mut frame) => {
                let start = frame.start;
                let stop = frame.stop;
                let local_bbox = &mut frame.bbox;
                if stop - start < leaf_bound {
                    let node = Node::Leaf { start, stop };
                    nodes[frame.node_index] = node;
                } else {
                    let exts = crate::aabb::extents(local_bbox);

                    let maxdim: u8 = index_of_max(&exts);
                    let mut dim: u8 = maxdim;

                    'retry: loop {
                        let d = dim as usize;
                        let half_dim = (local_bbox.mins[d] + local_bbox.maxs[d]) * 0.5;
                        let (left_end, lclip, rclip, lmin, rmax) =
                            sort_objects(bboxes, index, half_dim, d, start, stop);

                        if left_end == stop + 1 {
                            if rmax < half_dim {
                                local_bbox.maxs[d] = half_dim;
                                stack.push(frame);
                                continue 'construction;
                            } else {
                                let next = (dim + 1) % 3;
                                if next == maxdim {
                                    nodes[frame.node_index] = Node::Leaf { start, stop };
                                    break 'retry;
                                } else {
                                    dim = next;
                                }
                            }
                        } else if left_end == start {
                            if half_dim < lmin {
                                local_bbox.mins[d] = half_dim;
                                stack.push(frame);
                                continue 'construction;
                            } else {
                                let next = (dim + 1) % 3;
                                if next == maxdim {
                                    nodes[frame.node_index] = Node::Leaf { start, stop };
                                    break 'retry;
                                } else {
                                    dim = next;
                                }
                            }
                        } else {
                            let mut left_bbox = local_bbox.clone();
                            left_bbox.maxs[d] = half_dim;
                            let left_index = cursor;
                            cursor += 1;
                            let left = StackFrame {
                                start,
                                stop: (left_end - 1),
                                bbox: left_bbox,
                                node_index: left_index,
                            };
                            let mut right_bbox = local_bbox.clone();
                            right_bbox.mins[d] = half_dim;
                            let right_index = cursor;
                            cursor += 1;
                            let right = StackFrame {
                                start: left_end,
                                stop,
                                bbox: right_bbox,
                                node_index: right_index,
                            };
                            let node: Node = Node::Node {
                                axis: dim,
                                leftclip: lclip,
                                rightclip: rclip,
                                left: left_index,
                                right: right_index,
                            };
                            stack.push(right);
                            stack.push(left);
                            nodes[frame.node_index] = node;
                            continue 'construction;
                        }
                    }
                }
            }
        }
    }
}

pub fn alloc<E: Elt>(state: &E::State, objects: &[E::T], leaf_bound: usize) -> BihState {
    let size = objects.len();
    let mut index: Vec<usize> = vec![0; size];
    for (i, x) in index.iter_mut().enumerate() {
        *x = i;
    }

    let mut global: Aabb = crate::aabb::EMPTY;
    let boxes = objects
        .iter()
        .map(|obj| E::extents(state, obj))
        .collect::<Vec<Aabb>>();
    for aabb in boxes.iter() {
        global = crate::aabb::join(aabb, &global);
    }
    let mut nodes: Vec<Node> = Vec::new();
    compute_bih(leaf_bound, &boxes, &global, &mut index, &mut nodes);
    BihState {
        index,
        nodes,
        boxes,
        global,
    }
}

pub fn debug(bih: &BihState, node_index: usize, depth: usize) -> String {
    let node = &bih.nodes[node_index];
    match node {
        Node::Leaf { start, stop } => format!(
            "{:width$}Leaf start={start} stop={stop}\n",
            "",
            width = depth
        ),
        Node::Node {
            axis,
            leftclip,
            rightclip,
            left,
            right,
        } => {
            let left = debug(bih, *left, depth + 1);
            let right = debug(bih, *right, depth + 1);
            format!(
                "Node axis={axis}, lclip={leftclip}, rclip={rightclip}\n{:width$}left={}{:width$}right={}",
                "", left, "", right, width = depth
            )
        }
    }
}

impl fmt::Display for BihState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", debug(self, 0, 1))
    }
}

#[cfg(test)]
mod tests {}
