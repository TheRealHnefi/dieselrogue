use rltk::{Point, RandomNumberGenerator};
use std::collections::HashMap;
use crate::{Map, TileType, World, Direction, CombatTactic, Item, EntityKind, BLOCK_SIZE};

/// Minimum number of tiles for a connected region to be treated as a room rather
/// than a corridor or stub.
const MIN_ROOM_SIZE: usize = 16;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Semantic classification of a spawn candidate.
#[derive(Clone, Copy, PartialEq)]
pub enum SpawnCategory {
    /// Exactly one passable cardinal neighbour — the end of a passage.
    /// Good for item placement and ambush positions.
    DeadEnd,
    /// Three or more passable cardinal neighbours — a corridor crossing or T-junction.
    /// Good for stationary guards.
    Junction,
    /// All four cardinal neighbours are passable and the tile is inside a large region.
    /// Good for roaming AI or scattered items.
    RoomInterior,
}

/// A single candidate position for placing an entity or item.
#[derive(Clone)]
pub struct SpawnPoint {
    pub idx: usize,
    pub pos: Point,
    pub category: SpawnCategory,
}

/// A contiguous group of passable tiles (4-connected).
#[derive(Clone)]
pub struct Region {
    /// All tile indices that belong to this region.
    pub tiles: Vec<usize>,
    /// Index of the tile closest to the centroid of the region.
    pub center_idx: usize,
    /// True when the region is large enough to be treated as a room and is indoors.
    pub is_room: bool,
}

/// Output of the spawn analysis pass.
pub struct SpawnMap {
    pub spawn_points: Vec<SpawnPoint>,
    pub regions: Vec<Region>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Analyse `map` and return classified spawn candidates and connected regions.
pub fn analyze(map: &Map) -> SpawnMap {
    let regions = find_regions(map);

    // Tile → region index lookup, built once and shared by classification.
    let mut tile_region: Vec<Option<usize>> = vec![None; map.width * map.height];
    for (ri, region) in regions.iter().enumerate() {
        for &idx in &region.tiles {
            tile_region[idx] = Some(ri);
        }
    }

    let mut spawn_points = Vec::new();
    for idx in 0..map.width * map.height {
        if !tile_passable(map.tiles[idx]) {
            continue;
        }
        let degree = cardinal_passable_count(map, idx);
        let in_room = tile_region[idx]
            .map(|ri| regions[ri].is_room)
            .unwrap_or(false);

        let category = match degree {
            1 => Some(SpawnCategory::DeadEnd),
            3..=4 if !in_room => Some(SpawnCategory::Junction),
            4 if in_room => Some(SpawnCategory::RoomInterior),
            _ => None,
        };

        if let Some(category) = category {
            spawn_points.push(SpawnPoint { idx, pos: map.idx_pos(idx), category });
        }
    }

    SpawnMap { spawn_points, regions }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Returns true for tile types that entities can stand on.
fn tile_passable(tile: TileType) -> bool {
    matches!(tile, TileType::Floor | TileType::Ground | TileType::Road)
}

/// Count how many of the four cardinal neighbours of `idx` are passable.
fn cardinal_passable_count(map: &Map, idx: usize) -> u8 {
    let w = map.width as i32;
    let h = map.height as i32;
    let p = map.idx_pos(idx);
    let mut count = 0u8;
    for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
        let nx = p.x + dx;
        let ny = p.y + dy;
        if nx >= 0 && ny >= 0 && nx < w && ny < h {
            if tile_passable(map.tiles[map.xy_idx(nx, ny)]) {
                count += 1;
            }
        }
    }
    count
}

/// Flood-fill all passable tiles into 4-connected regions.
fn find_regions(map: &Map) -> Vec<Region> {
    let n = map.width * map.height;
    let mut visited = vec![false; n];
    let mut regions = Vec::new();

    for start in 0..n {
        if visited[start] || !tile_passable(map.tiles[start]) {
            continue;
        }

        // BFS using the vec as a queue (index tracks the frontier head).
        let mut queue = vec![start];
        visited[start] = true;
        let mut qi = 0;

        while qi < queue.len() {
            let current = queue[qi];
            qi += 1;
            let p = map.idx_pos(current);
            for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
                let nx = p.x + dx;
                let ny = p.y + dy;
                if nx >= 0 && ny >= 0 && nx < map.width as i32 && ny < map.height as i32 {
                    let ni = map.xy_idx(nx, ny);
                    if !visited[ni] && tile_passable(map.tiles[ni]) {
                        visited[ni] = true;
                        queue.push(ni);
                    }
                }
            }
        }

        // The final queue is exactly the tiles of this region.
        let tiles = queue;

        let cx: i32 = tiles.iter().map(|&i| map.idx_pos(i).x).sum::<i32>()
            / tiles.len() as i32;
        let cy: i32 = tiles.iter().map(|&i| map.idx_pos(i).y).sum::<i32>()
            / tiles.len() as i32;
        let center_idx = *tiles.iter().min_by_key(|&&i| {
            let p = map.idx_pos(i);
            (p.x - cx).abs() + (p.y - cy).abs()
        }).unwrap();

        let is_room = tiles.len() >= MIN_ROOM_SIZE && map.tiles[tiles[0]] == TileType::Floor;
        regions.push(Region { tiles, center_idx, is_room });
    }

