use rltk::{Point, RandomNumberGenerator};
use std::collections::HashMap;
use crate::{Map, TileType, World, Direction, CombatTactic, Item, EntityKind, BLOCK_SIZE, AI, ActorAI, Profile};
use crate::item::MakeItem;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Semantic classification of a spawn candidate.
#[derive(Clone, Copy, PartialEq)]
pub enum SpawnCategory {
    /// Exactly one passable cardinal neighbour — the end of a passage.
    DeadEnd,
    /// Three or more passable cardinal neighbours.
    Junction,
    /// All four cardinal neighbours are passable and the tile is inside a large region.
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
    /// True if the region is likely to contain interesting things
    pub is_interesting: bool,
    /// Degrees of separation from player start position.
    /// Unreachable if equal to usize::MAX
    pub depth: usize
}

impl Region {
    pub fn has_item(&self, map: &Map) -> bool {
        for tile in &self.tiles {
            let pos = map.idx_pos(*tile);
            if map.get_item_ref(pos.x, pos.y).is_some() {
                return true;
            }
        }
        return false;
    }
}

/// A set of doorway tiles that fully separates two structural regions.
pub struct RegionBoundary {
    pub region_a: usize,
    pub region_b: usize,
    /// Tile indices of every Doorway tile on this boundary.
    pub door_tiles: Vec<usize>,
}

/// Output of the spawn analysis pass.
pub struct SpawnMap {
    /// tile index to region index. None for walls, fences and doors.
    pub tile_region: Vec<Option<usize>>,
    /// Admissible spawn points
    pub spawn_points: Vec<SpawnPoint>,
    /// Discrete regions for spawn logic
    pub regions: Vec<Region>,
    /// Boundaries between regions
    pub boundaries: Vec<RegionBoundary>
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Analyse `map` and return classified spawn candidates and connected regions.
pub fn create_spawn_map(map: &Map, start_pos: usize) -> SpawnMap {
    let mut regions = find_regions(map);

    // Tile to region index lookup, built once and shared by classification.
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
            4 if in_room => Some(SpawnCategory::RoomInterior),
            3..=4 => Some(SpawnCategory::Junction),
            _ => None,
        };

        if let Some(category) = category {
            spawn_points.push(SpawnPoint { idx, pos: map.idx_pos(idx), category });
        }
    }

    // Every Doorway tile that touches exactly two distinct regions is a boundary tile.
    let mut boundary_map: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for idx in 0..map.width * map.height {
        if map.tiles[idx] != TileType::Doorway {
            continue;
        }
        let p = map.idx_pos(idx);
        let mut adjacent_regions: Vec<usize> = Vec::new();
        for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
            let nx = p.x + dx;
            let ny = p.y + dy;
            if nx >= 0 && ny >= 0 && nx < map.width as i32 && ny < map.height as i32 {
                let ni = map.xy_idx(nx, ny);
                if let Some(z) = tile_region[ni] {
                    if !adjacent_regions.contains(&z) {
                        adjacent_regions.push(z);
                    }
                }
            }
        }
        if adjacent_regions.len() == 2 {
            let key = (adjacent_regions[0].min(adjacent_regions[1]),
                       adjacent_regions[0].max(adjacent_regions[1]));
            boundary_map.entry(key).or_default().push(idx);
        }
    }

    let boundaries = boundary_map.into_iter()
        .map(|((a, b), door_tiles)| RegionBoundary { region_a: a, region_b: b, door_tiles })
        .collect();

    let start_region = tile_region[start_pos].unwrap_or(0);
    set_region_depth(&mut regions, start_region, &boundaries);
    set_interesting_regions(&mut regions, &boundaries);

    #[cfg(debug_assertions)]
    {
        let mut junctions = 0;
        let mut deadends = 0;
        let mut interiors = 0;
        for p in &spawn_points {
            match p.category {
                SpawnCategory::DeadEnd => deadends += 1,
                SpawnCategory::Junction => junctions += 1,
                SpawnCategory::RoomInterior => interiors += 1
            }
        }
        println!("== Spawn analysis ==");
        println!("   Found {} dead ends, {} junctions, {} room interiors, {} total", deadends, junctions, interiors, spawn_points.len());
        println!("   Found {} regions, smallest is {} tiles, biggest is {} tiles",
            regions.len(),
            regions.iter().min_by(|lhs, rhs| lhs.tiles.len().cmp(&rhs.tiles.len())).unwrap().tiles.len(),
            regions.iter().max_by(|lhs, rhs| lhs.tiles.len().cmp(&rhs.tiles.len())).unwrap().tiles.len());
        println!("   Found {} rooms, of which {} are interesting",
            regions.iter().filter(|r| r.is_room).count(),
            regions.iter().filter(|r| r.is_interesting).count());
        println!("== End spawn analysis ==");
    }

    SpawnMap { tile_region, spawn_points, regions, boundaries }
}

