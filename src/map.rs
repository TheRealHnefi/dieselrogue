use rltk::{Point, RandomNumberGenerator};
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use crate::entity::Pawn;
use crate::item::Item;
use crate::tile::TileType;
use crate::block::*;
use crate::DistField;
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
/// keeping the distinct-waypoint count (and thus pinned flow fields) low.
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
fn cap_waypoints(waypoints: &mut Vec<Point>, max: usize) {
    if waypoints.len() <= max || max == 0 { return; }
    let step = waypoints.len() as f32 / max as f32;
    *waypoints = (0..max).map(|i| waypoints[(i as f32 * step) as usize]).collect();
}

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

/// Resident cache of flow fields keyed by goal tile index, with turn-based
/// eviction so dynamic goals (investigation / last-known cells) don't accumulate
/// once nobody is heading to them any more.
struct NavFieldCache {
    fields: HashMap<usize, DistField>,
    /// Turns since each field's goal was last demanded (0 = wanted this turn).
    /// Only *evictable* fields appear here; pinned fields are never aged.
    idle: HashMap<usize, u32>,
    /// Goals whose fields are permanent (pre-built static routes). Never aged or
    /// evicted, so they cause no steady-state rebuild spikes.
    pinned: HashSet<usize>,
}

impl NavFieldCache {
    fn new() -> Self {
        NavFieldCache { fields: HashMap::new(), idle: HashMap::new(), pinned: HashSet::new() }
    }

    fn get(&self, goal: usize) -> Option<&DistField> {
        self.fields.get(&goal)
    }

    fn contains(&self, goal: usize) -> bool {
        self.fields.contains_key(&goal)
    }

    fn insert(&mut self, goal: usize, field: DistField) {
        self.fields.insert(goal, field);
        self.idle.insert(goal, 0);
    }

    /// Insert a permanent field that eviction never touches.
    fn insert_pinned(&mut self, goal: usize, field: DistField) {
        self.fields.insert(goal, field);
        self.pinned.insert(goal);
    }

    fn clear(&mut self) {
        self.fields.clear();
        self.idle.clear();
        self.pinned.clear();
    }