    regions
}

// ---------------------------------------------------------------------------
// Zone detection
// ---------------------------------------------------------------------------

/// A set of doorway tiles that fully separates two structural zones.
pub struct ZoneBoundary {
    pub zone_a: usize,
    pub zone_b: usize,
    /// Tile indices of every Doorway tile on this boundary.
    pub door_tiles: Vec<usize>,
}

/// Output of zone analysis.
pub struct ZoneMap {
    /// tile index → zone index.  None for walls, fences, and doorway tiles.
    pub tile_zone: Vec<Option<usize>>,
    /// zone index → all tile indices in that zone.
    pub zones: Vec<Vec<usize>>,
    /// Every boundary between a pair of zones.
    pub boundaries: Vec<ZoneBoundary>,
}

/// Flood-fill the map treating Doorway tiles as walls to find structural zones,
/// then identify the doorway tiles that separate each pair of adjacent zones.
pub fn find_zones(map: &Map) -> ZoneMap {
    let n = map.width * map.height;
    let mut tile_zone: Vec<Option<usize>> = vec![None; n];
    let mut zones: Vec<Vec<usize>> = Vec::new();

    for start in 0..n {
        if tile_zone[start].is_some() || !zone_passable(map.tiles[start]) {
            continue;
        }
        let zone_idx = zones.len();
        let mut tiles = Vec::new();
        let mut queue = vec![start];
        tile_zone[start] = Some(zone_idx);
        let mut qi = 0;
        while qi < queue.len() {
            let current = queue[qi];
            qi += 1;
            tiles.push(current);
            let p = map.idx_pos(current);
            for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
                let nx = p.x + dx;
                let ny = p.y + dy;
                if nx >= 0 && ny >= 0 && nx < map.width as i32 && ny < map.height as i32 {
                    let ni = map.xy_idx(nx, ny);
                    if tile_zone[ni].is_none() && zone_passable(map.tiles[ni]) {
                        tile_zone[ni] = Some(zone_idx);
                        queue.push(ni);
                    }
                }
            }
        }
        zones.push(tiles);
    }

    // Every Doorway tile that touches exactly two distinct zones is a boundary tile.
    let mut boundary_map: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for idx in 0..n {
        if map.tiles[idx] != TileType::Doorway {
            continue;
        }
        let p = map.idx_pos(idx);
        let mut adjacent_zones: Vec<usize> = Vec::new();
        for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
            let nx = p.x + dx;
            let ny = p.y + dy;
            if nx >= 0 && ny >= 0 && nx < map.width as i32 && ny < map.height as i32 {
                let ni = map.xy_idx(nx, ny);
                if let Some(z) = tile_zone[ni] {
                    if !adjacent_zones.contains(&z) {
                        adjacent_zones.push(z);
                    }
                }
            }
        }
        if adjacent_zones.len() == 2 {
            let key = (adjacent_zones[0].min(adjacent_zones[1]),
                       adjacent_zones[0].max(adjacent_zones[1]));
            boundary_map.entry(key).or_default().push(idx);
        }
    }

    let boundaries = boundary_map.into_iter()
        .map(|((a, b), door_tiles)| ZoneBoundary { zone_a: a, zone_b: b, door_tiles })
        .collect();

    ZoneMap { tile_zone, zones, boundaries }
}

