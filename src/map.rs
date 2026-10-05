use rltk::{Point, RandomNumberGenerator};
use std::cmp::{max, min};
use std::collections::HashSet;
use crate::entity::Pawn;
use crate::item::Item;
use crate::tile::TileType;
use crate::block::*;
use crate::spawn::{SpawnMap, Region};
use super::{GameError, Error};

/// Patrol-route generation strategy chosen at map creation.
pub enum PatrolStyle {
    /// Road- and door-following routes for gameplay maps.
    Roads,
    /// Concentric placeholder rings for the AI benchmark map.
    Rings,
}

// --- Patrol route generation tunables (perimeter loops) ---
/// Contour steps between waypoints on straight stretches. One patroller spawns
/// per waypoint (see `place_patrolling_enemies`), so this also sets patrol density.
const WAYPOINT_SPACING: usize = 28;
/// Waypoints per loop are capped here; huge contours get a wider stride instead.
const MAX_RING_WAYPOINTS: usize = 24;
/// Loops that thin out below this many waypoints are degenerate and dropped.
const MIN_RING_WAYPOINTS: usize = 4;
/// Obstacle clusters smaller than this get no patrol ring (lamp posts, sheds).
const MIN_HOLE_TILES: usize = 100;
/// Patrol loops per region, biggest contours first.
const MAX_ROUTES_PER_REGION: usize = 4;

/// Clockwise Moore neighbourhood, starting west.
const MOORE: [(i32, i32); 8] = [(-1, 0), (-1, -1), (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1)];