pub fn spawn_loot(world: &mut World, spawn_map: &SpawnMap, rng: &mut RandomNumberGenerator) {
    let starting_pool: &[MakeItem] = &[Item::pistol, Item::flare_gun, Item::knife, Item::grenade];
    let equipment_pool: &[MakeItem] = &[
        Item::pistol, Item::flare_gun, Item::knife,
        Item::revolver, Item::shock_pistol, Item::submachine_gun,
        Item::grenade, Item::fire_grenade, Item::flashbang,
        Item::bolt_action_rifle, Item::semi_auto_rifle,
        Item::assault_rifle, Item::machinegun,
        Item::shock_carbine, Item::flamethrower,
        Item::shock_grenade,
        Item::rotary_machinegun,
        Item::rocket_launcher,
        Item::bulletproof_vest, Item::light_kevlar_pants,
        Item::riot_armor, Item::riot_pants,
        Item::heavy_combat_suit,
        Item::helmet, Item::heavy_helmet,
    ];
    let consumables_pool: &[MakeItem] = &[
        Item::ammo_bullets, Item::ammo_rockets,
        Item::ammo_batteries, Item::ammo_fuel,
        Item::medkit, Item::large_medkit, Item::elixir,
        Item::stimpack,
    ];
    let exceptional_pool: &[MakeItem] = &[
        Item::multi_rocket_launcher,
        Item::shock_cannon,
        Item::rocket_boots,
        Item::tactical_helmet,
        Item::jetpack,
    ];

    /// Higher means fewer items
    const EXCEPTIONAL_ITEM_SPARSITY: usize = 4;
    const EQUIPMENT_ITEM_SPARSITY: usize = 4;
    const CONSUMABLE_ITEM_SPARSITY: usize = 4;

    let boring_rooms: Vec<&Region> = spawn_map.regions.iter().filter(|r| r.is_room && !r.is_interesting).collect();
    let interesting_rooms: Vec<&Region> = spawn_map.regions.iter().filter(|r| r.is_interesting).collect();

    let mut total_items_placed: usize = 0;
    
    // Place starting loot
    {
        let mut items_placed = 0;
        let starting_room = spawn_map.regions.iter().find(|r| r.depth == 0).unwrap_or(&spawn_map.regions[0]);
        let amount = rng.range(1, starting_pool.len());
        for _ in 0 .. amount {
            let target_tile_idx = rng.range(0, starting_room.tiles.len());
            let target_tile = starting_room.tiles[target_tile_idx];
            let target_item_idx = rng.range(1, starting_pool.len());

            if world.add_item(world.map.idx_pos(target_tile), starting_pool[target_item_idx]()).is_ok() {
                items_placed += 1;
            };
        }

        total_items_placed += items_placed;
        println!("Placed {} starting items", items_placed);
    }

    let exceptional_placed = place_items_in_rooms(world, exceptional_pool, &interesting_rooms, EXCEPTIONAL_ITEM_SPARSITY, rng);
    let equipment_placed = place_items_in_rooms(world, equipment_pool, &boring_rooms, EQUIPMENT_ITEM_SPARSITY, rng);
    let consumables_placed = place_items_in_rooms(world, consumables_pool, &boring_rooms, CONSUMABLE_ITEM_SPARSITY, rng);

    total_items_placed += exceptional_placed;
    total_items_placed += equipment_placed;
    total_items_placed += consumables_placed;
    println!("Placed {} exceptional items", exceptional_placed);
    println!("Placed {} equipment items", equipment_placed);
    println!("Placed {} consumable items", consumables_placed);

    println!("Placed {} total items", total_items_placed);
}