/// Passable for zone analysis: Doorway counts as a wall so it forms zone boundaries.
fn zone_passable(tile: TileType) -> bool {
    matches!(tile, TileType::Floor | TileType::Ground | TileType::Road)
}

/// BFS over the zone graph to compute how many boundaries separate each zone from
/// `start_zone`.  Unreachable zones get `usize::MAX`.
pub fn zone_depths(zone_map: &ZoneMap, start_zone: usize) -> Vec<usize> {
    let n = zone_map.zones.len();
    let mut depth = vec![usize::MAX; n];
    if start_zone >= n { return depth; }

    let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
    for b in &zone_map.boundaries {
        adj[b.zone_a].push(b.zone_b);
        adj[b.zone_b].push(b.zone_a);
    }

    depth[start_zone] = 0;
    let mut queue = vec![start_zone];
    let mut qi = 0;
    while qi < queue.len() {
        let cur = queue[qi]; qi += 1;
        for &nb in &adj[cur] {
            if depth[nb] == usize::MAX {
                depth[nb] = depth[cur] + 1;
                queue.push(nb);
            }
        }
    }
    depth
}

/// Randomly marks zones as interesting based on whether they contain rooms.
/// Each room in a zone has a 1-in-2 chance to flag the zone as interesting;
/// a zone is interesting if any of its rooms passes the check.
pub fn mark_interesting_zones(
    zone_map: &ZoneMap,
    spawn_map: &SpawnMap,
    rng: &mut RandomNumberGenerator,
) -> Vec<bool> {
    let mut interesting = vec![false; zone_map.zones.len()];
    for region in &spawn_map.regions {
        if !region.is_room { continue; }
        if let Some(zi) = zone_map.tile_zone[region.center_idx] {
            if rng.range(0, 2) == 0 {
                interesting[zi] = true;
            }
        }
    }
    interesting
}

// ---------------------------------------------------------------------------
// Tank spawn analysis
// ---------------------------------------------------------------------------

/// A door at least this many tiles wide is considered a hangar entrance.
const MIN_HANGAR_DOOR_WIDTH: usize = 3;

/// Candidate tile indices for tank placement.
pub struct TankSpawnMap {
    /// Open road tiles (not dead-end stubs).
    pub road_tiles: Vec<usize>,
    /// Floor tiles inside rooms that have a wide enough entrance door.
    pub hangar_tiles: Vec<usize>,
}

/// Find tiles suitable for spawning tanks: open road sections and hangars
/// (rooms whose widest adjacent doorway meets `MIN_HANGAR_DOOR_WIDTH`).
pub fn find_tank_spawns(map: &Map, regions: &[Region]) -> TankSpawnMap {
    let n = map.width * map.height;

    let road_tiles: Vec<usize> = (0..n)
        .filter(|&idx| {
            map.tiles[idx] == TileType::Road
                && cardinal_passable_count(map, idx) >= 2
        })
        .collect();

    let mut hangar_tiles = Vec::new();
    for region in regions.iter().filter(|r| r.is_room) {
        if region_has_wide_door(map, region) {
            hangar_tiles.extend(
                region.tiles.iter().copied().filter(|&i| tile_passable(map.tiles[i]))
            );
        }
    }

    TankSpawnMap { road_tiles, hangar_tiles }
}

