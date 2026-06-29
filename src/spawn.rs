use rltk::Point;
use std::collections::HashMap;
use crate::{Map, TileType};

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
    /// True when the region is large enough to be treated as an open area or room.
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
    matches!(tile, TileType::Floor | TileType::Ground | TileType::Road | TileType::Doorway)
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

        let is_room = tiles.len() >= MIN_ROOM_SIZE;
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
