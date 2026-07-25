use std::collections::BinaryHeap;
use std::cell::RefCell;
use std::cmp::Ordering;
use crate::Map;

/// Maximum tile expansions before returning a best-effort partial path.
/// Bounds worst-case cost per call while still covering paths across the full map
/// in the common case.
const MAX_EXPANSIONS: usize = 2048;

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

    /// Trace from `target` back to `start` and write steps into `out` in
    /// **reversed order**: `out[0]` is the step closest to `target`,
    /// `out.last()` is the first step after `start`.  The parent-trace loop
    /// naturally produces this order, so no final reverse is needed.
    fn fill_reversed(&self, start: usize, target: usize, out: &mut Vec<usize>) {
        out.clear();
        let mut current = target;
        loop {
            let parent = match self.get(current) {
                Some((p, _)) => p,
                None => break,
            };
            if parent == start {
                out.push(current);
                break;
            }
            out.push(current);
            if parent == current {
                break; // safety: non-negative edge costs prevent this
            }
            current = parent;
        }
    }
}

thread_local! {
    static SCRATCH: RefCell<AStarScratch> = RefCell::new(AStarScratch::new());
}

/// Like [`navigate`], but skips the repath when the destination has moved no
/// more than `tolerance` tiles from the last computed target and the existing
/// path is not stale (empty or next step blocked).
///
/// Pass `tolerance = 0` for fixed destinations (waypoints, anchors, last-known
/// positions) where exact precision is required.  Use a higher value for
/// moving targets to avoid repathing every turn.
///
/// `cached_target` should be stored alongside the path buffer and passed back
/// unchanged on every call.
pub fn navigate_cached(
    start: usize,
    end: usize,
    map: &Map,
    out: &mut Vec<usize>,
    cached_target: &mut Option<usize>,
    tolerance: u32,
) -> bool {
    let path_stale = out.is_empty()
        || out.last().map_or(false, |&i| map.blocked_idx(i));

    let dest_ok = match *cached_target {
        Some(prev) if prev == end => true,
        Some(prev) if tolerance > 0 =>
            map.get_pathing_distance(prev, end) <= tolerance as f32,
        _ => false,
    };

    if dest_ok && !path_stale {
        return true;
    }

    *cached_target = Some(end);
    navigate(start, end, map, out)
}

/// Returns the adjacent walkable tile that minimizes distance to `end`, or
/// `None` if no walkable neighbour is strictly closer than `start` itself.
///
/// O(8) — use this when the destination is visible so that line-of-sight
/// guarantees a straight approach won't get trapped behind an opaque wall.
/// The caller should fall back to [`navigate_cached`] on `None`.
pub fn greedy_step(start: usize, end: usize, map: &Map) -> Option<usize> {
    let current_h = map.get_pathing_distance(start, end);
    let mut best_h   = current_h;
    let mut best_idx = None;

    for (idx, _) in map.get_available_exits(start) {
        let h = map.get_pathing_distance(idx, end);
        if h < best_h {
            best_h   = h;
            best_idx = Some(idx);
        }
    }

    best_idx
}

/// A precomputed Dijkstra distance field over **static terrain** toward a single
/// goal cell.  `dist[i]` is the integer cost of the cheapest terrain path from
/// tile `i` to the goal (orthogonal step = 10, diagonal = 14); [`u16::MAX`] marks
/// tiles from which the goal is unreachable.
///
/// Pawn occupancy is deliberately **not** baked in, so the field stays valid as
/// entities move and can be built once and shared across turns and agents.
/// Transient blocking is handled at read time by [`DistField::step`], which only
/// descends into tiles that are currently walkable.
pub struct DistField {
    dist: Vec<u16>,
}

impl DistField {
    /// Orthogonal / diagonal step costs, scaled to integers so the flood fill
    /// can use a cheap integer queue.  Mirror the 1.0 / 1.45 costs in
    /// `Map::get_available_exits`.
    const ORTHO: u32 = 10;
    const DIAG:  u32 = 14;

    /// Next step for an agent at `from`: the *currently walkable* neighbour with
    /// the lowest field value strictly below `from`'s own.  Returns `None` when
    /// the goal is unreachable from `from` over static terrain, or when every
    /// descending neighbour is transiently blocked (caller should wait or fall
    /// back to A*).
    ///
    /// `map.get_available_exits` already excludes pawn-occupied tiles, so reading
    /// the static field through it yields dynamic obstacle avoidance for free.
    pub fn step(&self, from: usize, map: &Map) -> Option<usize> {
        let cur = self.dist[from];
        if cur == u16::MAX {
            return None;
        }
        let mut best_idx = None;
        let mut best_val = cur;
        for (nb, _) in map.get_available_exits(from) {
            if self.dist[nb] < best_val {
                best_val = self.dist[nb];
                best_idx = Some(nb);
            }
        }
        best_idx
    }
}