/// Returns true if any tile in the region is adjacent to a doorway run at least
/// `MIN_HANGAR_DOOR_WIDTH` tiles wide.
fn region_has_wide_door(map: &Map, region: &Region) -> bool {
    for &tile_idx in &region.tiles {
        let p = map.idx_pos(tile_idx);
        for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
            let nx = p.x + dx;
            let ny = p.y + dy;
            if nx < 0 || ny < 0 || nx >= map.width as i32 || ny >= map.height as i32 {
                continue;
            }
            if map.tiles[map.xy_idx(nx, ny)] != TileType::Doorway {
                continue;
            }
            let h = doorway_run(map, nx, ny, 1, 0);
            let v = doorway_run(map, nx, ny, 0, 1);
            if h >= MIN_HANGAR_DOOR_WIDTH || v >= MIN_HANGAR_DOOR_WIDTH {
                return true;
            }
        }
    }
    false
}

/// Count consecutive Doorway tiles through `(sx, sy)` along axis `(dx, dy)`,
/// scanning in both directions from the starting tile (inclusive).
fn doorway_run(map: &Map, sx: i32, sy: i32, dx: i32, dy: i32) -> usize {
    let mut len = 1usize;
    for sign in [1i32, -1] {
        let (mut x, mut y) = (sx + dx * sign, sy + dy * sign);
        while x >= 0 && y >= 0 && x < map.width as i32 && y < map.height as i32 {
            if map.tiles[map.xy_idx(x, y)] == TileType::Doorway {
                len += 1;
                x += dx * sign;
                y += dy * sign;
            } else {
                break;
            }
        }
    }
    len
}

// ---------------------------------------------------------------------------
// World population passes
// ---------------------------------------------------------------------------

const GUARD_DENSITY: f32 = 0.5;
const KEY_COPIES_PER_COLOR: usize = 3;

fn chebyshev(a: Point, b: Point) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

fn guard_too_close(pos: Point, placed: &[Point], min_dist: i32) -> bool {
    placed.iter().any(|&p| chebyshev(pos, p) < min_dist)
}

fn dir_toward(from: Point, to: Point) -> Direction {
    let dx = (to.x - from.x).signum();
    let dy = (to.y - from.y).signum();
    match (dx, dy) {
        ( 0, -1) => Direction::Up,
        ( 1, -1) => Direction::UpRight,
        ( 1,  0) => Direction::Right,
        ( 1,  1) => Direction::DownRight,
        ( 0,  1) => Direction::Down,
        (-1,  1) => Direction::DownLeft,
        (-1,  0) => Direction::Left,
        (-1, -1) => Direction::UpLeft,
        _        => Direction::Up,
    }
}

fn fy_shuffle<T>(v: &mut Vec<T>, rng: &mut RandomNumberGenerator) {
    for i in (1..v.len()).rev() {
        let j = rng.range(0, (i + 1) as i32) as usize;
        v.swap(i, j);
    }
}

fn is_spawnable(tile: TileType) -> bool {
    matches!(tile, TileType::Floor | TileType::Ground | TileType::Road)
}

/// BFS over the zone graph treating locked doors (whose color is not in `unlocked`) as
/// impassable walls.  Returns a bitmask of which zones the player can reach.
fn reachable_zones(
    adj: &[Vec<(usize, Option<usize>)>],
    start_zone: usize,
    n_zones: usize,
    unlocked: &std::collections::HashSet<usize>,
) -> Vec<bool> {
    let mut reachable = vec![false; n_zones];
    if start_zone >= n_zones { return reachable; }
    reachable[start_zone] = true;
    let mut queue = vec![start_zone];
    let mut qi = 0;
    while qi < queue.len() {
        let cur = queue[qi]; qi += 1;
        for &(nb, color_opt) in &adj[cur] {
            if reachable[nb] { continue; }
            if color_opt.map_or(true, |c| unlocked.contains(&c)) {
                reachable[nb] = true;
                queue.push(nb);
            }
        }
    }
    reachable
}

enum BoundaryKind { OuterWall, InnerWall, Regular }

