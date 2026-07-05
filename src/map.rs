use rltk::{BaseMap, Algorithm2D, Point, RandomNumberGenerator};
use std::cmp::{max, min};
use std::collections::HashMap;
use crate::entity::Pawn;
use crate::item::Item;
use crate::tile::TileType;
use crate::block::*;
use crate::DistField;
use super::{GameError, Error};

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
    /// Built lazily via [`Map::ensure_field`] and shared across all agents
    /// navigating to that goal. Terrain-only (see [`DistField`]), so entries stay
    /// valid as pawns move and never need rebuilding for a static map.
    nav_fields: HashMap<usize, DistField>,
    /// Shared, read-only patrol routes as ordered loops of waypoints. Built once
    /// at map generation (concentric rings) and referenced by `Profile::Patrol`
    /// via index, so many patrollers share the same waypoint cells — which lets
    /// their navigation amortize onto shared flow fields. Append ad-hoc routes
    /// via [`Map::register_patrol_route`].
    pub patrol_routes: Vec<Vec<Point>>,
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

    pub fn new_game_map(size_in_blocks: usize, rng: &mut RandomNumberGenerator) -> Map {
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
          nav_fields: HashMap::new(),
          patrol_routes: Vec::new(),
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

        map.build_patrol_rings();
        return map;
    }

    pub fn new_empty_map(map_width: usize, map_height: usize) -> Map {
        let tile_count = map_width * map_height;
        Map {
            tiles: vec![TileType::Ground; tile_count],
            width: map_width,
            height: map_height,
            revealed_tiles: vec![false; tile_count],
            visible_tiles: vec![false; tile_count],
            pawns: vec![None; tile_count],
            items: vec![None; tile_count],
            fov_blocked: vec![false; tile_count],
            nav_fields: HashMap::new(),
            patrol_routes: Vec::new(),
        }
    }

    /// Append a patrol route and return its id. Used for ad-hoc / test routes;
    /// the standard concentric rings are built at map generation.
    pub fn register_patrol_route(&mut self, route: Vec<Point>) -> usize {
        self.patrol_routes.push(route);
        self.patrol_routes.len() - 1
    }

    /// Build the default shared patrol routes: concentric rectangular rings
    /// centred on the map, from a ~100-tile-wide innermost ring out toward the
    /// edges. Each route is the four ring corners (a closed loop); corners are
    /// snapped to the nearest walkable tile so patrollers can actually reach them.
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

    /// Nearest walkable-terrain tile to `p` via an expanding ring search. Falls
    /// back to the clamped point if none is found (not expected on a real map).
    fn snap_to_walkable(&self, p: Point) -> Point {
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
    /// the map centre — distributes patrollers across the concentric rings.
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

    /// Ensure a resident flow field toward `goal` exists, building it once over
    /// static terrain if absent. Idempotent and cheap once warm — the caller may
    /// invoke it every turn for the goals it needs.
    pub fn ensure_field(&mut self, goal: usize) {
        if !self.nav_fields.contains_key(&goal) {
            let field = crate::build_field(goal, self);
            self.nav_fields.insert(goal, field);
        }
    }

    /// The resident flow field toward `goal`, if one has been built.
    pub fn field_for(&self, goal: usize) -> Option<&DistField> {
        self.nav_fields.get(&goal)
    }

    /// Neighbours passable over **static terrain**, ignoring transient pawn
    /// occupancy. Used to build resident [`DistField`]s that stay valid as
    /// entities move. Costs mirror [`Map::get_available_exits`] (1.0 / 1.45).
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
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, index: usize) -> bool {
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

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
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
    
    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}