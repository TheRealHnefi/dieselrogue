use std::collections::BinaryHeap;
use std::cell::RefCell;
use std::cmp::Ordering;
use rltk::BaseMap;
use crate::Map;

/// Maximum tile expansions before returning a best-effort partial path.
/// Bounds worst-case cost per call while still covering paths across the full map
/// in the common case.
const MAX_EXPANSIONS: usize = 2048;

pub struct NavPath {
    /// Tile indices in forward order: steps[0] is the first tile to enter,
    /// steps[last] is the destination (or the closest reachable tile when
    /// success is false).  The start tile is not included.
    pub steps: Vec<usize>,
    /// True if the exact destination was reached within the expansion budget.
    pub success: bool,
}

#[derive(PartialEq)]
struct Node {
    idx: usize,
    f: f32,
    g: f32,
}

impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap on f.  NaN-safe: treat NaN as equal.
        other.f.partial_cmp(&self.f).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Per-thread reusable scratch space for A*.
///
/// `visited` is a flat array indexed by tile index storing
/// `(generation, came_from, g)`. An entry is valid when its generation
/// matches `self.generation`, so "clearing" between calls is a single
/// counter increment rather than iterating over the array.
struct AStarScratch {
    visited:    Vec<(u32, usize, f32)>,
    generation: u32,
    open:       BinaryHeap<Node>,
}

impl AStarScratch {
    fn new() -> Self {
        AStarScratch {
            visited:    Vec::new(),
            generation: 1,
            open:       BinaryHeap::with_capacity(512),
        }
    }

    fn reset(&mut self, map_size: usize) {
        if self.visited.len() < map_size {
            self.visited.resize(map_size, (0, 0, 0.0));
        }
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            // generation 0 is reserved as "unvisited"; skip it
            self.generation = 1;
            self.visited.iter_mut().for_each(|e| e.0 = 0);
        }
        self.open.clear();
    }

    #[inline]
    fn get(&self, idx: usize) -> Option<(usize, f32)> {
        let e = self.visited[idx];
        if e.0 == self.generation { Some((e.1, e.2)) } else { None }
    }

    #[inline]
    fn insert(&mut self, idx: usize, came_from: usize, g: f32) {
        self.visited[idx] = (self.generation, came_from, g);
    }

    fn build_path(&self, start: usize, target: usize, success: bool) -> NavPath {
        let mut steps = Vec::new();
        let mut current = target;

        loop {
            let parent = match self.get(current) {
                Some((p, _)) => p,
                None => break,
            };
            if parent == start {
                steps.push(current);
                break;
            }
            steps.push(current);
            if parent == current {
                break; // safety: non-negative edge costs prevent this
            }
            current = parent;
        }

        steps.reverse();
        NavPath { steps, success }
    }
}

thread_local! {
    static SCRATCH: RefCell<AStarScratch> = RefCell::new(AStarScratch::new());
}

/// Find a path from `start` to `end` on `map`.
///
/// Uses A* with lazy deletion so each tile is effectively expanded at most once.
/// If the expansion budget is exhausted before the destination is reached, returns
/// the path to the visited tile with the smallest heuristic distance to the goal,
/// giving the caller a best-effort route that always makes forward progress.
///
/// Returns an empty `steps` vec only when no visited neighbour of `start` exists
/// (completely walled in) or when `start == end`.
pub fn navigate(start: usize, end: usize, map: &Map) -> NavPath {
    if start == end {
        return NavPath { steps: vec![], success: true };
    }

    SCRATCH.with(|cell| {
        let mut scratch = cell.borrow_mut();
        scratch.reset(map.width * map.height);

        let h0 = map.get_pathing_distance(start, end);
        scratch.insert(start, start, 0.0);
        scratch.open.push(Node { idx: start, f: h0, g: 0.0 });

        let mut best_h   = h0;
        let mut best_idx = start;
        let mut expansions = 0;
        let mut found = false;

        while let Some(current) = scratch.open.pop() {
            if let Some((_, recorded_g)) = scratch.get(current.idx) {
                if current.g > recorded_g + f32::EPSILON {
                    continue;
                }
            }

            if current.idx == end {
                found = true;
                break;
            }

            if expansions >= MAX_EXPANSIONS {
                break;
            }
            expansions += 1;

            for (neighbour, edge_cost) in map.get_available_exits(current.idx) {
                let new_g = current.g + edge_cost;

                if let Some((_, existing_g)) = scratch.get(neighbour) {
                    if new_g >= existing_g {
                        continue;
                    }
                }

                let h = map.get_pathing_distance(neighbour, end);
                if h < best_h {
                    best_h   = h;
                    best_idx = neighbour;
                }

                scratch.insert(neighbour, current.idx, new_g);
                scratch.open.push(Node { idx: neighbour, f: new_g + h, g: new_g });
            }
        }

        if found {
            scratch.build_path(start, end, true)
        } else if best_idx != start {
            scratch.build_path(start, best_idx, false)
        } else {
            NavPath { steps: vec![], success: false }
        }
    })
}