fn classify_boundary(map: &Map, boundary: &ZoneBoundary) -> BoundaryKind {
    let size = map.width / BLOCK_SIZE;
    let inner_margin = size / 4;
    let inner_min = inner_margin;
    let inner_max = size - 1 - inner_margin;
    let has_inner_ring = inner_margin > 0 && inner_min < inner_max;

    let mut is_outer = false;
    let mut is_inner = false;

    for &tile_idx in &boundary.door_tiles {
        let p = map.idx_pos(tile_idx);
        let bi = p.x as usize / BLOCK_SIZE;
        let bj = p.y as usize / BLOCK_SIZE;
        if bi == 0 || bi == size - 1 || bj == 0 || bj == size - 1 {
            is_outer = true;
        }
        if has_inner_ring
            && bi >= inner_min && bi <= inner_max
            && bj >= inner_min && bj <= inner_max
            && (bi == inner_min || bi == inner_max || bj == inner_min || bj == inner_max)
        {
            is_inner = true;
        }
    }

    if is_outer { BoundaryKind::OuterWall }
    else if is_inner { BoundaryKind::InnerWall }
    else { BoundaryKind::Regular }
}

impl World {
    /// Stationary guards adjacent to doorways.
    pub(crate) fn spawn_sentinels(
        &mut self,
        zone_map: &ZoneMap,
        interesting: &[bool],
        placed: &mut Vec<Point>,
        n: &mut usize,
        rng: &mut RandomNumberGenerator,
    ) {
        const RATE: f32 = 0.30;
        const MIN_DIST: i32 = 5;

        let cardinals = [(0i32, -1i32), (1, 0), (0, 1), (-1, 0)];

        let mut candidates: Vec<(Point, Point, bool)> = Vec::new();
        for idx in 0..self.map.width * self.map.height {
            if self.map.tiles[idx] != TileType::Doorway {
                continue;
            }
            let door_pos = self.map.idx_pos(idx);

            let guards_interesting = cardinals.iter().any(|&(dx, dy)| {
                let nx = door_pos.x + dx;
                let ny = door_pos.y + dy;
                if nx < 0 || ny < 0 || nx >= self.map.width as i32 || ny >= self.map.height as i32 {
                    return false;
                }
                let ni = self.map.xy_idx(nx, ny);
                zone_map.tile_zone[ni].map_or(false, |zi| interesting.get(zi).copied().unwrap_or(false))
            });

            // Place guard 3 tiles away from door.
            for &(dx, dy) in &[(0i32, -3i32), (3, 0), (0, 3), (-3, 0)] {
                let nx = door_pos.x + dx;
                let ny = door_pos.y + dy;
                if nx < 0 || ny < 0 || nx >= self.map.width as i32 || ny >= self.map.height as i32 {
                    continue;
                }
                let nidx = self.map.xy_idx(nx, ny);
                if is_spawnable(self.map.tiles[nidx]) {
                    candidates.push((Point::new(nx, ny), door_pos, guards_interesting));
                    break;
                }
            }
        }

        // Shuffle for randomness within each tier, then stable-sort interesting doors first.
        fy_shuffle(&mut candidates, rng);
        candidates.sort_by_key(|&(_, _, gi)| if gi { 0usize } else { 1 });

        let target = ((candidates.len() as f32) * RATE * GUARD_DENSITY) as usize;
        let mut count = 0;
        for (guard_pos, door_pos, _) in candidates {
            if count >= target { break; }
            if guard_too_close(guard_pos, placed, MIN_DIST) { continue; }

            let facing = if self.map.get_tile(guard_pos.x, guard_pos.y) == TileType::Floor {
                dir_toward(guard_pos, door_pos)
            } else {
                dir_toward(door_pos, guard_pos)
            };
            *n += 1;
            if self.create_guard_actor(guard_pos, facing, format!("Sentinel {}", n), CombatTactic::Hold).is_ok() {
                placed.push(guard_pos);
                count += 1;
            }
        }
        println!("  Sentinels: {}", count);
    }