pub fn spawn_enemies(world: &mut World, spawn_map: &SpawnMap, rng: &mut RandomNumberGenerator) {
    assert!(world.map.width == world.map.height);
    const ENEMY_SPARSITY: i32 = 200;
    
    let map_radius = world.map.width / 2;
    assert!(map_radius > 15);

    // Radius of circle with no random enemy spawns
    let center_zone_radius: i32 = 15;
    // Radius of inner zone - easy guards
    let inner_zone_radius: i32 = map_radius as i32 / 3 + 5;
    // Radius of middle zone - varied guards
    let middle_zone_radius: i32 = 2 * map_radius as i32 / 3 + 10;
    // Radius of outer zone - difficult guards
    let outer_zone_radius: i32 = map_radius as i32;

    let center = Point { x: (world.map.width / 2) as i32, y: (world.map.height / 2) as i32 };

    let inner_area = inner_zone_radius * inner_zone_radius - center_zone_radius * center_zone_radius;
    let middle_area = middle_zone_radius * middle_zone_radius - inner_zone_radius * inner_zone_radius;
    let outer_area = outer_zone_radius * outer_zone_radius - middle_zone_radius * middle_zone_radius;

    println!("Map radii: center: {} inner: {} middle: {} outer: {}",
        center_zone_radius,
        inner_zone_radius,
        middle_zone_radius,
        outer_zone_radius);
    println!("Map areas: center: {} inner: {} middle: {} outer: {}",
        center_zone_radius*center_zone_radius,
        inner_area,
        middle_area,
        outer_area);

    let mut enemy_count = 0;
    // Naive guard placement for testing
    if false {
        for _ in 0..(inner_area / ENEMY_SPARSITY) {
            let pos = find_enemy_placement(world, spawn_map, center_zone_radius, inner_zone_radius, rng);
            let result = world.create_light_guard(center + pos, Direction::Up);
            match result {
                Ok(_) => enemy_count += 1,
                Err(e) => print!("{}", e.message)
            }
        }
        for _ in 0..(middle_area / ENEMY_SPARSITY) {
            let pos = find_enemy_placement(world, spawn_map, inner_zone_radius, middle_zone_radius, rng);
            let result = world.create_medium_guard(center + pos, Direction::Up);
            match result {
                Ok(_) => enemy_count += 1,
                Err(e) => print!("{}", e.message)
            }
        }
        for _ in 0..(outer_area / ENEMY_SPARSITY) {
            let pos = find_enemy_placement(world, spawn_map, middle_zone_radius, outer_zone_radius, rng);
            let result = world.create_heavy_guard(center + pos, Direction::Up);
            match result {
                Ok(_) => enemy_count += 1,
                Err(e) => print!("{}", e.message)
            }
        }
    }

    // Place guards near interesting doors
    {
        let placement = find_interesting_guard_positions(world, spawn_map, rng);
        for (pos, facing) in placement {
            let distance_to_center = chebyshev(pos, center);
            if distance_to_center < center_zone_radius {
                continue;
            }
            let result = if distance_to_center < inner_zone_radius {
                let random = rng.range(0, 10);
                if random < 5 {
                    world.create_light_guard(pos, facing)
                } else if random < 7 {
                    world.create_riot_guard(pos, facing)
                } else if random < 9 {
                    world.create_medium_guard(pos, facing)
                } else {
                    world.create_flamer_guard(pos, facing)
                }
            }
            else if distance_to_center < middle_zone_radius {
                let random = rng.range(0, 10);
                if random < 5 {
                    world.create_medium_guard(pos, facing)
                } else if random < 7 {
                    world.create_riot_guard(pos, facing)
                } else if random < 9 {
                    world.create_heavy_guard(pos, facing)
                } else {
                    world.create_flamer_guard(pos, facing)
                }
            }
            else {
                let random = rng.range(0, 10);
                if random < 8 {
                    world.create_heavy_guard(pos, facing)
                } else {
                    world.create_rocket_guard(pos, facing)
                }
            };
            
            match result {
                Ok(_) => {
                    let profile = Profile::Guard { anchor: pos, combat_tactic: CombatTactic::Hold };
                    let ai = AI::Actor(ActorAI::new(profile));
                    enemy_count += 1;
                },
                Err(e) => print!("{}", e.message)
            }
        }
    }

    println!("Placed {} enemies", enemy_count);
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Places where guards should stand to guard interesting doorways
fn find_interesting_guard_positions(
    world: &mut World,
    spawn_map: &SpawnMap,
    rng: &mut RandomNumberGenerator
) -> Vec<(Point, Direction)> {
    let mut result: Vec<(Point, Direction)> = Vec::new();

    let interesting_doors: Vec<&RegionBoundary> = spawn_map.boundaries.iter().filter(|b| spawn_map.regions[b.region_a].is_interesting || spawn_map.regions[b.region_b].is_interesting).collect();

    for door in interesting_doors {
        let middle_idx = door.door_tiles[door.door_tiles.len() / 2];
        let candidate_positions = world.map.get_available_exits(middle_idx);
        let position_idx = candidate_positions[rng.range(0, candidate_positions.len())].0; // The first value of the tuple is the tile index
        let pos = world.map.idx_pos(position_idx);
        let door_pos = world.map.idx_pos(middle_idx);
        // Face away from door being guarded
        let mut facing = Direction::Up;
        match Direction::delta_to_dir(pos.x - door_pos.x, pos.y - door_pos.y) {
            Ok(dir) => facing = dir,
            Err(e) => println!("Error when placing guard by door: {}", e.message)
        }
        // If looking into a wall, face door instead
        let (d_pos_x, d_pos_y) = facing.delta_pos();
        if world.map.get_tile(pos.x + d_pos_x, pos.y + d_pos_y) == TileType::Wall {
            match Direction::delta_to_dir(door_pos.x - pos.x, door_pos.y - pos.y) {
                Ok(dir) => facing = dir,
                Err(e) => println!("Error when placing guard by door: {}", e.message)
            }
        }
        result.push((pos, facing))
    }
    result
}

fn find_enemy_placement(
    world: &mut World,
    spawn_map: &SpawnMap,
    inner_radius: i32,
    outer_radius: i32,
    rng: &mut RandomNumberGenerator
) -> Point {
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    assert!(inner_radius < outer_radius);
    while x.abs() < inner_radius && y.abs() < inner_radius {
        x = rng.range(-outer_radius, outer_radius);
        y = rng.range(-outer_radius, outer_radius);
    }
    Point {x, y}
}

/// Returns number of placed items
fn place_items_in_rooms(
    world: &mut World,
    pool: &[MakeItem],
    rooms: &Vec<&Region>,
    item_sparsity: usize,
    rng: &mut RandomNumberGenerator
) -> usize {
    // Higher means rare items can appear at shallower depths
    const RARITY_TOLERANCE: usize = 2;
    // Higher means rare items are rarer compared to common items
    const RARITY_FACTOR: usize = 1;

    let max_depth = rooms.iter().filter(|r| r.depth != usize::MAX).max_by_key(|r| r.depth).unwrap().depth;
    let max_rarity = pool.iter().max_by_key(|i| i().rarity).unwrap()().rarity as usize;
    let min_rarity = pool.iter().min_by_key(|i| i().rarity).unwrap()().rarity as usize;
    // Depth should be higher than rarity, giving a positive integer factor
    let rarity_factor = max_depth / max_rarity;
    let min_depth_factor = rarity_factor - RARITY_TOLERANCE;

    let min_depth = min_depth_factor * min_rarity;

    let item_rarity: Vec<(MakeItem, u8)> =
        pool.iter().map(|&f| (f, f().rarity)).collect();
    let weighted_pool: Vec<(MakeItem, u8)> = item_rarity.iter()
        .flat_map(|&(f, r)| {
            std::iter::repeat((f, r)).take(RARITY_FACTOR * (1 + max_rarity - r as usize))
        })
        .collect();

    let mut items_placed = 0;
    for room in rooms {
        if !(rng.range(0, item_sparsity) == 0) { continue; }

        if room.depth < min_depth { continue; }

        let mut picked_item: Option<MakeItem> = None;
        while picked_item.is_none() {
            let item_idx = rng.range(0, weighted_pool.len());
            let (item, rarity) = weighted_pool[item_idx];
            
            let item_min_depth = rarity as usize * min_depth_factor;
            if room.depth >= item_min_depth {
                picked_item = Some(item);
            }
        }        

        if world.add_item(world.map.idx_pos(room.center_idx), picked_item.unwrap()()).is_ok() {
            items_placed += 1;
        }
    }

    items_placed
}

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
/// Set interesting to false and depth to usize::MAX for later update.
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

        // Calculate center
        let cx: i32 = tiles.iter().map(|&i| map.idx_pos(i).x).sum::<i32>()
            / tiles.len() as i32;
        let cy: i32 = tiles.iter().map(|&i| map.idx_pos(i).y).sum::<i32>()
            / tiles.len() as i32;
        let center_idx = *tiles.iter().min_by_key(|&&i| {
            let p = map.idx_pos(i);
            (p.x - cx).abs() + (p.y - cy).abs()
        }).unwrap();

        // Determine whether this is a room
        let is_room = map.tiles[tiles[0]] == TileType::Floor;

        regions.push(Region { tiles, center_idx, is_room, is_interesting: false, depth: usize::MAX });
    }

    regions
}