/// Build a full-map [`DistField`] toward `goal` (see [`build_field_bounded`]).
pub fn build_field(goal: usize, map: &Map) -> DistField {
    build_field_bounded(goal, map, u32::MAX)
}

/// Build a [`DistField`] toward `goal` by flooding outward over static terrain
/// (walls block; pawns are ignored — see [`DistField`]), stopping once the cost
/// exceeds `max_cost`. Tiles past that horizon stay [`u16::MAX`] (unreachable),
/// so agents there fall back to A*. Pass `u32::MAX` for a full-map field (static
/// goals); a bound keeps dynamic-goal fields (investigation / last-known) cheap,
/// since interested agents cluster near the goal. Cost is O(tiles within bound).
pub fn build_field_bounded(goal: usize, map: &Map, max_cost: u32) -> DistField {
    println!("Building field");
    let size = map.width * map.height;
    let mut dist = vec![u16::MAX; size];
    if goal >= size {
        return DistField { dist };
    }

    // Reuse `Node`'s min-heap-on-`f` ordering as an integer priority queue:
    // `f` carries the accumulated cost, `g`/`h` are unused here.
    let mut open: BinaryHeap<Node> = BinaryHeap::new();
    dist[goal] = 0;
    open.push(Node { idx: goal, f: 0.0, g: 0.0 });

    while let Some(current) = open.pop() {
        let d = current.f as u32;
        if d > dist[current.idx] as u32 {
            continue; // stale heap entry — a cheaper cost was already recorded
        }
        for (nb, cost) in map.terrain_exits(current.idx) {
            let step = if cost > 1.0 { DistField::DIAG } else { DistField::ORTHO };
            let nd = d + step;
            if nd <= max_cost && nd < dist[nb] as u32 {
                // Reserve u16::MAX for "unreachable"; clamp so it never collides.
                dist[nb] = nd.min(u16::MAX as u32 - 1) as u16;
                open.push(Node { idx: nb, f: nd as f32, g: 0.0 });
            }
        }
    }

    DistField { dist }
}

/// Find a path from `start` to `end` on `map`, writing steps into `out`.
///
/// Steps are written in **reversed order**: `out[0]` is the step closest to
/// `end`, `out.last()` is the first step to take from `start`.  This layout
/// lets callers maintain the path as a stack (pop from back = advance).
///
/// Returns `true` if the exact destination was reached within the expansion
/// budget.  On a partial path `out` holds a best-effort route toward the goal
/// (always making forward progress); `out` is empty only when start is
/// completely walled in.
///
/// `out` is always cleared before writing.
pub fn navigate(start: usize, end: usize, map: &Map, out: &mut Vec<usize>) -> bool {
    #[cfg(debug_assertions)]
    puffin::profile_function!();

    if start == end {
        out.clear();
        return true;
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
            // Lazy deletion: a cheaper route to this tile was already recorded.
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
            scratch.fill_reversed(start, end, out);
            true
        } else if best_idx != start {
            scratch.fill_reversed(start, best_idx, out);
            false
        } else {
            out.clear();
            false
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Map;
    use crate::tile::TileType;

    /// Follow the field one `step` at a time from `start`; return the number of
    /// steps to reach `goal`, or `None` if the field reports being stuck.
    fn walk(field: &DistField, map: &Map, start: usize, goal: usize) -> Option<usize> {
        let mut at = start;
        for steps in 0..1000 {
            if at == goal { return Some(steps); }
            at = field.step(at, map)?;
        }
        None
    }

    #[test]
    fn field_descends_to_goal_on_open_terrain() {
        let map = Map::new_empty_map(12, 12);
        let goal = map.xy_idx(6, 6);
        let field = build_field(goal, &map);
        assert_eq!(walk(&field, &map, map.xy_idx(2, 2), goal), Some(4)); // Chebyshev distance (diagonals allowed)
    }

    #[test]
    fn field_routes_around_wall() {
        let mut map = Map::new_empty_map(12, 12);
        // Vertical wall at x=6 spanning y=1..=9, leaving a detour open at the
        // bottom (y=10, y=11) between the start (left) and goal (right).
        for y in 1..10 {
            let idx = map.xy_idx(6, y);
            map.tiles[idx] = TileType::Wall;
        }
        let goal = map.xy_idx(10, 6);
        let field = build_field(goal, &map);
        // Reachable only by detouring around the wall's bottom end.
        assert!(walk(&field, &map, map.xy_idx(2, 6), goal).is_some());
    }

    #[test]
    fn field_reports_unreachable_goal() {
        let mut map = Map::new_empty_map(12, 12);
        // Full vertical wall at x=6 splits the interior in two (no detour).
        for y in 1..12 {
            let idx = map.xy_idx(6, y);
            map.tiles[idx] = TileType::Wall;
        }
        let goal = map.xy_idx(10, 6);
        let field = build_field(goal, &map);
        let start = map.xy_idx(2, 6);
        assert_eq!(field.step(start, &map), None); // walled off from the goal
    }
}