    /// Patrol guards following pathfinder-computed road routes.
    pub(crate) fn spawn_patrollers(
        &mut self,
        spawn_map: &SpawnMap,
        placed: &mut Vec<Point>,
        n: &mut usize,
        rng: &mut RandomNumberGenerator,
    ) {
        const RATE: f32 = 0.005;
        const MIN_DIST: i32 = 15;
        const MIN_PATROL_DIST: i32 = 20;
        const MAX_PATROL_DIST: i32 = 80;

        let junctions: Vec<usize> = spawn_map.spawn_points.iter()
            .enumerate()
            .filter(|(_, sp)| {
                matches!(sp.category, SpawnCategory::Junction) &&
                matches!(self.map.tiles[sp.idx], TileType::Road)
            })
            .map(|(i, _)| i)
            .collect();

        let mut order: Vec<usize> = (0..junctions.len()).collect();
        fy_shuffle(&mut order, rng);

        let target = ((junctions.len() as f32) * RATE * GUARD_DENSITY) as usize;
        let mut used: Vec<usize> = Vec::new();
        let mut count = 0;

        for &oi in &order {
            if count >= target { break; }
            let ai = junctions[oi];
            if used.contains(&ai) { continue; }
            let a_pos = {
                let sp = &spawn_map.spawn_points[ai];
                if guard_too_close(sp.pos, placed, MIN_DIST) { continue; }
                sp.pos
            };

            // Keep the existing junction-pair gate for placement/spacing parity:
            // only spawn where a suitably distant second junction exists.
            let bi = order.iter()
                .map(|&oi2| junctions[oi2])
                .find(|&bi| {
                    if bi == ai || used.contains(&bi) { return false; }
                    let d = chebyshev(spawn_map.spawn_points[bi].pos, a_pos);
                    d >= MIN_PATROL_DIST && d <= MAX_PATROL_DIST
                });
            let bi = match bi { Some(b) => b, None => continue };
            let b_pos = spawn_map.spawn_points[bi].pos;

            // Assign to the shared concentric ring nearest this spawn, starting at
            // the closest waypoint on it. Patrollers thus share a handful of routes
            // (and their flow fields) rather than each carrying a bespoke path.
            let route_id = self.map.nearest_patrol_route(a_pos);
            let waypoint_index = self.map.nearest_waypoint_index(route_id, a_pos);

            let facing = dir_toward(a_pos, b_pos);
            *n += 1;
            if self.create_patrol_actor(a_pos, facing, format!("Patroller {}", n), route_id, waypoint_index, CombatTactic::Pursue).is_ok() {
                placed.push(a_pos);
                used.push(ai);
                used.push(bi);
                count += 1;
            }
        }
        println!("  Patrollers: {}", count);
    }

