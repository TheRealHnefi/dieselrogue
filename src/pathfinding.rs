use std::collections::{BinaryHeap, HashMap};
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
    #[cfg(debug_assertions)]
    puffin::profile_function!();
    if start == end {
        return NavPath { steps: vec![], success: true };
    }

    // visited[tile] = (came_from, cheapest g seen)
    let mut visited: HashMap<usize, (usize, f32)> = HashMap::new();
    let mut open: BinaryHeap<Node> = BinaryHeap::new();

    let h0 = map.get_pathing_distance(start, end);
    visited.insert(start, (start, 0.0));
    open.push(Node { idx: start, f: h0, g: 0.0 });

    let mut best_h   = h0;
    let mut best_idx = start;
    let mut expansions = 0;

    while let Some(current) = open.pop() {
        // Lazy deletion: a cheaper route to this tile was already recorded.
        if let Some(&(_, recorded_g)) = visited.get(&current.idx) {
            if current.g > recorded_g + f32::EPSILON {
                continue;
            }
        }

        if current.idx == end {
            return build_path(&visited, start, end, true);
        }

        if expansions >= MAX_EXPANSIONS {
            break;
        }
        expansions += 1;

        for (neighbour, edge_cost) in map.get_available_exits(current.idx) {
            let new_g = current.g + edge_cost;

            if let Some(&(_, existing_g)) = visited.get(&neighbour) {
                if new_g >= existing_g {
                    continue;
                }
            }

            let h = map.get_pathing_distance(neighbour, end);
            if h < best_h {
                best_h   = h;
                best_idx = neighbour;
            }

            visited.insert(neighbour, (current.idx, new_g));
            open.push(Node { idx: neighbour, f: new_g + h, g: new_g });
        }
    }

    // Destination not reached within budget: return path to closest node seen.
    if best_idx != start {
        build_path(&visited, start, best_idx, false)
    } else {
        NavPath { steps: vec![], success: false }
    }
}

fn build_path(
    visited: &HashMap<usize, (usize, f32)>,
    start: usize,
    target: usize,
    success: bool,
) -> NavPath {
    let mut steps = Vec::new();
    let mut current = target;

    loop {
        let parent = match visited.get(&current) {
            Some(&(p, _)) => p,
            None => break,
        };
        if parent == start {
            steps.push(current);
            break;
        }
        steps.push(current);
        if parent == current {
            break; // safety: should never happen with non-negative edge costs
        }
        current = parent;
    }

    steps.reverse();
    NavPath { steps, success }
}
