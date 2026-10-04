use rltk::{Point, RandomNumberGenerator};
use std::cmp::{max, min};
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

/// Greedily collapse points within `radius` (Chebyshev) into one representative,
/// keeping the distinct-waypoint count low.
fn cluster_points(points: &[Point], radius: i32) -> Vec<Point> {
    let mut reps: Vec<Point> = Vec::new();
    for &p in points {
        if reps.iter().all(|&r| (r.x - p.x).abs().max((r.y - p.y).abs()) > radius) {
            reps.push(p);
        }
    }
    reps
}

/// Evenly subsample down to `max` points, preserving spread.
// fn cap_waypoints(waypoints: &mut Vec<Point>, max: usize) {
//     if waypoints.len() <= max || max == 0 { return; }
//     let step = waypoints.len() as f32 / max as f32;
//     *waypoints = (0..max).map(|i| waypoints[(i as f32 * step) as usize]).collect();
// }

/// Reorder into a greedy nearest-neighbour chain for a sensible walking order.
fn order_nearest_neighbour(waypoints: &mut Vec<Point>) {
    let n = waypoints.len();
    for i in 1..n {
        let prev = waypoints[i - 1];
        let best = (i..n).min_by_key(|&j| {
            let (dx, dy) = (waypoints[j].x - prev.x, waypoints[j].y - prev.y);
            dx * dx + dy * dy
        }).unwrap_or(i);
        waypoints.swap(i, best);
    }
}

/// Split an ordered waypoint list into up to `count` contiguous routes.
fn split_routes(waypoints: &[Point], count: usize) -> Vec<Vec<Point>> {
    let count = count.clamp(1, waypoints.len().max(1));
    let per = (waypoints.len() + count - 1) / count;
    waypoints.chunks(per.max(1)).map(|c| c.to_vec()).collect()
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

    /// Build gameplay patrol routes for large regions: road/door transitions for
    /// outdoor regions, doorway-to-doorway loops for indoor ones. Route and
    /// waypoint counts are bounded.
    fn create_patrol_routes(&mut self, spawn_map: &SpawnMap, rng: &mut RandomNumberGenerator) {
        const MIN_REGION_TILES: usize = 1024;
        const TILES_PER_ROUTE:  usize = 40_000;
        const MAX_ROUTES:       usize = 8;
        //const MAX_WAYPOINTS:    usize = 6;

        // rng reserved for future jitter; deterministic layout for now.
        let _ = rng;

        for (ri, region) in spawn_map.regions.iter().enumerate() {
            if region.tiles.len() < MIN_REGION_TILES { continue; }

            let mut waypoints = if region.is_room {
                self.door_waypoints(ri, spawn_map)
            } else {
                self.road_waypoints(region)
            };
            if waypoints.len() < 2 { continue; }

            let route_count = (region.tiles.len() / TILES_PER_ROUTE).clamp(1, MAX_ROUTES);
            // TODO: Could be useful to reduce number of waypoints
            //cap_waypoints(&mut waypoints, route_count * MAX_WAYPOINTS);
            order_nearest_neighbour(&mut waypoints);

            for chunk in split_routes(&waypoints, route_count) {
                if chunk.len() >= 2 {
                    self.register_patrol_route(chunk);
                }
            }
        }

        #[cfg(debug_assertions)]
        tracing::debug!("Created {} patrol routes", self.patrol_routes.len());
    }

    /// Waypoints for an outdoor region: road tiles that transition into a doorway
    /// or terminate into open ground, clustered to one point per site.
    fn road_waypoints(&self, region: &Region) -> Vec<Point> {
        const ROAD_WIDTH: i32 = 6;
        let candidates: Vec<Point> = region.tiles.iter()
            .map(|&idx| self.idx_pos(idx))
            .filter(|&p| self.tile_at(p.x, p.y) == Some(TileType::Road))
            .filter(|&p| self.is_road_transition(p, ROAD_WIDTH))
            .collect();
        cluster_points(&candidates, ROAD_WIDTH)
    }

    /// True if road tile `p` is a doorway approach or a road end-cap. Ground on
    /// one side alone marks a road's flank too; an end-cap additionally has road
    /// running `road_width` deep the opposite way (crossing the width hits ground
    /// within that span, running down the length does not).
    fn is_road_transition(&self, p: Point, road_width: i32) -> bool {
        const DIRS: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        for (dx, dy) in DIRS {
            match self.tile_at(p.x + dx, p.y + dy) {
                Some(TileType::Doorway) => return true,
                Some(TileType::Ground) => {
                    let deep = (1..=road_width).all(|step|
                        self.tile_at(p.x - dx * step, p.y - dy * step) == Some(TileType::Road));
                    if deep { return true; }
                }
                _ => {}
            }
        }
        false
    }

    /// Waypoints for an indoor region: the middle doorway tile of each boundary.
    fn door_waypoints(&self, region_idx: usize, spawn_map: &SpawnMap) -> Vec<Point> {
        let door_positions: Vec<usize> = spawn_map.boundaries.iter()
            .filter(|b| b.region_a == region_idx || b.region_b == region_idx)
            .filter(|b| !b.door_tiles.is_empty())
            //.map(|b| self.idx_pos(b.door_tiles[b.door_tiles.len() / 2]))
            .map(|b| b.door_tiles[b.door_tiles.len() / 2])
            .collect();

        let mut waypoints: Vec<Point> = vec!();

        for door in door_positions {
            let exits = self.get_available_exits(door);
            let waypoint = exits.iter().find(|exit| spawn_map.tile_region[exit.0] == Some(region_idx));
            match waypoint {
                Some((wp, _)) => waypoints.push(self.idx_pos(*wp)),
                None => ()
            }
        }

        waypoints
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