    pub(crate) fn spawn_loot(
        &mut self,
        zone_map: &ZoneMap,
        spawn_map: &SpawnMap,
        depths: &[usize],
        rng: &mut RandomNumberGenerator,
    ) {
        const TOTAL_LOOT: usize = 400;

        type MakeItem = fn() -> Item;
        let pool: &[MakeItem] = &[
            Item::pistol, Item::flare_gun, Item::knife,
            Item::revolver, Item::shock_pistol, Item::submachine_gun,
            Item::grenade, Item::fire_grenade, Item::flashbang,
            Item::bulletproof_vest,
            Item::bolt_action_rifle, Item::semi_auto_rifle,
            Item::assault_rifle, Item::machinegun,
            Item::shock_carbine, Item::flamethrower,
            Item::shock_grenade,
            Item::rotary_machinegun, Item::shock_cannon,
            Item::rocket_launcher, Item::multi_rocket_launcher,
        ];

        let item_meta: Vec<(MakeItem, u8)> =
            pool.iter().map(|&f| (f, f().rarity)).collect();
        let weighted_pool: Vec<(MakeItem, u8)> = item_meta.iter()
            .flat_map(|&(f, r)| {
                std::iter::repeat((f, r)).take(4usize.saturating_sub(r as usize))
            })
            .collect();
        if weighted_pool.is_empty() { return; }

        let nz = zone_map.zones.len();

        let mut indoor_spawns: Vec<Vec<Point>> = vec![vec![]; nz];
        for sp in &spawn_map.spawn_points {
            let Some(zi) = zone_map.tile_zone[sp.idx] else { continue };
            if depths[zi] == usize::MAX { continue; }
            if self.map.tiles[sp.idx] == TileType::Floor {
                indoor_spawns[zi].push(sp.pos);
            }
        }

        let max_depth = (0..nz)
            .filter(|&zi| depths[zi] != usize::MAX && !indoor_spawns[zi].is_empty())
            .map(|zi| depths[zi])
            .max()
            .unwrap_or(0);

        let mut zones_by_depth: Vec<Vec<usize>> = vec![vec![]; max_depth + 1];
        for zi in 0..nz {
            let d = depths[zi];
            if d != usize::MAX && d <= max_depth && !indoor_spawns[zi].is_empty() {
                zones_by_depth[d].push(zi);
            }
        }

        if zones_by_depth.iter().all(|v| v.is_empty()) { return; }

        let mut zone_has_item = vec![false; nz];
        let mut placed = 0usize;

        for _ in 0..TOTAL_LOOT {
            let (make, rarity) = weighted_pool[rng.range(0, weighted_pool.len() as i32) as usize];

            let base = (rarity as usize * max_depth) / 3;
            let jitter = rng.range(-1i32, 4);
            let mut target = ((base as i32 + jitter).max(0) as usize).min(max_depth);

            if target < 4 && max_depth >= 4 && rng.range(0, 50) < 49 {
                target = rng.range(4, max_depth as i32 + 1) as usize;
            }

            let Some(depth) = (0..=max_depth)
                .filter(|&d| zones_by_depth[d].iter().any(|&zi| !zone_has_item[zi]))
                .min_by_key(|&d| ((d as i32) - (target as i32)).abs())
            else { continue };

            let available: Vec<usize> = zones_by_depth[depth].iter()
                .copied()
                .filter(|&zi| !zone_has_item[zi])
                .collect();
            let zi = available[rng.range(0, available.len() as i32) as usize];

            let spawns = &indoor_spawns[zi];
            let pos = spawns[rng.range(0, spawns.len() as i32) as usize];

            let _ = self.add_item(pos, make());
            zone_has_item[zi] = true;
            placed += 1;
        }

        println!("Spawned {} loot items.", placed);
    }

    /// First pass: assign lock colors to door entities.
    /// Returns `boundary_colors[bi]` = the color assigned to boundary `bi`,
    /// or `None` if that boundary stays unlocked.
    pub(crate) fn assign_door_colors(
        &mut self,
        zone_map: &ZoneMap,
        depths: &[usize],
        interesting: &[bool],
    ) -> Vec<Option<usize>> {
        const OUTER_WALL_COLOR: usize = 15; // Gold
        const INNER_WALL_COLOR: usize = 13; // Silver

        let mut boundary_colors = vec![None; zone_map.boundaries.len()];

        let regular_colors: Vec<usize> = (0..crate::components::KEY_COLORS.len())
            .filter(|&c| c != OUTER_WALL_COLOR && c != INNER_WALL_COLOR)
            .collect();

        let mut order: Vec<usize> = (0..zone_map.boundaries.len()).collect();
        order.sort_by_key(|&bi| {
            let b = &zone_map.boundaries[bi];
            depths[b.zone_a].min(depths[b.zone_b])
        });

        let mut regular_color_idx = 0usize;
        let mut locked_count = 0usize;

        for &bi in &order {
            let b = &zone_map.boundaries[bi];
            let kind = classify_boundary(&self.map, b);
            let deep = if depths[b.zone_a] <= depths[b.zone_b] { b.zone_b } else { b.zone_a };

            let color_opt: Option<usize> = match kind {
                BoundaryKind::OuterWall => Some(OUTER_WALL_COLOR),
                BoundaryKind::InnerWall => Some(INNER_WALL_COLOR),
                BoundaryKind::Regular => {
                    if interesting.get(deep).copied().unwrap_or(false) {
                        let c = regular_colors[regular_color_idx % regular_colors.len()];
                        regular_color_idx += 1;
                        Some(c)
                    } else {
                        None
                    }
                }
            };

            boundary_colors[bi] = color_opt;

            if let Some(color) = color_opt {
                locked_count += 1;
                for &tile_idx in &b.door_tiles {
                    if let Some(pawn) = &self.map.pawns[tile_idx] {
                        let eid = pawn.entity_id;
                        if self.entities[eid].kind == EntityKind::Door {
                            self.entities[eid].color = Some(color);
                        }
                    }
                }
            }
        }

        println!("Locked {} of {} zone boundaries.", locked_count, order.len());
        boundary_colors
    }

