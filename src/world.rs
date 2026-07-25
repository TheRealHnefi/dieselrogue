use super::*;
use rltk::{Point, RandomNumberGenerator};
use strum::IntoEnumIterator;
use std::collections::HashMap;
use crate::animation::explosion_animation;

pub struct ActiveItem {
    pub item_id: usize,
    pub location: ItemLocation,
}

/// The contents of the game world itself.
pub struct World {
    pub player_id: Option<usize>,
    pub entities: Vec<Entity>,
    pub map: Map,
    pub pending_levelup: bool,
    pub sounds: Vec<SoundEvent>,
    pub sounds_last_turn: Vec<SoundEvent>,
    pub active_items: Vec<ActiveItem>,
    active_items_ticked: bool,
    next_item_id: usize,
    pub debug_mode: bool,
    pub parallel_ai: bool,
}

/// Overall guard density multiplier. Raise or lower to scale all guard counts.
const GUARD_DENSITY: f32 = 0.5;

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

const KEY_COPIES_PER_COLOR: usize = 3;

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
    /// Create new world.
    /// # Arguments
    /// * `size` - Number of blocks that make up one size of the map.
    pub fn new(size: usize, seed: u64) -> Self {
        let mut rng = RandomNumberGenerator::seeded(seed);
        let mut world = World {
            player_id: Option::None,
            entities: vec![],
            next_item_id: 0,
            pending_levelup: false,
            sounds: vec![],
            sounds_last_turn: vec![],
            active_items: vec![],
            active_items_ticked: false,
            map: Map::new_game_map(size, &mut rng),
            debug_mode: false,
            parallel_ai: false,
        };

        let pos = Point {x: (world.map.width / 2) as i32, y: (world.map.height / 2) as i32};
        let _result = world.create_player(pos,
            Direction::Up,
            String::from("Player"));

        world.init_static_entities();

        // Topology analysis — run once, shared by all placement passes.
        let spawn_map = analyze(&world.map);
        let zone_map  = find_zones(&world.map);

        // Zone depths from the player's starting tile, used for placement weighting.
        let player_tile = world.get_player()
            .map(|p| world.map.pos_idx(p.position))
            .unwrap_or_else(|_| world.map.xy_idx(
                (world.map.width / 2) as i32, (world.map.height / 2) as i32));
        let start_zone = zone_map.tile_zone[player_tile].unwrap_or(0);
        let depths = zone_depths(&zone_map, start_zone);

        let interesting = mark_interesting_zones(&zone_map, &spawn_map, &mut rng);
        let boundary_colors = world.assign_door_colors(&zone_map, &depths, &interesting);
        world.place_zone_keys(&zone_map, &spawn_map, &depths, &boundary_colors, start_zone, &mut rng);
        world.spawn_loot(&zone_map, &spawn_map, &depths, &mut rng);

        let mut placed: Vec<Point> = Vec::new();
        let mut guard_n = 0usize;
        println!("Spawning guards:");
        world.spawn_sentinels(&mut placed, &mut guard_n, &mut rng);
        // world.spawn_patrollers(&spawn_map, &mut placed, &mut guard_n, &mut rng);
        // world.spawn_squads(&spawn_map, &mut placed, &mut guard_n, &mut rng);
        // world.spawn_idle_guards(&spawn_map, &mut placed, &mut guard_n, &mut rng);
        println!("Spawned {} guards total.", guard_n);

        // Spawn tanks on roads and in hangars, skewing toward outer zones.
        let tank_spawns = find_tank_spawns(&world.map, &spawn_map.regions);

        // Max tanks to place per zone depth (index = depth, value = cap).
        // Depth 0 = player's start zone (inner) → no tanks.
        const MAX_TANKS_BY_DEPTH: &[usize] = &[0, 2, 4, 6];
        const MIN_TANK_DIST: i32 = 15;

        let tank_dirs = [Direction::Up, Direction::Right, Direction::Down, Direction::Left];
        let mut tank_placed: Vec<Point> = Vec::new();

        for depth in 1..MAX_TANKS_BY_DEPTH.len() {
            let cap = MAX_TANKS_BY_DEPTH[depth];

            let mut candidates: Vec<usize> = tank_spawns.road_tiles.iter()
                .chain(tank_spawns.hangar_tiles.iter())
                .copied()
                .filter(|&idx| zone_map.tile_zone[idx].map(|z| depths[z]) == Some(depth))
                .collect();

            for i in (1..candidates.len()).rev() {
                let j = rng.range(0, (i + 1) as i32) as usize;
                candidates.swap(i, j);
            }

            let mut placed_here = 0usize;
            for idx in candidates {
                if placed_here >= cap { break; }
                let pos = world.map.idx_pos(idx);
                let too_close = tank_placed.iter().any(|&p| {
                    (p.x - pos.x).abs().max((p.y - pos.y).abs()) < MIN_TANK_DIST
                });
                if too_close { continue; }
                let facing = tank_dirs[tank_placed.len() % tank_dirs.len()];
                if world.create_tank(pos, facing, format!("Tank {}", tank_placed.len() + 1)).is_ok() {
                    tank_placed.push(pos);
                    placed_here += 1;
                }
            }
        }

        println!("Spawned {} tanks.", tank_placed.len());

        return world;
    }

    /// Create new world for testing purposes.
    pub fn new_test() -> Self {
        Self {
            player_id: Option::None,
            entities: vec![],
            next_item_id: 0,
            pending_levelup: false,
            sounds: vec![],
            sounds_last_turn: vec![],
            active_items: vec![],
            active_items_ticked: false,
            map: Map::new_empty_map(100, 100),
            debug_mode: false,
            parallel_ai: false,
        }
    }

    pub fn create_player(&mut self, pos: Point, facing: Direction, name: String) -> Result<(), GameError> {
        if self.entities.len() > 0 {
            return Err(GameError {
                error: Error::BadPrecondition,
                message: String::from("Tried to create player, but entities already exist")
            });
        }

        let nearest_pos = self.map.nearest_free_pawn_position(pos)?;

        let mut player = Entity::new_human(0, nearest_pos, facing, name);
        player.kind = EntityKind::Player;
        player.paper_doll = Some(PaperDoll::Player);
        player.body.update_abilities();
        player.color = Some(5);

        player.create_pawns(&mut self.map);
        self.entities.push(player);
        self.player_id = Some(0);
        self.entities[0].update_view(&mut self.map);

        Ok(())
    }

    pub fn create_zombie_goon(&mut self, pos: Point, facing: Direction, name: String) -> Result<(), GameError> {
        let actual_pos = self.map.nearest_free_pawn_position(pos)?;
        let mut entity = Entity::new_human(self.entities.len(), actual_pos, facing, name);
        entity.ai = AI::Rotator;
        entity.paper_doll = Some(PaperDoll::MaleSilhouette);
        self.equip_pistol(&mut entity);
        entity.create_pawns(&mut self.map);
        self.entities.push(entity);

        Ok(())
    }

    pub fn create_forward_goon(&mut self, pos: Point, facing: Direction, name: String) -> Result<(), GameError> {
        let actual_pos = self.map.nearest_free_pawn_position(pos)?;
        let mut entity = Entity::new_human(self.entities.len(), actual_pos, facing, name);
        entity.ai = AI::Forward;
        entity.paper_doll = Some(PaperDoll::MaleSilhouette);
        self.equip_pistol(&mut entity);
        entity.create_pawns(&mut self.map);
        self.entities.push(entity);

        Ok(())
    }

    pub fn create_patrolling_goon(&mut self, pos: Point, facing: Direction, name: String, waypoints: Vec<Point>) -> Result<(), GameError> {
        let actual_pos = self.map.nearest_free_pawn_position(pos)?;
        let mut entity = Entity::new_patrolling_goon(self.entities.len(), actual_pos, facing, name, waypoints);
        self.equip_pistol(&mut entity);
        entity.create_pawns(&mut self.map);
        self.entities.push(entity);
        Ok(())
    }

    /// Creates an NPC with the full profile+alert AI system.
    pub fn create_actor(&mut self, pos: Point, facing: Direction, name: String, profile: Profile) -> Result<(), GameError> {
        let actual_pos = self.map.nearest_free_pawn_position(pos)?;
        let mut entity = Entity::new_human(self.entities.len(), actual_pos, facing, name);
        entity.ai = AI::Actor(ActorAI::new(profile));
        entity.paper_doll = Some(PaperDoll::MaleSilhouette);
        self.equip_pistol(&mut entity);
        entity.create_pawns(&mut self.map);
        self.entities.push(entity);
        Ok(())
    }

    pub fn create_patrol_actor(&mut self, pos: Point, facing: Direction, name: String, waypoints: Vec<Point>, tactic: CombatTactic) -> Result<(), GameError> {
        self.create_actor(pos, facing, name, Profile::Patrol {
            waypoints,
            waypoint_index: 0,
            combat_tactic: tactic,
        })
    }

    pub fn create_guard_actor(&mut self, pos: Point, facing: Direction, name: String, tactic: CombatTactic) -> Result<(), GameError> {
        let anchor = self.map.nearest_free_pawn_position(pos)?;
        self.create_actor(pos, facing, name, Profile::Guard { anchor, combat_tactic: tactic })
    }

    pub fn create_stationary_actor(&mut self, pos: Point, facing: Direction, name: String, tactic: CombatTactic) -> Result<(), GameError> {
        self.create_actor(pos, facing, name, Profile::Stationary { combat_tactic: tactic })
    }

    // --- Guard placement passes -------------------------------------------

    /// Stationary guards adjacent to doorways
    fn spawn_sentinels(
        &mut self,
        placed: &mut Vec<Point>,
        n: &mut usize,
        rng: &mut RandomNumberGenerator,
    ) {
        const RATE: f32 = 0.30;
        const MIN_DIST: i32 = 5;

        let mut candidates: Vec<(Point, Point)> = Vec::new();
        for idx in 0..self.map.width * self.map.height {
            if self.map.tiles[idx] != TileType::Doorway { continue; }
            let door_pos = self.map.idx_pos(idx);
            // Place guard 3 tiles away from door
            for &(dx, dy) in &[(0i32, -3i32), (3, 0), (0, 3), (-3, 0)] {
                let nx = door_pos.x + dx;
                let ny = door_pos.y + dy;
                if nx < 0 || ny < 0 || nx >= self.map.width as i32 || ny >= self.map.height as i32 {
                    continue;
                }
                let nidx = self.map.xy_idx(nx, ny);
                if is_spawnable(self.map.tiles[nidx]) {
                    candidates.push((Point::new(nx, ny), door_pos));
                    break;
                }
            }
        }

        fy_shuffle(&mut candidates, rng);
        let target = ((candidates.len() as f32) * RATE * GUARD_DENSITY) as usize;
        let mut count = 0;
        for (guard_pos, door_pos) in candidates {
            if count >= target { break; }
            if guard_too_close(guard_pos, placed, MIN_DIST) { continue; }
            // Face away from the door
            let facing = dir_toward(door_pos, guard_pos);
            *n += 1;
            if self.create_guard_actor(guard_pos, facing, format!("Sentinel {}", n), CombatTactic::Hold).is_ok() {
                placed.push(guard_pos);
                count += 1;
            }
        }
        println!("  Sentinels: {}", count);
    }

    /// Patrol guards following pathfinder-computed road routes.
    fn spawn_patrollers(
        &mut self,
        spawn_map: &SpawnMap,
        placed: &mut Vec<Point>,
        n: &mut usize,
        rng: &mut RandomNumberGenerator,
    ) {
        const RATE: f32 = 0.15;
        const MIN_DIST: i32 = 15;
        const MIN_PATROL_DIST: i32 = 20;
        const MAX_PATROL_DIST: i32 = 80;
        const WAYPOINT_STEP: usize = 8;

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
            let (a_pos, a_idx) = {
                let sp = &spawn_map.spawn_points[ai];
                if guard_too_close(sp.pos, placed, MIN_DIST) { continue; }
                (sp.pos, sp.idx)
            };

            let bi = order.iter()
                .map(|&oi2| junctions[oi2])
                .find(|&bi| {
                    if bi == ai || used.contains(&bi) { return false; }
                    let d = chebyshev(spawn_map.spawn_points[bi].pos, a_pos);
                    d >= MIN_PATROL_DIST && d <= MAX_PATROL_DIST
                });
            let bi = match bi { Some(b) => b, None => continue };
            let b_pos = spawn_map.spawn_points[bi].pos;
            let b_idx = spawn_map.spawn_points[bi].idx;

            let path = navigate(a_idx, b_idx, &self.map);
            if !path.success || path.steps.is_empty() { continue; }

            let mut waypoints = vec![a_pos];
            for step_i in (WAYPOINT_STEP..path.steps.len()).step_by(WAYPOINT_STEP) {
                waypoints.push(self.map.idx_pos(path.steps[step_i]));
            }
            waypoints.push(b_pos);

            let facing = dir_toward(a_pos, b_pos);
            *n += 1;
            if self.create_patrol_actor(a_pos, facing, format!("Patroller {}", n), waypoints, CombatTactic::Pursue).is_ok() {
                placed.push(a_pos);
                used.push(ai);
                used.push(bi);
                count += 1;
            }
        }
        println!("  Patrollers: {}", count);
    }

    /// Small squads — a Guard leader with Follow-profile members nearby.
    fn spawn_squads(
        &mut self,
        spawn_map: &SpawnMap,
        placed: &mut Vec<Point>,
        n: &mut usize,
        rng: &mut RandomNumberGenerator,
    ) {
        const RATE: f32 = 0.10;
        const MIN_DIST: i32 = 12;
        const SQUAD_SIZE: usize = 2;
        const SQUAD_RADIUS: i32 = 5;

        let room_pts: Vec<usize> = spawn_map.spawn_points.iter()
            .enumerate()
            .filter(|(_, sp)| matches!(sp.category, SpawnCategory::RoomInterior))
            .map(|(i, _)| i)
            .collect();

        let mut order: Vec<usize> = (0..room_pts.len()).collect();
        fy_shuffle(&mut order, rng);

        let target = ((room_pts.len() as f32) * RATE * GUARD_DENSITY) as usize;
        let mut count = 0;

        for &oi in &order {
            if count >= target { break; }
            let li = room_pts[oi];
            let (leader_pos, ok) = {
                let sp = &spawn_map.spawn_points[li];
                (sp.pos, !guard_too_close(sp.pos, placed, MIN_DIST))
            };
            if !ok { continue; }

            let followers: Vec<Point> = room_pts.iter()
                .filter(|&&fi| fi != li)
                .filter(|&&fi| {
                    let sp = &spawn_map.spawn_points[fi];
                    let d = chebyshev(sp.pos, leader_pos);
                    d > 0 && d <= SQUAD_RADIUS && !guard_too_close(sp.pos, placed, 2)
                })
                .map(|&fi| spawn_map.spawn_points[fi].pos)
                .take(SQUAD_SIZE)
                .collect();

            if followers.len() < SQUAD_SIZE { continue; }

            *n += 1;
            let leader_n = *n;
            if self.create_guard_actor(leader_pos, Direction::Down, format!("Squad Leader {}", leader_n), CombatTactic::Pursue).is_ok() {
                placed.push(leader_pos);
                let leader_id = self.entities.len() - 1;
                for (fi, &fp) in followers.iter().enumerate() {
                    let facing = dir_toward(fp, leader_pos);
                    *n += 1;
                    let _ = self.create_actor(
                        fp, facing,
                        format!("Squad Member {} ({})", fi + 1, leader_n),
                        Profile::Follow {
                            target_id: leader_id,
                            last_known_pos: leader_pos,
                            combat_tactic: CombatTactic::Pursue,
                        },
                    );
                    placed.push(fp);
                }
                count += 1;
            }
        }
        println!("  Squads: {} ({} guards each)", count, SQUAD_SIZE + 1);
    }

    /// Stationary guards scattered through room interiors.
    fn spawn_idle_guards(
        &mut self,
        spawn_map: &SpawnMap,
        placed: &mut Vec<Point>,
        n: &mut usize,
        rng: &mut RandomNumberGenerator,
    ) {
        const RATE: f32 = 0.015;
        const MIN_DIST: i32 = 8;

        let room_pts: Vec<usize> = spawn_map.spawn_points.iter()
            .enumerate()
            .filter(|(_, sp)| matches!(sp.category, SpawnCategory::RoomInterior))
            .map(|(i, _)| i)
            .collect();

        let mut order: Vec<usize> = (0..room_pts.len()).collect();
        fy_shuffle(&mut order, rng);

        let target = ((room_pts.len() as f32) * RATE * GUARD_DENSITY) as usize;
        let dirs = [Direction::Up, Direction::Right, Direction::Down, Direction::Left];
        let mut count = 0;

        for &oi in &order {
            if count >= target { break; }
            let (pos, ok) = {
                let sp = &spawn_map.spawn_points[room_pts[oi]];
                (sp.pos, !guard_too_close(sp.pos, placed, MIN_DIST))
            };
            if !ok { continue; }
            let facing = dirs[count % dirs.len()];
            *n += 1;
            if self.create_stationary_actor(pos, facing, format!("Guard {}", n), CombatTactic::Hold).is_ok() {
                placed.push(pos);
                count += 1;
            }
        }
        println!("  Idle guards: {}", count);
    }

    fn equip_pistol(&mut self, entity: &mut Entity) {
        let mut pistol = Item::pistol();
        pistol.id = self.next_item_id;
        self.next_item_id += 1;
        let _ = entity.body.equip(pistol);
        entity.body.update_armor();
    }

    pub fn create_tank(&mut self, pos: Point, facing: Direction, name: String) -> Result<(), GameError> {
        let pos = self.map.nearest_free_pawn_position_sized(pos, 3, 3)?;

        let mut tank = Entity::new_tank(self.entities.len(), pos, facing, name);
        tank.paper_doll = Some(PaperDoll::Tank);
        tank.create_pawns(&mut self.map);
        self.entities.push(tank);

        Ok(())
    }

    pub fn get_player(&self) -> Result<&Entity, GameError> {
        match self.player_id {
            Some(id) => return Ok(&self.entities[id]),
            None => return Err(GameError {
                error: Error::BadPrecondition,
                message: format!("No player exists")
            })
        }
    }

    /// Resolves the player's current aim target position.
    /// For `AimingAtGround` returns the stored point; for `AimingAtEntity` looks up
    /// the entity's current center so the aim tracks movement.
    pub fn get_player_aim_position(&self) -> Option<Point> {
        let player = self.get_player().ok()?;
        let key = StatusEffect::AimingAtGround(Point { x: 0, y: 0 }, Item::pistol());
        match player.body.get_status_effect(&key) {
            Some(StatusEffect::AimingAtGround(pos, _))     => Some(*pos),
            Some(StatusEffect::AimingAtEntity(entity_id, _)) =>
                self.entities.get(*entity_id).map(|e| e.center()),
            _ => None,
        }
    }

    pub fn get_player_mut(&mut self) -> Result<&mut Entity, GameError> {
        match self.player_id {
            Some(id) => return Ok(&mut self.entities[id]),
            None => return Err(GameError {
                error: Error::BadPrecondition,
                message: format!("No player exists")
            })
        }
    }

    pub fn compute_levelup_options(&self) -> Vec<Ability> {
        match self.get_player() {
            Ok(player) => Ability::iter()
                .filter(|a| !a.is_innate() && !player.has_ability(a.clone()))
                .collect(),
            Err(_) => vec![],
        }
    }

    pub fn add_item(&mut self, pos: Point, mut item: Item)  -> Result<(), GameError> {
        let actual_pos = self.map.nearest_free_item_position(pos)?;

        let index = self.map.pos_idx(actual_pos);
        item.id = self.next_item_id;
        self.next_item_id += 1;
        self.map.items[index] = Some(item);
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub fn resolve_intent_declaration(&mut self) {
        // Step 1: Extract all AI states so we can hold &self.entities (immutable)
        // while mutating AI state (path cache etc.) during computation.
        let mut ai_states: Vec<AI> = self.entities.iter_mut()
            .map(|e| std::mem::replace(&mut e.ai, AI::None))
            .collect();

        // Step 2: Compute intents and collect any sounds emitted by AIs (e.g. alert shouts).
        // Each closure only reads map, entities, and sounds — safe to run on a thread pool.
        let map = &self.map;
        let entities = &self.entities;
        let sounds = &self.sounds_last_turn[..];

        let compute = |(ai, entity): (&mut AI, &Entity)| -> (Option<Intent>, Vec<SoundEvent>) {
            match entity.driving {
                DrivingState::Driving(_)  => (None, vec![]),
                DrivingState::DrivenBy(_) => (None, vec![]),
                _ => ai.compute_intent(entity, map, entities, sounds),
            }
        };

        let results: Vec<(Option<Intent>, Vec<SoundEvent>)> = if self.parallel_ai {
            use rayon::prelude::*;
            ai_states.par_iter_mut()
                .zip(entities.par_iter())
                .map(compute)
                .collect()
        } else {
            ai_states.iter_mut()
                .zip(entities.iter())
                .map(compute)
                .collect()
        };

        // Separate intents and emitted sounds; extend sounds after the map/entities borrows end.
        let mut ai_sounds: Vec<SoundEvent> = vec![];
        let mut intents: Vec<Option<Intent>> = results.into_iter()
            .map(|(intent, emitted)| { ai_sounds.extend(emitted); intent })
            .collect();

        // Step 3: Resolve vehicle-pilot pairs sequentially.
        for i in 0..entities.len() {
            if let DrivingState::DrivenBy(pilot_id) = entities[i].driving {
                let (intent, emitted) = ai_states[pilot_id].compute_intent(
                    &entities[i], map, entities, sounds,
                );
                intents[i] = intent;
                ai_sounds.extend(emitted);
            }
        }
        // map/entities borrows end here; now safe to borrow self.sounds.
        self.sounds.extend(ai_sounds);

        // Step 4: Restore AI states and apply computed intents.
        for ((entity, ai), maybe_intent) in self.entities.iter_mut()
            .zip(ai_states)
            .zip(intents)
        {
            entity.ai = ai;
            if let Some(intent) = maybe_intent {
                entity.intent = intent;
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn resolve_phase(&mut self, phase: ExecutionPhase, log: &mut GameLog) -> Vec<Animation> {
        if phase == ExecutionPhase::Movement {
            self.cancel_contested_moves();
        }

        if phase == ExecutionPhase::ActiveItems {
            return self.resolve_active_items(log);
        }

        // Collect effects from all entities whose intent fires this phase.
        // Action functions are pure (&Entity, &Map, &[Entity]) so this is safe
        // to run in parallel — no shared mutable state.
        let effects: Vec<Effect> = if self.parallel_ai {
            use rayon::prelude::*;
            self.entities.par_iter()
                .filter(|e| e.intent.phase == phase
                         && e.body.get_status_effect(&StatusEffect::Shocked(0)).is_none())
                .flat_map(|e| (e.intent.action)(e, &self.map, &self.entities))
                .collect()
        } else {
            self.entities.iter()
                .filter(|e| e.intent.phase == phase
                         && e.body.get_status_effect(&StatusEffect::Shocked(0)).is_none())
                .flat_map(|e| (e.intent.action)(e, &self.map, &self.entities))
                .collect()
        };

        // Reset intents for entities that fired this phase.
        for entity in self.entities.iter_mut() {
            if entity.intent.phase == phase {
                entity.intent = idle_intent();
            }
        }

        return self.resolve_effects(&effects, log);
    }

    fn resolve_active_items(&mut self, log: &mut GameLog) -> Vec<Animation> {
        if self.active_items_ticked {
            return vec![];
        }
        self.active_items_ticked = true;

        struct Tick {
            item_id: usize,
            location: ItemLocation,
            damage: Damage,
            timeout: u32,
            radius: u32,
            flash: bool,
        }

        let mut ticks: Vec<Tick> = vec!();
        for active in &self.active_items {
            let found = match &active.location {
                ItemLocation::OnMap(pos) => {
                    let idx = self.map.pos_idx(*pos);
                    self.map.items[idx].as_ref()
                        .filter(|i| i.id == active.item_id)
                        .and_then(|i| if let ItemKind::FusedExplosive { damage, timeout, radius, flash } = i.kind {
                            Some((damage, timeout, radius, flash))
                        } else { None })
                        .map(|(d, t, r, f)| (active.location.clone(), d, t, r, f))
                },
                ItemLocation::InInventory(eid) => {
                    self.entities.get(*eid)
                        .and_then(|e| e.body.inventory.iter().find(|i| i.id == active.item_id))
                        .and_then(|i| if let ItemKind::FusedExplosive { damage, timeout, radius, flash } = i.kind {
                            Some((damage, timeout, radius, flash))
                        } else { None })
                        .map(|(d, t, r, f)| (active.location.clone(), d, t, r, f))
                },
            };
            if let Some((location, damage, timeout, radius, flash)) = found {
                ticks.push(Tick { item_id: active.item_id, location, damage, timeout, radius, flash });
            }
        }

        let mut effects: Vec<Effect> = vec!();
        let mut exploded: Vec<usize> = vec!();

        for tick in &ticks {
            let new_timeout = tick.timeout - 1;
            if new_timeout == 0 {
                let pos = match &tick.location {
                    ItemLocation::OnMap(p) => *p,
                    ItemLocation::InInventory(eid) => self.entities[*eid].position,
                };
                effects.extend(self.detonate_explosive(pos, tick.damage, tick.radius, tick.flash, log));
                self.remove_item_from_location(tick.item_id, &tick.location);
                exploded.push(tick.item_id);
            } else {
                self.set_fuse_timeout(tick.item_id, &tick.location, new_timeout);
            }
        }

        self.active_items.retain(|a| !exploded.contains(&a.item_id));

        self.resolve_effects(&effects, log)
    }

    fn detonate_explosive(&self, pos: Point, damage: Damage, radius: u32, flash: bool, log: &mut GameLog) -> Vec<Effect> {
        let mut effects: Vec<Effect> = vec![];
        let r = radius as i32;
        if flash {
            log.log(String::from("A flashbang goes off!"));
            for entity in &self.entities {
                let dx = entity.position.x - pos.x;
                let dy = entity.position.y - pos.y;
                if dx * dx + dy * dy <= r * r && entity.can_see(pos) {
                    effects.push(Effect::ApplyStatus { target_id: entity.id, status: StatusEffect::Blind(5) });
                }
            }
            effects.push(Effect::Animation(flashbang_animation(pos, radius)));
        } else {
            log.log(String::from("A grenade explodes!"));
            for entity in &self.entities {
                let dx = entity.position.x - pos.x;
                let dy = entity.position.y - pos.y;
                if dx * dx + dy * dy <= r * r {
                    for part in 0..entity.body.parts.len() {
                        effects.push(Effect::Damage { entity_id: entity.id, bodypart_index: part, raw_damage: damage });
                    }
                }
            }
            effects.push(Effect::Animation(explosion_animation(pos, radius)));
        }
        effects.push(Effect::Sound(SoundEvent { kind: SoundKind::Explosion, pos, volume: 25 }));
        effects
    }

    fn set_fuse_timeout(&mut self, item_id: usize, location: &ItemLocation, timeout: u32) {
        match location {
            ItemLocation::OnMap(pos) => {
                let idx = self.map.pos_idx(*pos);
                if let Some(item) = &mut self.map.items[idx] {
                    if item.id == item_id {
                        if let ItemKind::FusedExplosive { timeout: ref mut t, .. } = item.kind {
                            *t = timeout;
                        }
                    }
                }
            },
            ItemLocation::InInventory(eid) => {
                if let Some(entity) = self.entities.get_mut(*eid) {
                    if let Some(item) = entity.body.inventory.iter_mut().find(|i| i.id == item_id) {
                        if let ItemKind::FusedExplosive { timeout: ref mut t, .. } = item.kind {
                            *t = timeout;
                        }
                    }
                }
            },
        }
    }

    fn remove_item_from_location(&mut self, item_id: usize, location: &ItemLocation) {
        match location {
            ItemLocation::OnMap(pos) => {
                let idx = self.map.pos_idx(*pos);
                if self.map.items[idx].as_ref().map_or(false, |i| i.id == item_id) {
                    self.map.items[idx] = None;
                }
            },
            ItemLocation::InInventory(eid) => {
                if let Some(entity) = self.entities.get_mut(*eid) {
                    entity.body.inventory.retain(|i| i.id != item_id);
                }
            },
        }
    }

    fn cancel_contested_moves(&mut self) {
        // Count how many entities intend to enter each tile this turn.
        let mut target_counts: HashMap<(i32, i32), usize> = HashMap::new();
        for entity in &self.entities {
            if entity.intent.phase == ExecutionPhase::Movement
                && std::ptr::fn_addr_eq(entity.intent.action, move_action as Action)
            {
                if let IntentData::Target(pos) = entity.intent.data {
                    *target_counts.entry((pos.x, pos.y)).or_insert(0) += 1;
                }
            }
        }

        // Cancel every move intent whose destination is contested.
        for entity in &mut self.entities {
            if entity.intent.phase == ExecutionPhase::Movement
                && std::ptr::fn_addr_eq(entity.intent.action, move_action as Action)
            {
                if let IntentData::Target(pos) = entity.intent.data {
                    if target_counts[&(pos.x, pos.y)] > 1 {
                        entity.intent = idle_intent();
                    }
                }
            }
        }
    }

    #[tracing::instrument(skip_all)]
    fn resolve_effects(&mut self, effects: &Vec<Effect>, log: &mut GameLog) -> Vec<Animation> {
        let mut animations = vec!();
        let mut deathlist: Vec<usize> = vec!();
        for effect in effects.iter() {
            match effect {
                Effect::Damage{entity_id: id, bodypart_index: part_index, raw_damage: damage} => {
                    let elec_penetrates = self.entities[*id].body.parts[*part_index].armor.electrical_penetrates(*damage);
                    self.handle_damage(*id, *part_index, *damage, &mut deathlist, log);
                    if damage.fire > 0 {
                        self.entities[*id].apply_status_effect(&StatusEffect::Burning(5));
                    }
                    if elec_penetrates {
                        self.entities[*id].apply_status_effect(&StatusEffect::Shocked(1));
                    }
                }
                Effect::OpenDoor {pos, actor_id} =>
                    self.handle_open_door(*pos, *actor_id, log),
                Effect::DestroyWall(pos) =>
                    self.handle_destroy_wall(*pos),
                Effect::Embark{pilot_id, vehicle_id} =>
                    self.handle_embark(*pilot_id, *vehicle_id, log),
                Effect::Disembark{pilot_id, vehicle_id} =>
                    self.handle_disembark(*pilot_id, *vehicle_id, log),
                Effect::Animation(animation) =>
                    animations.push(animation.clone()),
                Effect::ApplyStatus{target_id, status} =>
                    self.entities[*target_id].apply_status_effect(status),
                Effect::BurnTick{entity_id: id, bodypart_index: part_index} =>
                    self.handle_damage(*id, *part_index, Damage::new(0, 0, 1, 0), &mut deathlist, log),
                Effect::SyncActiveItem{item_id, location} =>
                    self.sync_active_item(*item_id, location.clone()),
                Effect::Sound(sound) =>
                    self.sounds.push(sound.clone()),
                Effect::Twist{entity_id, direction} => {
                    self.entities[*entity_id].intent = Intent {
                        phase: ExecutionPhase::Movement,
                        data: IntentData::Direction(*direction),
                        action: actions::turn_action,
                    };
                },
                Effect::Distract{entity_id} => {
                    let entity = &mut self.entities[*entity_id];
                    entity.clear_aiming();
                    entity.intent = idle_intent();
                },
                Effect::Log(msg) => log.log(msg.clone()),
                Effect::Move { entity_id, pos } => {
                    self.entities[*entity_id].set_position(*pos, &mut self.map);
                    self.entities[*entity_id].clear_aiming();
                },
                Effect::SetFacing { entity_id, direction } => {
                    self.entities[*entity_id].body.facing = *direction;
                    let pos = self.entities[*entity_id].position;
                    self.entities[*entity_id].set_position(pos, &mut self.map);
                    self.entities[*entity_id].clear_aiming();
                },
                Effect::ConsumeAmmo { entity_id, slot, shots } => {
                    if let Some(item) = self.entities[*entity_id].get_equipped_item(*slot) {
                        if let ItemKind::Firearm { ref mut ammo, .. } = item.kind {
                            *ammo = ammo.saturating_sub(*shots);
                        }
                    }
                },
                Effect::SpendEnergy { entity_id, amount } => {
                    self.entities[*entity_id].body.energy =
                        self.entities[*entity_id].body.energy.saturating_sub(*amount);
                },
                Effect::PickUpItem { entity_id } => {
                    let pos = self.entities[*entity_id].position;
                    let idx = self.map.xy_idx(pos.x, pos.y);
                    if let Some(item) = self.map.items[idx].take() {
                        log.log(format!("{} picked up {}", self.entities[*entity_id].name, item.name));
                        if item.active {
                            self.sync_active_item(item.id, ItemLocation::InInventory(*entity_id));
                        }
                        self.entities[*entity_id].body.inventory.push(item);
                    }
                },
                Effect::DropItem { entity_id, item_id } => {
                    let pos = self.entities[*entity_id].position;
                    if let Some(item) = self.entities[*entity_id].take_item_by_id(*item_id) {
                        if let Ok(drop_pos) = self.map.nearest_free_item_position(pos) {
                            if item.active {
                                self.sync_active_item(item.id, ItemLocation::OnMap(drop_pos));
                            }
                            let map_idx = self.map.pos_idx(drop_pos);
                            self.map.items[map_idx] = Some(item);
                        }
                    }
                },
                Effect::ThrowItem { entity_id, item_id, target_pos } => {
                    if let Some(item) = self.entities[*entity_id].take_item_by_id(*item_id) {
                        if let Ok(drop_pos) = self.map.nearest_free_item_position(*target_pos) {
                            if item.active {
                                self.sync_active_item(item.id, ItemLocation::OnMap(drop_pos));
                            }
                            let map_idx = self.map.pos_idx(drop_pos);
                            self.map.items[map_idx] = Some(item);
                        }
                    }
                },
                Effect::PrimeItem { entity_id, item_id } => {
                    if let Some(item) = self.entities[*entity_id].body.inventory.iter_mut()
                        .find(|i| i.id == *item_id)
                    {
                        item.active = true;
                        item.inventory_actions.retain(|a| a.name != "Prime");
                        self.sync_active_item(*item_id, ItemLocation::InInventory(*entity_id));
                    }
                },
                Effect::EquipItem { entity_id, item_id } => {
                    if let Some(item) = self.entities[*entity_id].take_item_by_id(*item_id) {
                        let item_name = item.name.clone();
                        match self.entities[*entity_id].body.equip(item.clone()) {
                            Ok(displaced) => {
                                log.log(format!("{} equipped {}", self.entities[*entity_id].name, item_name));
                                for d in displaced {
                                    log.log(format!("{} unequipped {}", self.entities[*entity_id].name, d.name));
                                    self.entities[*entity_id].body.inventory.push(d);
                                }
                            },
                            Err(_) => self.entities[*entity_id].body.inventory.push(item),
                        }
                        self.entities[*entity_id].body.update_armor();
                    }
                },
                Effect::UnequipItem { entity_id, item_id } => {
                    // Find which slot holds this item, then unequip.
                    let slot = self.entities[*entity_id].body.item_slots.iter()
                        .find(|s| s.item.as_ref().map_or(false, |i| i.id == *item_id))
                        .map(|s| s.slot_type);
                    if let Some(slot) = slot {
                        if let Some(item) = self.entities[*entity_id].body.unequip(SlotType::from(slot)) {
                            let was_aiming_at = self.entities[*entity_id].body.status_effects.iter()
                                .find_map(|s| match s {
                                    StatusEffect::AimingAtGround(_, i) => Some(i.id),
                                    StatusEffect::AimingAtEntity(_, i) => Some(i.id),
                                    _ => None,
                                });
                            if was_aiming_at == Some(item.id) {
                                self.entities[*entity_id].clear_aiming();
                            }
                            log.log(format!("{} unequipped {}", self.entities[*entity_id].name, item.name));
                            self.entities[*entity_id].body.inventory.push(item);
                            self.entities[*entity_id].body.update_armor();
                        }
                    }
                },
            }
        }
        self.post_resolve(deathlist);
        animations
    }

    pub fn sync_active_item(&mut self, item_id: usize, location: ItemLocation) {
        if let Some(entry) = self.active_items.iter_mut().find(|e| e.item_id == item_id) {
            entry.location = location;
        } else {
            self.active_items.push(ActiveItem { item_id, location });
        }
    }

    fn handle_damage(&mut self, id: usize, part_index: usize, damage: Damage, deathlist: &mut Vec<usize>, log: &mut GameLog) {
        self.entities[id].apply_damage(part_index, damage);
        let is_player = Some(id) == self.player_id;
        if self.entities[id].mortally_wounded() && !deathlist.contains(&id) {
            if self.debug_mode && is_player {
                log.log(format!("{} would have died (debug mode).", self.entities[id].name));
            } else {
                log.log(format!("{} was killed!", self.entities[id].name));
                deathlist.push(id);
            }
        }
    }

    pub fn entity_for_pawn(&self, pawn: &Pawn) -> &Entity {
        &self.entities[pawn.entity_id]
    }

    fn handle_open_door(&mut self, pos: Point, actor_id: usize, log: &mut GameLog) {
        let index = self.map.pos_idx(pos);
        let entity_id = match &self.map.pawns[index] {
            Some(pawn) if self.entities[pawn.entity_id].kind == EntityKind::Door => pawn.entity_id,
            _ => return,
        };
        if let Some(door_color) = self.entities[entity_id].color {
            let has_key = self.entities[actor_id].body.inventory.iter().any(|item| {
                matches!(&item.kind, ItemKind::Key { color } if *color == door_color)
            });
            if !has_key {
                log.log("The door is locked.".to_string());
                return;
            }
        }
        self.entities[entity_id].clear_pawns(&mut self.map);
        self.update_views_near_event(pos, 10);
    }

    fn handle_destroy_wall(&mut self, pos: Point) {
        let index = self.map.pos_idx(pos);
        if self.map.tiles[index] == TileType::Wall {
            self.map.tiles[index] = TileType::Floor;
            self.update_views_near_event(pos, 10);
        }
    }

    fn handle_embark(&mut self, pilot_id: usize, vehicle_id: usize, log: &mut GameLog) {
        self.entities[pilot_id].driving = DrivingState::Driving(vehicle_id);
        self.entities[pilot_id].clear_pawns(&mut self.map);
        self.entities[vehicle_id].driving = DrivingState::DrivenBy(pilot_id);

        log.log(format!("{} entered {}", self.entities[pilot_id].name, self.entities[vehicle_id].name));

        if self.entities[pilot_id].id == self.player_id.unwrap() {
            self.entities[pilot_id].set_visible_tiles(&mut self.map, false);
            self.entities[vehicle_id].set_visible_tiles(&mut self.map, true);
            self.entities[self.player_id.unwrap()].kind = EntityKind::Actor;
            self.player_id = Some(vehicle_id);
            self.entities[vehicle_id].kind = EntityKind::Player;
        }
    }

    fn handle_disembark(&mut self, pilot_id: usize, vehicle_id: usize, log: &mut GameLog) {
        let vehicle_center = self.entities[vehicle_id].center();
        match self.map.nearest_free_pawn_position(vehicle_center) {
            Ok(pos) => {
                self.entities[pilot_id].driving = DrivingState::None;
                self.entities[vehicle_id].driving = DrivingState::Drivable;
                self.entities[pilot_id].position = pos;
                self.entities[pilot_id].create_pawns(&mut self.map);
                self.entities[vehicle_id].create_pawns(&mut self.map);
                self.entities[pilot_id].update_view(&mut self.map);

                if self.entities[vehicle_id].id == self.player_id.unwrap() {
                    self.entities[vehicle_id].set_visible_tiles(&mut self.map, false);
                    self.entities[pilot_id].set_visible_tiles(&mut self.map, true);
                    self.player_id = Some(pilot_id);
                    self.entities[vehicle_id].kind = EntityKind::Actor;
                    self.entities[pilot_id].kind = EntityKind::Player;
                }

                log.log(format!("{} left their vehicle", self.entities[pilot_id].name));
            },
            Err(_) => {
                log.log(format!("{} tried to disembark, but there is no room", self.entities[pilot_id].name));
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn resolve_status_effects(&mut self, log: &mut GameLog) {
        self.apply_noise_deafness();
        self.active_items_ticked = false;

        // Collect burn/status effects. Each entity only reads and writes its own
        // state, so this is safe to run across the thread pool.
        let effects: Vec<Effect> = if self.parallel_ai {
            use rayon::prelude::*;
            self.entities.par_iter_mut()
                .flat_map(|e| e.resolve_status_effects())
                .collect()
        } else {
            self.entities.iter_mut()
                .flat_map(|e| e.resolve_status_effects())
                .collect()
        };
        self.resolve_effects(&effects, log);

        // Update viewsheds:
        //   1. Clear the player's tile markings from the map (sequential — writes map).
        //   2. Recompute every viewshed (parallel — read-only map, writes own viewshed).
        //   3. Re-mark the player's freshly computed tiles (sequential — writes map).
        if let Some(id) = self.player_id {
            self.entities[id].set_visible_tiles(&mut self.map, false);
        }
        if self.parallel_ai {
            use rayon::prelude::*;
            let map = &self.map;
            self.entities.par_iter_mut().for_each(|e| e.update_viewshed_only(map));
        } else {
            let map = &self.map;
            self.entities.iter_mut().for_each(|e| e.update_viewshed_only(map));
        }
        if let Some(id) = self.player_id {
            self.entities[id].set_visible_tiles(&mut self.map, true);
        }

        self.clear_stale_entity_aim();
    }

    fn clear_stale_entity_aim(&mut self) {
        let player_id = match self.player_id {
            Some(id) => id,
            None => return,
        };

        let aim_info: Option<(usize, u32)> = {
            let player = &self.entities[player_id];
            let key = StatusEffect::AimingAtGround(Point { x: 0, y: 0 }, Item::pistol());
            match player.body.get_status_effect(&key) {
                Some(StatusEffect::AimingAtEntity(entity_id, item)) => {
                    let range = match item.kind {
                        ItemKind::Firearm { range, .. } => range,
                        _ => 0,
                    };
                    Some((*entity_id, range))
                },
                _ => None,
            }
        };

        let (target_id, range) = match aim_info {
            Some(info) => info,
            None => return,
        };

        let should_clear = match self.entities.get(target_id) {
            None => true,
            Some(target) => {
                let target_center = target.center();
                let in_sight = self.map.visible_tiles[self.map.pos_idx(target_center)];
                if !in_sight {
                    true
                } else {
                    let player_center = self.entities[player_id].center();
                    let dist = rltk::DistanceAlg::Pythagoras.distance2d(player_center, target_center);
                    dist > range as f32
                }
            }
        };

        if should_clear {
            self.entities[player_id].clear_aiming();
        }
    }

    fn apply_noise_deafness(&mut self) {
        let noise_per_entity: Vec<u32> = self.entities.iter().map(|entity| {
            let pos = entity.center();
            self.sounds_last_turn.iter()
                .filter_map(|s| {
                    let vol = s.volume as i32;
                    let dx = (pos.x - s.pos.x).abs();
                    let dy = (pos.y - s.pos.y).abs();
                    if dx > vol || dy > vol {
                        return None;
                    }
                    let dist = rltk::DistanceAlg::Pythagoras.distance2d(pos, s.pos);
                    if dist <= s.volume as f32 {
                        Some((s.volume as f32 - dist).max(0.0) as u32)
                    } else {
                        None
                    }
                })
                .sum()
        }).collect();

        for (i, entity) in self.entities.iter_mut().enumerate() {
            if noise_per_entity[i] > 2 * entity.body.noise_tolerance {
                entity.apply_status_effect(&StatusEffect::Deaf(3));
            }
        }
    }

    fn post_resolve(&mut self, deathlist: Vec<usize>) {
        struct DeathInfo {
            sprite: Sprite,
            position: Point,
            size_x: u32,
            size_y: u32,
            drops: Vec<Item>,
        }

        let death_infos: Vec<DeathInfo> = deathlist.iter().map(|&id| {
            let entity = &self.entities[id];
            let mut drops: Vec<Item> = entity.body.inventory.clone();
            for slot in &entity.body.item_slots {
                if let Some(item) = &slot.item {
                    if !item.proxy {
                        drops.push(item.clone());
                    }
                }
            }
            DeathInfo {
                sprite: entity.sprite.clone(),
                position: entity.position,
                size_x: entity.size_x,
                size_y: entity.size_y,
                drops,
            }
        }).collect();

        for id in &deathlist {
            self.entities[*id].kill(&mut self.map);
        }

        for info in death_infos {
            match info.sprite {
                Sprite::Human => {
                    let index = self.map.pos_idx(info.position);
                    let mut corpse = Item::corpse();
                    corpse.id = self.next_item_id;
                    self.next_item_id += 1;
                    self.map.items[index] = Some(corpse);
                },
                Sprite::Tank => {
                    for dx in 0..info.size_x as i32 {
                        for dy in 0..info.size_y as i32 {
                            let index = self.map.xy_idx(info.position.x + dx, info.position.y + dy);
                            let mut rubble = Item::rubble();
                            rubble.id = self.next_item_id;
                            self.next_item_id += 1;
                            self.map.items[index] = Some(rubble);
                        }
                    }
                },
                Sprite::Door => (),
            }
            for item in info.drops {
                let _ = self.add_item(info.position, item);
            }
        }

        self.entities.retain(|entity| {
            let should_be_dead = deathlist.iter().any(|&id| id == entity.id);
            return !should_be_dead;
        });

        for (i, entity) in self.entities.iter_mut().enumerate() {
            entity.id = i;
        }

        // Update player_id to the player's new index after compaction.
        // Returns None if the player was killed and removed.
        if self.player_id.is_some() {
            self.player_id = self.entities.iter().position(|e| {
                matches!(e.kind, crate::entity::EntityKind::Player)
            });
        }

        // Pawn entity_ids are now stale. Dead entities already cleared their pawns
        // via kill(); surviving pawns are in the right tiles but hold old IDs.
        // Update entity_id in-place.
        if !deathlist.is_empty() {
            let entities = &self.entities;
            let map = &mut self.map;
            for entity in entities {
                for x in 0..entity.size_x {
                    for y in 0..entity.size_y {
                        let idx = map.xy_idx(entity.position.x + x as i32, entity.position.y + y as i32);
                        if let Some(pawn) = map.pawns[idx].as_mut() {
                            pawn.entity_id = entity.id;
                        }
                    }
                }
            }
        }
    }

    fn update_views_near_event(&mut self, position: Point, radius: i32) {
        let entity_ids = self.map.get_entities_in_vicinity(position, radius);
        for id in entity_ids {
            self.entities[id].update_view(&mut self.map);
        }
    }

    fn spawn_loot(&mut self, zone_map: &ZoneMap, spawn_map: &SpawnMap,
                  depths: &[usize], rng: &mut RandomNumberGenerator) {
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

        // Sample rarity from each constructor once, then build a weighted pool.
        // Rarity R → weight (4 − R): common items appear more often.
        let item_meta: Vec<(MakeItem, u8)> =
            pool.iter().map(|&f| (f, f().rarity)).collect();
        let weighted_pool: Vec<(MakeItem, u8)> = item_meta.iter()
            .flat_map(|&(f, r)| {
                std::iter::repeat((f, r)).take(4usize.saturating_sub(r as usize))
            })
            .collect();
        if weighted_pool.is_empty() { return; }

        let nz = zone_map.zones.len();

        // Floor-tile spawn points per reachable zone.
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

        // Index indoor zones by depth for fast lookup.
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

            // Map rarity to a target depth: rarity 0 → shallow, rarity 3 → deep.
            let base = (rarity as usize * max_depth) / 3;
            let jitter = rng.range(-1i32, 4); // -1 .. +3
            let mut target = ((base as i32 + jitter).max(0) as usize).min(max_depth);

            // Depths 0-2 are much less likely
            if target < 4 && max_depth >= 4 && rng.range(0, 50) < 49 {
                target = rng.range(4, max_depth as i32 + 1) as usize;
            }

            // Find the nearest depth with at least one available (item-free) indoor zone.
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
    fn assign_door_colors(
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
    fn place_zone_keys(
        &mut self,
        zone_map: &ZoneMap,
        spawn_map: &SpawnMap,
        depths: &[usize],
        boundary_colors: &[Option<usize>],
        start_zone: usize,
        rng: &mut RandomNumberGenerator,
    ) {
        let n_zones = zone_map.zones.len();

        // Zone adjacency carrying lock color (None = free passage).
        let mut adj: Vec<Vec<(usize, Option<usize>)>> = vec![vec![]; n_zones];
        for (bi, b) in zone_map.boundaries.iter().enumerate() {
            let color = boundary_colors[bi];
            adj[b.zone_a].push((b.zone_b, color));
            adj[b.zone_b].push((b.zone_a, color));
        }

        // Dead-end spawn points per zone for preferred key placement.
        let zone_dead_ends: Vec<Vec<Point>> = (0..n_zones).map(|zi| {
            let zone_set: std::collections::HashSet<usize> =
                zone_map.zones[zi].iter().copied().collect();
            spawn_map.spawn_points.iter()
                .filter(|sp| sp.category == SpawnCategory::DeadEnd && zone_set.contains(&sp.idx))
                .map(|sp| sp.pos)
                .collect()
        }).collect();

        // Unique colors sorted by the depth of their shallowest locked boundary.
        let mut color_first_depth: std::collections::HashMap<usize, usize> =
            std::collections::HashMap::new();
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
            // Zones the player can reach before collecting this color's key.
            let reachable = reachable_zones(&adj, start_zone, n_zones, &unlocked);

            let mut candidates: Vec<usize> = (0..n_zones)
                .filter(|&zi| reachable[zi] && !zone_has_key[zi])
                .collect();

            // Shuffle and keep up to KEY_COPIES_PER_COLOR.
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

    fn init_static_entities(&mut self) {
        fn door_length(map: &Map, start_x: i32, start_y: i32, dx: i32, dy: i32) -> i32 {
            let mut x = start_x;
            let mut y = start_y;
            let mut length = 0;
            let mut done = false;
            while !done {
                if map.get_tile(x, y) == TileType::Doorway {
                    length += 1;
                    x += dx;
                    y += dy;
                    if x >= map.width as i32 || y >= map.height as i32 {
                        done = true;
                    }
                } else {
                    done = true;
                }
            }
            return length;
        }

        for x in 0..self.map.width as i32 {
            for y in 0..self.map.height as i32 {
                let index = self.map.xy_idx(x, y);
                if self.map.get_tile(x, y) == TileType::Doorway && self.map.pawns[index].is_none() {
                    let right_length = door_length(&self.map, x, y, 1, 0);
                    let down_length = door_length(&self.map, x, y, 0, 1);

                    let door;
                    if right_length > down_length {
                        door = Entity::new_door(self.entities.len(), self.map.idx_pos(index), Direction::Right, right_length as u32);
                    } else if right_length < down_length {
                        door = Entity::new_door(self.entities.len(), self.map.idx_pos(index), Direction::Up, down_length as u32);
                    } else if self.map.get_tile(x + 1, y) ==  TileType::Wall {
                        door = Entity::new_door(self.entities.len(), self.map.idx_pos(index), Direction::Right, 1);
                    } else {
                        door = Entity::new_door(self.entities.len(), self.map.idx_pos(index), Direction::Up, 1);
                    }
                    door.create_pawns(&mut self.map);
                    self.entities.push(door);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_worldsize(world: World, size: usize) -> World {
        assert_eq!(world.entities.len(), size, "Position vector is of incorrect size");
        world
    }

    #[test]
    fn create_player() {
        let mut world = World::new_test();

        let pos = Point {x: 0, y: 0};
        let facing = Direction::Up;
        let name = "Player";
        let result = world.create_player(pos, facing, String::from(name));

        assert!(result.is_ok());
        world = assert_worldsize(world, 1);
        let player = &world.entities[world.player_id.unwrap()];
        assert_eq!(player.position, pos);
        assert_eq!(player.name, name);
    }

    #[test]
    fn create_two_players_fails() {
        let mut world = World::new_test();

        let pos = Point {x: 0, y: 0};
        let facing = Direction::Up;
        let name = "Player";
        let _res = world.create_player(pos, facing, String::from(name));
        let result = world.create_player(Point {x: pos.x+1, y: pos.y+1}, facing, String::from("P2"));

        assert!(result.is_err());
        world = assert_worldsize(world, 1);
        let player = &world.entities[world.player_id.unwrap()];
        assert_eq!(player.position, pos);
        assert_eq!(player.name, name);
    }

    #[test]
    fn create_entity() {
        let mut world = World::new_test();

        let pos = Point {x: 0, y: 0};
        let facing = Direction::Up;
        let name = "Entity";
        let result = world.create_zombie_goon(pos, facing, String::from(name));

        assert!(result.is_ok());
        world = assert_worldsize(world, 1);
        assert_eq!(world.entities[0].position, pos);
        assert_eq!(world.entities[0].name, name);
    }

    #[test]
    fn create_two_entities() {
        let mut world = World::new_test();

        let pos = Point {x: 0, y: 0};
        let facing = Direction::Up;
        let name = "Entity";
        let result1 = world.create_zombie_goon(pos, facing, String::from(name));

        let pos2 = Point {x: pos.x + 1, y: pos.y + 1};
        let name2 = "Entity2";
        let result2 = world.create_zombie_goon(pos2, facing, String::from(name2));

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        world = assert_worldsize(world, 2);
        assert_eq!(world.entities[0].position, pos);
        assert_eq!(world.entities[0].name, name);
        assert_eq!(world.entities[1].position, pos2);
        assert_eq!(world.entities[1].name, name2);
    }

    #[test]
    fn create_two_entities_on_same_pos_places_second_nearby() {
        let mut world = World::new_test();

        let pos = Point {x: 0, y: 0};
        let facing = Direction::Up;
        let result1 = world.create_zombie_goon(pos, facing, String::from("Entity"));
        let result2 = world.create_zombie_goon(pos, facing, String::from("Entity2"));

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        world = assert_worldsize(world, 2);
        assert_eq!(world.entities[0].position, pos);
        assert_ne!(world.entities[1].position, pos);
    }

    #[test]
    fn deathlisted_entities_die_others_reordered() {
        let number_of_entities:usize = 5;
        let mut world = World::new_test();

        // Create a bunch of entities, named after their id
        let pos = Point {x: 0, y: 0};
        let facing = Direction::Up;
        for i in 0..number_of_entities {
            assert!(world.create_zombie_goon(Point{x: pos.x+i as i32, y: pos.y}, facing, format!("{}", i)).is_ok());
        }
        // doom a few
        let deathlist: Vec<usize> = vec![1,3,4];

        // execute the doomed ones
        world.post_resolve(deathlist.clone());

        // check that number of survivors is correct
        assert!(world.entities.len() == number_of_entities - deathlist.len());

        // check that surviving individuals are named and ordered correctly
        for (index, entity) in world.entities.iter().enumerate() {
            let old_id = entity.name.parse::<usize>().unwrap();
            let should_be_dead = deathlist.iter().any(|&id| id == old_id);

            assert!(!should_be_dead);
            assert!(entity.id == index);
            assert!(world.map.pawns[world.map.xy_idx(pos.x + 1, pos.y)].is_none());
            assert!(world.map.pawns[world.map.xy_idx(pos.x + 3, pos.y)].is_none());
            assert!(world.map.pawns[world.map.xy_idx(pos.x + 4, pos.y)].is_none());
        }
    }

    #[test]
    fn add_item_to_floor_works() {
        let mut world = World::new_test();
        let pos = Point {x: 1, y: 1};

        let _ = world.add_item(pos, Item::pistol());

        let index = world.map.xy_idx(pos.x, pos.y);
        assert!(world.map.items[index].is_some());
    }

    #[test]
    fn add_items_on_top_of_eachother_pushes_one_aside() {
        let mut world = World::new_test();
        let pos = Point {x: 1, y: 1};

        let _ = world.add_item(pos, Item::pistol());
        let _ = world.add_item(pos, Item::pistol());

        assert!(world.map.items.iter().filter(|i| i.is_some()).count() == 2);

        for (index, item) in world.map.items.iter().enumerate() {
            if item.is_some() {
                assert!(world.map.tiles[index] == TileType::Ground);
            }
        }
    }

    #[test]
    fn forward_goons_blocked_by_contested_center_tile() {
        let mut world = World::new_test();

        let center = Point { x: 10, y: 10 };

        // Place one goon on each cardinal side of the center, all facing inward.
        // On the first tick every one of them will declare an intent to enter (10, 10).
        assert!(world.create_forward_goon(Point { x: center.x,     y: center.y - 1 }, Direction::Down,  String::from("North")).is_ok());
        assert!(world.create_forward_goon(Point { x: center.x,     y: center.y + 1 }, Direction::Up,    String::from("South")).is_ok());
        assert!(world.create_forward_goon(Point { x: center.x - 1, y: center.y     }, Direction::Right, String::from("West")).is_ok());
        assert!(world.create_forward_goon(Point { x: center.x + 1, y: center.y     }, Direction::Left,  String::from("East")).is_ok());

        let start_positions: Vec<Point> = world.entities.iter().map(|e| e.position).collect();

        simulate_tick(&mut world);

        // All goons must remain at their starting positions — nobody may enter the
        // center tile because all four are trying to simultaneously.
        for (i, entity) in world.entities.iter().enumerate() {
            assert_eq!(
                entity.position, start_positions[i],
                "entity '{}' moved when it should have been blocked", entity.name
            );
        }
    }

    fn simulate_tick(world: &mut World) {
        let mut log = GameLog { entries: vec![] };
        world.resolve_intent_declaration();
        let mut phase = ExecutionPhase::Idle;
        loop {
            world.resolve_phase(phase, &mut log);
            match phase.next() {
                Some(next) => phase = next,
                None => break,
            }
        }
    }

    #[test]
    fn actors_walking_in_line() {
        let mut world = World::new_test();

        // Five goons start in a row, all facing the same direction, patrolling
        // to a destination further along the row. Each goon is one step behind
        // the next — they walk in single file and must not collide.
        let y = 5;
        let number_of_entities = 5;
        let destination = Point { x: 20, y };
        let waypoints = vec![destination];

        for i in 0..number_of_entities {
            let pos = Point { x: i as i32, y };
            assert!(world.create_patrolling_goon(pos, Direction::Right, format!("{}", i), waypoints.clone()).is_ok());
        }

        for _ in 0..30 {
            simulate_tick(&mut world);
        }

        // All entities must still be alive
        assert_eq!(world.entities.len(), number_of_entities);

        // No two entities may occupy the same tile
        let positions: Vec<Point> = world.entities.iter().map(|e| e.position).collect();
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                assert_ne!(positions[i], positions[j], "entities {} and {} share a position", i, j);
            }
        }

        // The leading entity (started furthest ahead) must have advanced toward the destination
        let leader_start_x = (number_of_entities - 1) as i32;
        let leader = world.entities.iter().find(|e| e.name == format!("{}", number_of_entities - 1)).unwrap();
        assert!(leader.position.x > leader_start_x, "leader entity did not move");
    }
}