    /// Age fields against this turn's `demanded` goal set: reset demanded ones to
    /// 0, age the rest, evict those past `ttl`, then enforce a hard `cap` (oldest
    /// survivors first). Called once per turn under `&mut map`.
    fn evict(&mut self, demanded: &HashSet<usize>, ttl: u32, cap: usize) {
        for (goal, age) in self.idle.iter_mut() {
            if demanded.contains(goal) { *age = 0; } else { *age = age.saturating_add(1); }
        }
        let mut drop: Vec<usize> = self.idle.iter()
            .filter(|(_, &age)| age > ttl)
            .map(|(&g, _)| g)
            .collect();
        // Cap applies to evictable fields only (pinned aren't tracked in `idle`).
        if self.idle.len().saturating_sub(drop.len()) > cap {
            // Still over cap after TTL eviction: drop the oldest survivors too.
            let mut survivors: Vec<(usize, u32)> = self.idle.iter()
                .filter(|(g, _)| !drop.contains(g))
                .map(|(&g, &a)| (g, a))
                .collect();
            survivors.sort_unstable_by(|a, b| b.1.cmp(&a.1)); // oldest first
            let over = self.idle.len() - drop.len() - cap;
            drop.extend(survivors.into_iter().take(over).map(|(g, _)| g));
        }
        for g in drop {
            self.fields.remove(&g);
            self.idle.remove(&g);
        }
    }
}

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
    /// Resident cache of static-terrain flow fields, keyed by goal tile index.
    /// Built lazily via [`Map::ensure_field`]/[`Map::ensure_field_bounded`] and
    /// shared across all agents navigating to that goal. Evicted once their
    /// goal goes undemanded (see [`Map::evict_fields`]).
    nav_fields: NavFieldCache,
    /// Shared, read-only patrol routes as ordered loops of waypoints. Built once
    /// at map generation and referenced by `Profile::Patrol` via index, so many patrollers
    /// share the same waypoint cells which lets their navigation amortize onto shared flow fields.
    /// Append ad-hoc routes via [`Map::register_patrol_route`].
    pub patrol_routes: Vec<Vec<Point>>,
    /// When `false`, the flow-field layer is bypassed entirely: [`Map::field_step`]
    /// returns `None` and the Step 0 pre-pass skips building/evicting fields, so
    /// all AI navigation falls back to A*. Defaults to `true`.
    pub use_flow_fields: bool,
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
        println!("Generating map");
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
          nav_fields: NavFieldCache::new(),
          patrol_routes: Vec::new(),
          use_flow_fields: false,
        };

        let mut generated_blocks = generate_block_grid(size_in_blocks, rng);
        while generated_blocks.is_none() {
          generated_blocks = generate_block_grid(size_in_blocks, rng);
        }
        let blocks = generated_blocks.unwrap();
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
        if map.use_flow_fields {
            map.prebuild_patrol_fields();
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
            nav_fields: NavFieldCache::new(),
            patrol_routes: Vec::new(),
            use_flow_fields: true,
        }
    }

    /// Append a patrol route and return its id.
    pub fn register_patrol_route(&mut self, route: Vec<Point>) -> usize {
        self.patrol_routes.push(route);
        self.patrol_routes.len() - 1
    }

    /// Pre-build a permanent (pinned) full-map flow field for every distinct
    /// patrol-route waypoint.
    fn prebuild_patrol_fields(&mut self) {
        let mut goals: Vec<usize> = self.patrol_routes.iter()
            .flatten()
            .map(|&p| self.pos_idx(p))
            .collect();
        goals.sort_unstable();
        goals.dedup();

        #[cfg(debug_assertions)]
        println!("Building {} navfields", goals.len());

        for goal in goals {
            if !self.nav_fields.contains(goal) {
                let field = crate::build_field(goal, self);
                self.nav_fields.insert_pinned(goal, field);
            }
        }
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
    /// waypoint counts are bounded so each pinned full-map flow field stays cheap.
    fn create_patrol_routes(&mut self, spawn_map: &SpawnMap, rng: &mut RandomNumberGenerator) {
        const MIN_REGION_TILES: usize = 1024;
        const TILES_PER_ROUTE:  usize = 40_000;
        const MAX_ROUTES:       usize = 8;
        const MAX_WAYPOINTS:    usize = 6;

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
        println!("Created {} patrol routes", self.patrol_routes.len());
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

    /// Ensure a resident full-map flow field toward `goal` exists (see
    /// [`Map::ensure_field_bounded`]).
    pub fn ensure_field(&mut self, goal: usize) {
        self.ensure_field_bounded(goal, u32::MAX);
    }

    /// Ensure a resident flow field toward `goal` exists, building it once over
    /// static terrain if absent, flooding no further than `max_cost` (pass
    /// `u32::MAX` for full-map coverage).
    pub fn ensure_field_bounded(&mut self, goal: usize, max_cost: u32) {
        if !self.nav_fields.contains(goal) {
            let field = crate::build_field_bounded(goal, self, max_cost);
            self.nav_fields.insert(goal, field);
        }
    }

    /// The resident flow field toward `goal`, if one has been built.
    pub fn field_for(&self, goal: usize) -> Option<&DistField> {
        self.nav_fields.get(goal)
    }

    /// Next step from `from` down the flow field toward `goal`.
    /// Callers should fall back to A* on `None`.
    pub fn field_step(&self, from: usize, goal: usize) -> Option<usize> {
        if !self.use_flow_fields {
            return None;
        }
        self.field_for(goal).and_then(|f| f.step(from, self))
    }

    /// Age and evict resident fields against this turn's `demanded` goal set:
    /// fields whose goal went undemanded for more than `ttl` turns are dropped,
    /// and the cache is capped at `cap` entries (oldest survivors evicted first).
    pub fn evict_fields(&mut self, demanded: &HashSet<usize>, ttl: u32, cap: usize) {
        self.nav_fields.evict(demanded, ttl, cap);
    }

    /// Drop all resident flow fields. Call whenever terrain changes.
    pub fn invalidate_fields(&mut self) {
        self.nav_fields.clear();
    }

    /// Neighbours passable over static terrain, ignoring pawn occupancy.
    /// Costs mirror [`Map::get_available_exits`].
    pub fn terrain_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width as i32;
        let y = idx as i32 / self.width as i32;
        let w = self.width as usize;

        if self.terrain_passable(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.terrain_passable(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.terrain_passable(x, y-1) { exits.push((idx-w, 1.0)) };
        if self.terrain_passable(x, y+1) { exits.push((idx+w, 1.0)) };

        if self.terrain_passable(x-1, y-1) { exits.push(((idx-w)-1, 1.45)); }
        if self.terrain_passable(x+1, y-1) { exits.push(((idx-w)+1, 1.45)); }
        if self.terrain_passable(x-1, y+1) { exits.push(((idx+w)-1, 1.45)); }
        if self.terrain_passable(x+1, y+1) { exits.push(((idx+w)+1, 1.45)); }

        exits
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

    #[test]
    fn undemanded_fields_are_evicted_past_ttl() {
        let mut map = Map::new_empty_map(30);
        let goal = map.xy_idx(10, 10);
        map.ensure_field(goal);
        assert!(map.field_for(goal).is_some());

        let empty: HashSet<usize> = HashSet::new();
        for _ in 0..5 { map.evict_fields(&empty, 5, 64); } // within TTL
        assert!(map.field_for(goal).is_some());
        for _ in 0..3 { map.evict_fields(&empty, 5, 64); } // now past TTL
        assert!(map.field_for(goal).is_none());
    }

    #[test]
    fn demanded_fields_survive_eviction() {
        let mut map = Map::new_empty_map(30);
        let goal = map.xy_idx(10, 10);
        map.ensure_field(goal);
        let demanded: HashSet<usize> = std::iter::once(goal).collect();
        for _ in 0..100 { map.evict_fields(&demanded, 5, 64); }
        assert!(map.field_for(goal).is_some());
    }

    #[test]
    fn invalidate_clears_all_fields() {
        let mut map = Map::new_empty_map(30);
        let a = map.xy_idx(10, 10);
        let b = map.xy_idx(20, 20);
        map.ensure_field(a);
        map.ensure_field(b);
        assert!(map.field_for(a).is_some() && map.field_for(b).is_some());
        map.invalidate_fields();
        assert!(map.field_for(a).is_none() && map.field_for(b).is_none());
    }

    #[test]
    fn bounded_field_limits_reach() {
        let mut map = Map::new_empty_map(80);
        let goal = map.xy_idx(40, 40);
        map.ensure_field_bounded(goal, 100); // ~10 orthogonal steps
        let near = map.xy_idx(43, 40); // cost 30 <= 100: inside the horizon
        let far  = map.xy_idx(70, 70); // well beyond the bound
        let field = map.field_for(goal).unwrap();
        assert!(field.step(near, &map).is_some());
        assert!(field.step(far,  &map).is_none());
    }
}