    /// Second pass: scatter keys.  For each locked color, places up to
    /// `KEY_COPIES_PER_COLOR` keys across zones the player can reach *before*
    /// needing that color, guaranteeing solvability.  At most one key per zone.
    pub(crate) fn place_zone_keys(
        &mut self,
        zone_map: &ZoneMap,
        spawn_map: &SpawnMap,
        depths: &[usize],
        boundary_colors: &[Option<usize>],
        start_zone: usize,
        rng: &mut RandomNumberGenerator,
    ) {
        let n_zones = zone_map.zones.len();

        let mut adj: Vec<Vec<(usize, Option<usize>)>> = vec![vec![]; n_zones];
        for (bi, b) in zone_map.boundaries.iter().enumerate() {
            let color = boundary_colors[bi];
            adj[b.zone_a].push((b.zone_b, color));
            adj[b.zone_b].push((b.zone_a, color));
        }

        let zone_dead_ends: Vec<Vec<Point>> = (0..n_zones).map(|zi| {
            let zone_set: std::collections::HashSet<usize> =
                zone_map.zones[zi].iter().copied().collect();
            spawn_map.spawn_points.iter()
                .filter(|sp| sp.category == SpawnCategory::DeadEnd && zone_set.contains(&sp.idx))
                .map(|sp| sp.pos)
                .collect()
        }).collect();

        let mut color_first_depth: HashMap<usize, usize> = HashMap::new();
        for (bi, b) in zone_map.boundaries.iter().enumerate() {
            if let Some(color) = boundary_colors[bi] {
                let d = depths[b.zone_a].min(depths[b.zone_b]);
                color_first_depth.entry(color)
                    .and_modify(|e| *e = (*e).min(d))
                    .or_insert(d);
            }
        }
        let mut color_order: Vec<usize> = color_first_depth.keys().copied().collect();
        color_order.sort_by_key(|&c| color_first_depth[&c]);

        let mut zone_has_key = vec![false; n_zones];
        let mut unlocked: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut total_placed = 0usize;

        for color in color_order {
            let reachable = reachable_zones(&adj, start_zone, n_zones, &unlocked);

            let mut candidates: Vec<usize> = (0..n_zones)
                .filter(|&zi| reachable[zi] && !zone_has_key[zi])
                .collect();

            for i in (1..candidates.len()).rev() {
                let j = rng.range(0, (i + 1) as i32) as usize;
                candidates.swap(i, j);
            }
            candidates.truncate(KEY_COPIES_PER_COLOR);

            for zi in candidates {
                let key_pos = if !zone_dead_ends[zi].is_empty() {
                    let picks = &zone_dead_ends[zi];
                    picks[rng.range(0, picks.len() as i32) as usize]
                } else {
                    let tiles = &zone_map.zones[zi];
                    self.map.idx_pos(tiles[rng.range(0, tiles.len() as i32) as usize])
                };
                let _ = self.add_item(key_pos, Item::key(color));
                zone_has_key[zi] = true;
                total_placed += 1;
            }

            unlocked.insert(color);
        }

        println!("Placed {} keys across {} color(s).", total_placed, color_first_depth.len());
    }
}