/// Moore-neighbour boundary trace of the connected component of `mask` cells
/// containing `start` (which must be its topmost, then leftmost cell). Returns
/// the component's boundary cells in clockwise walk order, plus every non-mask
/// cell brushed along the way (`halo`) — the walkable ring when tracing an
/// obstacle. `mask` must return false off-map.
fn trace_boundary(start: Point, cells: usize, mask: impl Fn(Point) -> bool) -> (Vec<Point>, Vec<Point>) {
    let mut boundary = vec![start];
    let mut halo = vec![];
    let mut cur = start;
    let mut scan = 0; // scanning starts west: all cells above and left of `start` are non-mask
    // A boundary cell is revisited at most a few times (1-wide spurs); cap the walk.
    for _ in 0..4 * cells + 8 {
        let mut next = None;
        for k in 0..8 {
            let di = (scan + k) % 8;
            let n = Point { x: cur.x + MOORE[di].0, y: cur.y + MOORE[di].1 };
            if mask(n) {
                next = Some((di, n));
                break;
            }
            halo.push(n);
        }
        let Some((di, n)) = next else { break }; // isolated single cell
        if n == start {
            break; // loop closed
        }
        boundary.push(n);
        cur = n;
        scan = (di + 6) % 8; // resume behind-left of the move, keeping the wall on our right
    }
    (boundary, halo)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Map {
    pub width: usize,
    pub height: usize,

    pub tiles: Vec<TileType>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    /// Tile-indexed spatial index of entity presence. Each entry mirrors part of an [`Entity`]
    /// for O(1) lookup. See [`Entity`] and [`Pawn`] for the authoritative data and sync rules.
    pub pawns: Vec<Option<Pawn>>,
    pub items: Vec<Option<Item>>,
    /// Per-tile FOV-blocking flag for doorway entities. Set to `true` when a Door entity occupies
    /// a Doorway tile, cleared when the door is removed. Used by `is_opaque` without needing
    /// entity access.
    pub fov_blocked: Vec<bool>,
    /// Shared, read-only patrol routes as ordered loops of waypoints. Built once
    /// at map generation and referenced by `Profile::Patrol` via index.
    /// Append ad-hoc routes via [`Map::register_patrol_route`].
    pub patrol_routes: Vec<Vec<Point>>,
    /// Tiles the player has revealed since the World last collected them (for discovery XP).
    pub newly_revealed: usize,
}

impl Map {
    pub fn pos_idx(&self, pos: Point) -> usize {
        self.xy_idx(pos.x, pos.y)
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width) + x as usize
    }

    pub fn idx_pos(&self, idx: usize) -> Point {
        let y = idx / self.width;
        let x = idx % self.width;
        
        Point {x: x as i32, y: y as i32}
    }

    pub fn is_visible(&self, pos: Point) -> bool {
        let idx = self.pos_idx(pos);
        return self.visible_tiles[idx];
    }

    pub fn blocked(&self, x: i32, y: i32) -> bool {
        let index = self.xy_idx(x, y);
        self.blocked_idx(index)
    }

    pub fn get_tile(&self, x: i32, y: i32) -> TileType {
        let index = self.xy_idx(x, y);
        return self.tiles[index];
    }

    pub fn get_item_ref(&self, x: i32, y: i32) -> &Option<Item> {
        let index = self.xy_idx(x, y);
        return &self.items[index];
    }

    pub fn get_entity_id(&self, x: i32, y: i32) -> Option<usize> {
        let index = self.xy_idx(x, y);
        if let Some(pawn) = &self.pawns[index] {
            return Some(pawn.entity_id);
        }
        else {
            return None
        }
    }

    pub fn blocked_idx(&self, index: usize) -> bool {
        match self.tiles[index] {
            TileType::Floor => self.pawns[index].is_some(),
            TileType::Ground => self.pawns[index].is_some(),
            TileType::Road => self.pawns[index].is_some(),
            TileType::Wall => true,
            TileType::Doorway => self.pawns[index].is_some(),
            TileType::Fence => true,
            TileType::Window => true
        }
    }

    pub fn get_entities_in_vicinity(&self, center: Point, radius: i32) -> Vec<usize> {
        let min_x = max(center.x - radius, 0);
        let max_x = min(center.x + radius, self.width as i32);
        let min_y = max(center.y - radius, 0);
        let max_y = min(center.y + radius, self.height as i32);
        let mut result = vec!();
        for x in min_x..max_x {
            for y in min_y..max_y {
                let index = self.xy_idx(x, y);
                match &self.pawns[index] {
                    Some(pawn) => result.push(pawn.entity_id),
                    None => ()
                }
            }
        }

        return result;
    }

    pub fn nearest_free_item_position(&self, pos: Point) -> Result<Point, GameError> {

        fn is_free(map: &Map, idx: usize) -> bool {
            return matches!(map.tiles[idx], TileType::Floor | TileType::Ground | TileType::Road)
            && map.items[idx].is_none();
        }

        return self.find_nearest_tile(pos, 5, is_free);
    }

    pub fn nearest_free_pawn_position(&self, pos: Point) -> Result<Point, GameError> {

        fn is_free(map: &Map, idx: usize) -> bool {
            return !map.blocked_idx(idx);
        }

        return self.find_nearest_tile(pos, 5, is_free);
    }

    pub fn nearest_free_pawn_position_sized(&self, pos: Point, size_x: u32, size_y: u32) -> Result<Point, GameError> {
        let fits = |p: Point| -> bool {
            for dx in 0..size_x as i32 {
                for dy in 0..size_y as i32 {
                    let x = p.x + dx;
                    let y = p.y + dy;
                    if x >= self.width as i32 || y >= self.height as i32 || x < 0 || y < 0 {
                        return false;
                    }
                    if self.blocked_idx(self.xy_idx(x, y)) {
                        return false;
                    }
                }
            }
            true
        };

        if fits(pos) {
            return Ok(pos);
        }

        for distance in 1..=5_i32 {
            for dx in -distance..=distance {
                for dy in -distance..=distance {
                    let candidate = Point { x: pos.x + dx, y: pos.y + dy };
                    if fits(candidate) {
                        return Ok(candidate);
                    }
                }
            }
        }

        Err(GameError {
            message: String::from("Could not find open spot"),
            error: Error::UnsolvableSituation,
        })
    }

    fn find_nearest_tile(&self, pos: Point, radius: usize, good_tile: fn (&Map, usize) -> bool) -> Result<Point, GameError> {
        let mut index = self.xy_idx(pos.x, pos.y);

        if good_tile(&self, index) {
            return Ok(pos);
        }

        // This should be replaced by a spiral search for efficiency. But meh.
        for distance in 1..=radius as i32 {
            for dx in -distance..=distance {
                if pos.x + dx >= self.width as i32 || pos.x + dx < 0 {
                    continue;
                }
                for dy in -distance..=distance {
                    if pos.y + dy >= self.height as i32 || pos.y + dy < 0 {
                        continue;
                    }
                    index = self.xy_idx(pos.x + dx, pos.y + dy);
                    if good_tile(&self, index) {
                        return Ok(Point {x: pos.x + dx, y: pos.y + dy});
                    }
                }
            }
        }

        return Err(
            GameError {
                message: String::from("Could not find open spot"),
                error: Error::UnsolvableSituation
        });
    }

    /// Generate a gameplay map and its spawn analysis together. Patrol routes
    /// depend on the analysis, so both are built here and returned as a pair.
    pub fn new_game_map(size_in_blocks: usize, rng: &mut RandomNumberGenerator, style: PatrolStyle) -> (Map, SpawnMap) {
        tracing::debug!("Generating map");
        let map_width = size_in_blocks * BLOCK_SIZE;
        let map_height = size_in_blocks * BLOCK_SIZE;
        let tile_count = map_width * map_height;
        let mut map = Map {
          tiles: vec![TileType::Ground; tile_count],
          width: map_width,
          height: map_height,
          revealed_tiles: vec![false; tile_count],
          visible_tiles: vec![false; tile_count],
          pawns: vec![None; tile_count],
          items: vec![None; tile_count],
          fov_blocked: vec![false; tile_count],
          patrol_routes: Vec::new(),
          newly_revealed: 0,
        };

        // Backtracking can fail on an unlucky seed, so retry — but incompatible
        // block files fail every time, which must not hang the game.
        const MAX_GENERATION_ATTEMPTS: usize = 20;
        let blocks = (0..MAX_GENERATION_ATTEMPTS)
          .find_map(|_| generate_block_grid(size_in_blocks, rng))
          .unwrap_or_else(|| panic!("Map generation failed {} times; block files may be incompatible", MAX_GENERATION_ATTEMPTS));
        for i in 0..size_in_blocks {
          for j in 0..size_in_blocks {
            for x in 0..BLOCK_SIZE {
              for y in 0..BLOCK_SIZE {
                let block_index = j * size_in_blocks + i;
                let block = &blocks[block_index];
                let map_tile_index = map.xy_idx((x + (i * BLOCK_SIZE)) as i32, (y + (j * BLOCK_SIZE)) as i32);
                map.tiles[map_tile_index] = block.tiles[block_xy_idx(x, y)];
              }
            }
          }
        }

        // Start tile mirrors the player's central spawn; feeds region depth analysis.
        let start = map.snap_to_walkable(Point::new(map_width as i32 / 2, map_height as i32 / 2));
        let spawn_map = crate::create_spawn_map(&map, map.pos_idx(start));

        match style {
            PatrolStyle::Roads => map.create_patrol_routes(&spawn_map, rng),
            PatrolStyle::Rings => map.build_patrol_rings(),
        }
        (map, spawn_map)
    }

    pub fn new_empty_map(map_size: usize) -> Map {
        let tile_count = map_size * map_size;
        Map {
            tiles: vec![TileType::Ground; tile_count],
            width: map_size,
            height: map_size,
            revealed_tiles: vec![false; tile_count],
            visible_tiles: vec![false; tile_count],
            pawns: vec![None; tile_count],
            items: vec![None; tile_count],
            fov_blocked: vec![false; tile_count],
            patrol_routes: Vec::new(),
            newly_revealed: 0,
        }
    }

    /// Append a patrol route and return its id.
    pub fn register_patrol_route(&mut self, route: Vec<Point>) -> usize {
        self.patrol_routes.push(route);
        self.patrol_routes.len() - 1
    }

    /// Concentric rectangular rings centred on the map, from a ~100-tile-wide
    /// innermost ring out toward the edges. Uniform, predictable geometry for the
    /// AI benchmark map; gameplay maps use [`Map::create_patrol_routes`] instead.
    fn build_patrol_rings(&mut self) {
        const NUM_RINGS:   usize = 4;
        const INNER_WIDTH: i32   = 100; // narrowest ring spans ~100 tiles
        const EDGE_MARGIN: i32   = 16;  // keep the outermost ring off the border

        let (w, h)   = (self.width as i32, self.height as i32);
        let (cx, cy) = (w / 2, h / 2);
        let inner_half   = (INNER_WIDTH / 2).min(cx.min(cy) - 1).max(1);
        let outer_half_x = (cx - EDGE_MARGIN).max(inner_half);
        let outer_half_y = (cy - EDGE_MARGIN).max(inner_half);

        for ring in 0..NUM_RINGS {
            let t = if NUM_RINGS > 1 { ring as f32 / (NUM_RINGS - 1) as f32 } else { 0.0 };
            let half_x = inner_half + ((outer_half_x - inner_half) as f32 * t) as i32;
            let half_y = inner_half + ((outer_half_y - inner_half) as f32 * t) as i32;
            let corners = [
                Point::new(cx - half_x, cy - half_y),
                Point::new(cx + half_x, cy - half_y),
                Point::new(cx + half_x, cy + half_y),
                Point::new(cx - half_x, cy + half_y),
            ];
            let route: Vec<Point> = corners.iter().map(|&c| self.snap_to_walkable(c)).collect();
            self.patrol_routes.push(route);
        }
    }

    /// Build gameplay patrol routes as perimeter loops: every large region gets
    /// a loop along the inside of its border walls, plus a ring around each
    /// obstacle cluster (building, fenced yard) it encloses. Loops are traced
    /// along the terrain contour — consecutive waypoints are walking neighbours,
    /// never dead ends — and each is validated for mutual reachability before
    /// registration; a loop that fails is dropped, not shipped.
    fn create_patrol_routes(&mut self, spawn_map: &SpawnMap, rng: &mut RandomNumberGenerator) {
        const MIN_REGION_TILES: usize = 1024;
        /// Obstacle-ring waypoints drift up to this far from the building, so
        /// patrols don't trace its outline tile by tile.
        const RING_JITTER: i32 = 3;

        for (ri, region) in spawn_map.regions.iter().enumerate() {
            if region.tiles.len() < MIN_REGION_TILES { continue; }

            // (ring, hugs_obstacle): border loops follow walls on purpose;
            // only rings around buildings get pushed outward.
            let mut rings = vec![(self.region_border_ring(ri, spawn_map, region), false)];
            rings.extend(self.obstacle_rings(ri, spawn_map, region).into_iter().map(|r| (r, true)));
            rings.sort_by_key(|(r, _)| std::cmp::Reverse(r.len()));

            let in_region = |p: Point| {
                p.x >= 0 && p.y >= 0 && p.x < self.width as i32 && p.y < self.height as i32
                    && spawn_map.tile_region[self.pos_idx(p)] == Some(ri)
            };
            let mut routes: Vec<Vec<Point>> = Vec::new();
            for (ring, hugs_obstacle) in rings.into_iter().take(MAX_ROUTES_PER_REGION) {
                let mut route = self.thin_ring(&ring);
                if hugs_obstacle {
                    for wp in route.iter_mut() {
                        *wp = self.push_from_obstacle(*wp, rng.range(0, RING_JITTER + 1), &in_region);
                    }
                    route.dedup();
                }
                if route.len() >= MIN_RING_WAYPOINTS && self.ring_walkable(&route) {
                    routes.push(route);
                }
            }
            for route in routes {
                self.register_patrol_route(route);
            }
        }

        #[cfg(debug_assertions)]
        tracing::debug!("Created {} patrol routes", self.patrol_routes.len());
    }

    /// Nudge a waypoint up to `dist` tiles along the outward normal of the
    /// obstacle it hugs, stopping at anything that is not open region ground.
    fn push_from_obstacle(&self, p: Point, dist: i32, in_region: &impl Fn(Point) -> bool) -> Point {
        // Outward normal: away from the surrounding impassable tiles.
        let (mut nx, mut ny) = (0i32, 0i32);
        for (dx, dy) in MOORE {
            if !self.terrain_passable(p.x + dx, p.y + dy) {
                nx -= dx;
                ny -= dy;
            }
        }
        let (sx, sy) = (nx.signum(), ny.signum());
        let mut cur = p;
        for _ in 0..dist {
            if sx == 0 && sy == 0 { break; }
            let next = Point { x: cur.x + sx, y: cur.y + sy };
            if !in_region(next) || !self.open_ground(next) { break; }
            cur = next;
        }
        cur
    }

    /// The ordered ring of region tiles along the region's border — the inside
    /// of the walls that enclose it.
    fn region_border_ring(&self, ri: usize, spawn_map: &SpawnMap, region: &Region) -> Vec<Point> {
        let start = self.idx_pos(*region.tiles.iter().min().unwrap());
        let in_region = |p: Point| {
            p.x >= 0 && p.y >= 0 && p.x < self.width as i32 && p.y < self.height as i32
                && spawn_map.tile_region[self.pos_idx(p)] == Some(ri)
        };
        trace_boundary(start, region.tiles.len(), in_region).0
    }

    /// A walkable ring around each obstacle cluster enclosed by the region:
    /// non-region cells that cannot reach the region's bounding-box edge form a
    /// hole (a building and its interior), and the ring is the region tiles
    /// brushed while tracing the hole's boundary.
    fn obstacle_rings(&self, ri: usize, spawn_map: &SpawnMap, region: &Region) -> Vec<Vec<Point>> {
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (i32::MAX, i32::MAX, 0, 0);
        for &t in &region.tiles {
            let p = self.idx_pos(t);
            min_x = min_x.min(p.x); min_y = min_y.min(p.y);
            max_x = max_x.max(p.x); max_y = max_y.max(p.y);
        }
        let box_w = (max_x - min_x + 1) as usize;
        let local = |p: Point| (p.y - min_y) as usize * box_w + (p.x - min_x) as usize;
        let in_region = |p: Point| spawn_map.tile_region[self.pos_idx(p)] == Some(ri);

        let mut seen = vec![false; box_w * (max_y - min_y + 1) as usize];
        let mut rings: Vec<Vec<Point>> = Vec::new();
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = Point { x, y };
                if seen[local(p)] || in_region(p) {
                    continue;
                }
                // Flood this non-region component within the box.
                seen[local(p)] = true;
                let mut stack = vec![p];
                let mut hole: Vec<Point> = Vec::new();
                let mut touches_edge = false;
                while let Some(c) = stack.pop() {
                    hole.push(c);
                    touches_edge |= c.x == min_x || c.y == min_y || c.x == max_x || c.y == max_y;
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let n = Point { x: c.x + dx, y: c.y + dy };
                        if n.x < min_x || n.y < min_y || n.x > max_x || n.y > max_y { continue; }
                        if seen[local(n)] || in_region(n) { continue; }
                        seen[local(n)] = true;
                        stack.push(n);
                    }
                }
                // A component reaching the box edge is the outside, not a hole.
                if touches_edge || hole.len() < MIN_HOLE_TILES {
                    continue;
                }

                let hole_set: HashSet<usize> = hole.iter().map(|&h| self.pos_idx(h)).collect();
                let in_hole = |h: Point| h.x >= min_x && h.y >= min_y && h.x <= max_x && h.y <= max_y
                    && hole_set.contains(&self.pos_idx(h));
                let start = *hole.iter().min_by_key(|h| (h.y, h.x)).unwrap();
                let (_, halo) = trace_boundary(start, hole.len(), in_hole);
                let mut ring: Vec<Point> = halo.into_iter()
                    .filter(|&h| h.x >= 0 && h.y >= 0
                        && h.x < self.width as i32 && h.y < self.height as i32
                        && in_region(h))
                    .collect();
                ring.dedup();
                rings.push(ring);
            }
        }
        rings
    }

    /// Thin a traced contour into a waypoint loop: drop cramped cells (alcoves,
    /// dead-end nooks), then keep roughly every WAYPOINT_SPACING-th survivor.
    /// Legs between waypoints are walked with full pathfinding, so corners cut
    /// here are recovered by the walker hugging the obstacle.
    fn thin_ring(&self, ring: &[Point]) -> Vec<Point> {
        let open: Vec<Point> = ring.iter().copied().filter(|&p| self.open_ground(p)).collect();
        if open.len() < MIN_RING_WAYPOINTS {
            return vec![];
        }
        let stride = (open.len() / MAX_RING_WAYPOINTS + 1).max(WAYPOINT_SPACING)
            .min((open.len() / MIN_RING_WAYPOINTS).max(1));
        let mut out: Vec<Point> = open.into_iter().step_by(stride).collect();
        out.dedup();
        out
    }

    /// Walkable terrain with at least two cardinal ways out — never a dead end.
    fn open_ground(&self, p: Point) -> bool {
        self.terrain_passable(p.x, p.y)
            && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter()
                .filter(|(dx, dy)| self.terrain_passable(p.x + dx, p.y + dy))
                .count() >= 2
    }

    /// True when every leg of the loop (including the wrap) is fully walkable
    /// over static terrain. Generation-time guard against broken routes.
    fn ring_walkable(&self, route: &[Point]) -> bool {
        let mut path = Vec::new();
        (0..route.len()).all(|i| {
            let a = self.pos_idx(route[i]);
            let b = self.pos_idx(route[(i + 1) % route.len()]);
            a == b || crate::navigate(a, b, self, &mut path)
        })
    }

    /// Bounds-checked tile lookup; `None` when off-map.
    fn tile_at(&self, x: i32, y: i32) -> Option<TileType> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        Some(self.tiles[self.xy_idx(x, y)])
    }

    /// Nearest walkable-terrain tile to `p` via an expanding ring search. Falls
    /// back to the clamped point if none is found.
    pub fn snap_to_walkable(&self, p: Point) -> Point {
        let px = p.x.clamp(1, self.width as i32 - 1);
        let py = p.y.clamp(1, self.height as i32 - 1);
        if self.terrain_passable(px, py) {
            return Point::new(px, py);
        }
        let max_r = self.width.max(self.height) as i32;
        for r in 1..max_r {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r { continue; } // perimeter only
                    let (x, y) = (px + dx, py + dy);
                    if self.terrain_passable(x, y) {
                        return Point::new(x, y);
                    }
                }
            }
        }
        Point::new(px, py)
    }

    /// Index of the patrol route whose extent best matches `pos`'s distance from
    /// the map centre — distributes patrollers across the path.
    pub fn nearest_patrol_route(&self, pos: Point) -> usize {
        let (cx, cy) = (self.width as i32 / 2, self.height as i32 / 2);
        let d = (pos.x - cx).abs().max((pos.y - cy).abs());
        self.patrol_routes.iter().enumerate()
            .min_by_key(|(_, route)| {
                let rh = route.iter()
                    .map(|p| (p.x - cx).abs().max((p.y - cy).abs()))
                    .max()
                    .unwrap_or(0);
                (rh - d).abs()
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Index of the waypoint on `route_id` nearest to `pos`.
    pub fn nearest_waypoint_index(&self, route_id: usize, pos: Point) -> usize {
        self.patrol_routes.get(route_id).map_or(0, |route| {
            route.iter().enumerate()
                .min_by_key(|(_, p)| {
                    let (dx, dy) = (p.x - pos.x, p.y - pos.y);
                    dx * dx + dy * dy
                })
                .map(|(i, _)| i)
                .unwrap_or(0)
        })
    }

    /// Whether `(x, y)` is walkable terrain, disregarding pawns. Mirrors the
    /// border-exclusion bounds of [`Map::is_exit_valid`]; doorways count as
    /// passable (they only block movement when a pawn occupies them).
    fn terrain_passable(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width as i32 - 1 || y < 1 || y > self.height as i32 - 1 {
            return false;
        }
        matches!(
            self.tiles[self.xy_idx(x, y)],
            TileType::Floor | TileType::Ground | TileType::Road | TileType::Doorway
        )
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width as i32 - 1 || y < 1 || y > self.height as i32 - 1 {
            return false;
        }
        !self.blocked(x, y)
    }

    /// Whether a closed door's pawn occupies `idx`. Walking into the tile opens
    /// the door (see `resolve_step`), so AI navigation treats it as passable.
    pub fn is_closed_door(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Doorway && self.fov_blocked[idx]
    }

    /// Exits for AI navigation: like [`Map::get_available_exits`], but closed
    /// doors count as passable at a surcharge (facing + opening costs turns).
    pub fn nav_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        const DOOR_COST: f32 = 2.0;
        const STEPS: [(i32, i32, f32); 8] = [
            (-1,  0, 1.0),  (1,  0, 1.0),  (0, -1, 1.0),  (0, 1, 1.0),
            (-1, -1, 1.45), (1, -1, 1.45), (-1, 1, 1.45), (1, 1, 1.45),
        ];

        let mut exits = rltk::SmallVec::new();
        let w = self.width as i32;
        let x = idx as i32 % w;
        let y = idx as i32 / w;

        for (dx, dy, cost) in STEPS {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 1 || nx > w - 1 || ny < 1 || ny > self.height as i32 - 1 {
                continue;
            }
            let nidx = (idx as i32 + dx + dy * w) as usize;
            if !self.blocked_idx(nidx) {
                exits.push((nidx, cost));
            } else if self.is_closed_door(nidx) {
                exits.push((nidx, cost + DOOR_COST));
            }
        }
        exits
    }

    pub fn is_opaque(&self, index: usize) -> bool {
        match self.tiles[index] {
            TileType::Wall => true,
            TileType::Floor => false,
            TileType::Ground => false,
            TileType::Road => false,
            TileType::Doorway => self.fov_blocked[index],
            TileType::Fence => false,
            TileType::Window => false
        }
    }

    /// Checks for available exits for pathfinding, with a pathfinding cost. Treats
    /// diagonals as more costly to serve as a conservative heuristic.
    pub fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width as i32;
        let y = idx as i32 / self.width as i32;
        let w = self.width as usize;

        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, 1.0)) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, 1.0)) };
    
        if self.is_exit_valid(x-1, y-1) { exits.push(((idx-w)-1, 1.45)); }
        if self.is_exit_valid(x+1, y-1) { exits.push(((idx-w)+1, 1.45)); }
        if self.is_exit_valid(x-1, y+1) { exits.push(((idx+w)-1, 1.45)); }
        if self.is_exit_valid(x+1, y+1) { exits.push(((idx+w)+1, 1.45)); }

        exits
    }
    
    pub fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Route/waypoint/spawn stats over a few seeds — run explicitly when tuning
    /// patrol generation (one patroller spawns per waypoint):
    ///   cargo test --release -- --ignored --nocapture patrol_route_stats
    #[test]
    #[ignore]
    fn patrol_route_stats() {
        for seed in [1u64, 2, 3] {
            let world = crate::World::new(16, seed, PatrolStyle::Roads);
            let map = &world.map;
            let waypoints: usize = map.patrol_routes.iter().map(|r| r.len()).sum();
            let longest = map.patrol_routes.iter().map(|r| r.len()).max().unwrap_or(0);
            let patrollers = world.entities.iter().filter(|e| matches!(&e.ai,
                crate::AI::Actor(a) if matches!(a.profile, crate::Profile::Patrol { .. }))).count();
            let actors = world.entities.iter().filter(|e| matches!(&e.ai, crate::AI::Actor(_))).count();
            println!(
                "seed {}: {} routes, {} waypoints, longest route {}; {} patrollers, {} AI actors total",
                seed, map.patrol_routes.len(), waypoints, longest, patrollers, actors,
            );
        }
    }

    /// Every generated patrol route must be a loop of open, mutually reachable
    /// waypoints — the property that keeps patrols out of dead-end corners.
    #[test]
    fn patrol_routes_are_walkable_loops() {
        let mut rng = RandomNumberGenerator::seeded(7);
        let (map, _) = Map::new_game_map(8, &mut rng, PatrolStyle::Roads);
        assert!(!map.patrol_routes.is_empty(), "no patrol routes generated");
        for (i, route) in map.patrol_routes.iter().enumerate() {
            assert!(route.len() >= MIN_RING_WAYPOINTS, "route {} too short: {}", i, route.len());
            for &wp in route {
                assert!(map.open_ground(wp), "route {} waypoint {:?} is cramped", i, wp);
            }
            assert!(map.ring_walkable(route), "route {} has an unwalkable leg", i);
        }
    }
}