fn set_region_depth(regions: &mut Vec<Region>, start_region_idx: usize, region_boundaries: &Vec<RegionBoundary>) {
    let n = regions.len();
    if start_region_idx >= n { return; } // TODO: Error - handle explicitly

    let mut adjacent: Vec<Vec<usize>> = vec![vec![]; n];
    for b in region_boundaries {
        adjacent[b.region_a].push(b.region_b);
        adjacent[b.region_b].push(b.region_a);
    }

    regions[start_region_idx].depth = 0;
    let mut queue = vec![start_region_idx];
    let mut queue_idx = 0;
    while queue_idx < queue.len() {
        let current = queue[queue_idx];
        queue_idx += 1;
        for &neighbour in &adjacent[current] {
            if regions[neighbour].depth == usize::MAX {
                regions[neighbour].depth = regions[current].depth + 1;
                queue.push(neighbour);
            }
        }
    }

    // TODO: Filter out unreachable regions (depth = usize::MAX). But doing so breaks the region boundaries. Fix later.
}

// Note: each door is one boundary. There are no mirrored duplicates in the boundary set.
fn set_interesting_regions(regions: &mut Vec<Region>, region_boundaries: &Vec<RegionBoundary>) {
    const MIN_INTERESTING_DEPTH: usize = 3;

    let mut interesting_indices = vec!();
    for (i, region) in regions.iter().enumerate() {
        if !region.is_room || region.depth < MIN_INTERESTING_DEPTH { continue; }

        let boundaries: Vec<&RegionBoundary> = region_boundaries.iter().filter(|&b| b.region_a == i || b.region_b == i).collect();
        if boundaries.len() == 1 {
            // Only mark rooms in rooms as interesting
            for b in boundaries {
                if regions[b.region_a].is_room && regions[b.region_b].is_room {
                    interesting_indices.push(i);
                }
            }
        }
    }
    
    #[cfg(debug_assertions)]
    println!("Interesting regions found: {}", interesting_indices.len());

    for idx in interesting_indices {
        regions[idx].is_interesting = true;
    }
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

/// BFS over the region graph treating locked doors (whose color is not in `unlocked`) as
/// impassable walls.  Returns a bitmask of which regions the player can reach.
fn reachable_regions(
    adjacent: &[Vec<(usize, Option<usize>)>],
    start_region: usize,
    total_regions: usize,
    unlocked: &std::collections::HashSet<usize>,
) -> Vec<bool> {
    let mut reachable = vec![false; total_regions];
    if start_region >= total_regions { return reachable; }
    reachable[start_region] = true;
    let mut queue = vec![start_region];
    let mut qi = 0;
    while qi < queue.len() {
        let cur = queue[qi]; qi += 1;
        for &(nb, color_opt) in &adjacent[cur] {
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

fn classify_boundary(map: &Map, boundary: &RegionBoundary) -> BoundaryKind {
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
    /// Used for debugging spawn functionality
    #[cfg(debug_assertions)]
    pub(crate) fn spawn_debug(
        &mut self,
        spawn_map: &SpawnMap
    ) {
        println!("== Debugging spawns ==");
        println!("   Spawn map size: regions: {}, boundaries: {}", spawn_map.regions.len(), spawn_map.boundaries.len());
        println!("== Done debugging spawns ==");

        //self.spawn_depthmarkers(spawn_map);
        self.spawn_interesting_markers(spawn_map);
    }

    /// Used for debugging spawn functionality
    #[cfg(debug_assertions)]
    #[allow(unused)]
    fn spawn_depthmarkers(
        &mut self,
        spawn_map: &SpawnMap
    ) {
        for region in &spawn_map.regions {
            let maker: MakeItem = match region.depth {
                0 => Item::knife,
                1 => Item::pistol,
                2 => Item::revolver,
                3 => Item::bolt_action_rifle,
                4 => Item::rotary_machinegun,
                5 => Item::ammo_bullets,
                6 => Item::ammo_rockets,
                7 => Item::ammo_batteries,
                8 => Item::ammo_fuel,
                _ => Item::medkit
            };

            for tile in &region.tiles {
                if tile == &region.center_idx {
                    continue;
                }
                let pos = self.map.idx_pos(*tile);
                let _ = self.add_item(pos, maker());
            }
        }
    }

    /// Debug assignment of interesting status
    #[cfg(debug_assertions)]
    #[allow(unused)]
    fn spawn_interesting_markers(
        &mut self,
        spawn_map: &SpawnMap
    ) {
        for region in &spawn_map.regions {
            if !region.is_interesting { continue; }

            for tile in &region.tiles {
                let pos = self.map.idx_pos(*tile);
                let _ = self.add_item(pos, Item::knife());
            }
        }
    }

    /// Stationary guards adjacent to doorways.
    pub(crate) fn spawn_sentinels(
        &mut self,
        spawn_map: &SpawnMap,
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
                let region_index = spawn_map.tile_region[ni];
                match region_index {
                    Some(idx) => spawn_map.regions[idx].is_interesting,
                    None => false
                }
                
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

    /// First pass: assign lock colors to door entities.
    /// Returns `boundary_colors[bi]` = the color assigned to boundary `bi`,
    /// or `None` if that boundary stays unlocked.
    pub(crate) fn assign_door_colors(
        &mut self,
        spawn_map: &SpawnMap
    ) -> Vec<Option<usize>> {
        const OUTER_WALL_COLOR: usize = 15; // Gold
        const INNER_WALL_COLOR: usize = 13; // Silver

        let mut boundary_colors = vec![None; spawn_map.boundaries.len()];

        let regular_colors: Vec<usize> = (0..crate::components::COLORS.len())
            .filter(|&c| c != OUTER_WALL_COLOR && c != INNER_WALL_COLOR)
            .collect();

        let mut order: Vec<usize> = (0..spawn_map.boundaries.len()).collect();
        order.sort_by_key(|&bi| {
            let b = &spawn_map.boundaries[bi];
            spawn_map.regions[b.region_a].depth.min(spawn_map.regions[b.region_b].depth)
        });

        let mut regular_color_idx = 0usize;
        let mut locked_count = 0usize;

        for &bi in &order {
            let b = &spawn_map.boundaries[bi];
            let kind = classify_boundary(&self.map, b);
            let deep = if spawn_map.regions[b.region_a].depth <= spawn_map.regions[b.region_b].depth { b.region_b } else { b.region_a };

            let color_opt: Option<usize> = match kind {
                BoundaryKind::OuterWall => Some(OUTER_WALL_COLOR),
                BoundaryKind::InnerWall => Some(INNER_WALL_COLOR),
                BoundaryKind::Regular => {
                    if spawn_map.regions[deep].is_interesting {
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
        spawn_map: &SpawnMap,
        boundary_colors: &[Option<usize>],
        start_region: usize,
        rng: &mut RandomNumberGenerator,
    ) {
        let total_regions = spawn_map.regions.len();

        let mut adjacent: Vec<Vec<(usize, Option<usize>)>> = vec![vec![]; total_regions];
        for (b_idx, b) in spawn_map.boundaries.iter().enumerate() {
            let color = boundary_colors[b_idx];
            adjacent[b.region_a].push((b.region_b, color));
            adjacent[b.region_b].push((b.region_a, color));
        }

        let mut color_first_depth: HashMap<usize, usize> = HashMap::new();
        for (b_idx, b) in spawn_map.boundaries.iter().enumerate() {
            if let Some(color) = boundary_colors[b_idx] {
                let d = spawn_map.regions[b.region_a].depth.min(spawn_map.regions[b.region_b].depth);
                color_first_depth.entry(color)
                    .and_modify(|e| *e = (*e).min(d))
                    .or_insert(d);
            }
        }
        let mut color_order: Vec<usize> = color_first_depth.keys().copied().collect();
        color_order.sort_by_key(|&c| color_first_depth[&c]);

        let mut zone_has_key = vec![false; total_regions];
        let mut unlocked: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut total_placed = 0usize;

        for color in color_order {
            let reachable = reachable_regions(&adjacent, start_region, total_regions, &unlocked);

            let mut candidates: Vec<usize> = (0..total_regions)
                .filter(|&zi| reachable[zi] && !zone_has_key[zi])
                .collect();

            for i in (1..candidates.len()).rev() {
                let j = rng.range(0, (i + 1) as i32) as usize;
                candidates.swap(i, j);
            }
            candidates.truncate(KEY_COPIES_PER_COLOR);

            for ri in candidates {
                let key_pos = {
                    let tiles = &spawn_map.regions[ri].tiles;
                    self.map.idx_pos(tiles[rng.range(0, tiles.len() as i32) as usize])
                };
                let _ = self.add_item(key_pos, Item::key(color));
                zone_has_key[ri] = true;
                total_placed += 1;
            }

            unlocked.insert(color);
        }

        println!("Placed {} keys across {} color(s).", total_placed, color_first_depth.len());
    }
}
