use super::*;
use rltk::{Point, RandomNumberGenerator};
use strum::IntoEnumIterator;
use std::collections::{HashMap, HashSet};
use crate::animation::explosion_animation;

pub struct ActiveItem {
    pub item_id: usize,
    pub location: ItemLocation,
}

/// The contents of the game world itself.
pub struct World {
    pub player_id: Option<usize>,
    pub player_xp: usize,
    pub player_level: usize,
    /// XP required to reach the next level.
    pub player_xp_to_next_level: usize,
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


impl World {
    /// Create new world.
    /// # Arguments
    /// * `size` - Number of blocks that make up one size of the map.
    pub fn new(size: usize, seed: u64) -> Self {
        let mut rng = RandomNumberGenerator::seeded(seed);
        let mut world = World {
            player_id: Option::None,
            player_xp: 0,
            player_level: 0,
            player_xp_to_next_level: 1000,
            entities: vec![],
            next_item_id: 0,
            pending_levelup: false,
            sounds: vec![],
            sounds_last_turn: vec![],
            active_items: vec![],
            active_items_ticked: false,
            map: Map::new_game_map(size, &mut rng),
            debug_mode: false,
            parallel_ai: true,
        };

        let player_pos = Point {x: (world.map.width / 2) as i32, y: (world.map.height / 2) as i32};
        let _result = world.create_player(player_pos,
            Direction::Up,
            String::from("Player"));

        world.init_static_entities();

        // Region depths from the player's starting tile, used for placement weighting.
        // Note that this needs recalculating as below since player_pos may be impassable.
        let player_tile = world.get_player()
            .map(|p| world.map.pos_idx(p.position))
            .unwrap_or_else(|_| world.map.xy_idx(
                (world.map.width / 2) as i32, (world.map.height / 2) as i32));
        // Topology analysis — run once, shared by all placement passes.
        let spawn_map = create_spawn_map(&world.map, player_tile);
        
        if true {
            let boundary_colors = world.assign_door_colors(&spawn_map);
            let start_region = spawn_map.tile_region[player_tile].unwrap_or(0);
            world.place_zone_keys(&spawn_map, &boundary_colors, start_region, &mut rng);
            spawn_loot(&mut world, &spawn_map, &mut rng);

            let mut placed: Vec<Point> = Vec::new();
            let mut guard_n = 0usize;
            println!("Spawning guards:");
            world.spawn_sentinels(&spawn_map, &mut placed, &mut guard_n, &mut rng);
            world.spawn_patrollers(&spawn_map, &mut placed, &mut guard_n, &mut rng);
            println!("Spawned {} guards total.", guard_n);
        } else {
            world.spawn_debug(&spawn_map);
        }

        return world;
    }

    /// Create new world for testing purposes.
    pub fn new_test() -> Self {
        Self {
            player_id: Option::None,
            player_xp: 0,
            player_level: 0,
            player_xp_to_next_level: 1000,
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

    /// Small visual testbed for tuning AI behaviour: a 3×3-block map of randomly
    /// assembled centre ("middle") blocks with two circular patrol routes, each
    /// walked by one patrolling actor. Block edge/boundary conditions are
    /// intentionally ignored — we only need obstacles for the AI to act on.
    /// (The `ai_benchmark` test is the separate performance testbed.)
    pub fn new_ai_testbed() -> Self {
        const GRID: usize = 3;
        let dim = GRID * BLOCK_SIZE;
        let mut rng = RandomNumberGenerator::new();

        let mut map = Map::new_empty_map(dim, dim);
        let middle = generate_blocks("middleblock");
        if !middle.is_empty() {
            for bx in 0..GRID {
                for by in 0..GRID {
                    let block = &middle[rng.range(0, middle.len() as i32) as usize];
                    for x in 0..BLOCK_SIZE {
                        for y in 0..BLOCK_SIZE {
                            let idx = map.xy_idx((bx * BLOCK_SIZE + x) as i32, (by * BLOCK_SIZE + y) as i32);
                            map.tiles[idx] = block.tiles[block_xy_idx(x, y)];
                        }
                    }
                }
            }
        }

        let mut world = World {
            player_id: Option::None,
            player_xp: 0,
            player_level: 0,
            player_xp_to_next_level: 1000,
            entities: vec![],
            next_item_id: 0,
            pending_levelup: false,
            sounds: vec![],
            sounds_last_turn: vec![],
            active_items: vec![],
            active_items_ticked: false,
            map,
            debug_mode: false,
            parallel_ai: false,
        };

        let center = Point { x: (dim / 2) as i32, y: (dim / 2) as i32 };
        let _ = world.create_player(center, Direction::Up, String::from("Player"));

        // Two concentric circular patrol routes, one patroller each.
        world.spawn_test_patroller(center, 18, 12, "Patroller A");
        world.spawn_test_patroller(center, 32, 16, "Patroller B");

        let _ = world.add_item(center, Item::jetpack());
        let _ = world.add_item(center, Item::rocket_boots());

        world
    }

    /// Register a circular patrol route of `waypoints` points at `radius` around
    /// `center` (each snapped to a walkable tile) and spawn one patrolling actor
    /// on it. Helper for [`World::new_ai_testbed`].
    fn spawn_test_patroller(&mut self, center: Point, radius: i32, waypoints: usize, name: &str) {
        use std::f32::consts::TAU;
        let route: Vec<Point> = (0..waypoints).map(|i| {
            let theta = TAU * (i as f32) / (waypoints as f32);
            let raw = Point {
                x: center.x + (radius as f32 * theta.cos()).round() as i32,
                y: center.y + (radius as f32 * theta.sin()).round() as i32,
            };
            self.map.snap_to_walkable(raw)
        }).collect();

        let start = route[0];
        let route_id = self.map.register_patrol_route(route);
        let _ = self.create_patrol_actor(start, Direction::Up, name.to_string(), route_id, 0, CombatTactic::Pursue);
    }

    pub fn create_player(&mut self, pos: Point, facing: Direction, name: String) -> Result<(), GameError> {
        if self.entities.len() > 0 {
            return Err(GameError {
                error: Error::BadPrecondition,
                message: String::from("Tried to create player, but entities already exist")
            });
        }

        let nearest_pos = self.map.nearest_free_pawn_position(pos)?;

        let mut player = Entity::human(0, nearest_pos, facing, name);
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

    // Old enemy creation functions. TODO: REMOVE ----------------------
    /// Creates an NPC with the full profile+alert AI system.
    pub fn create_actor(&mut self, pos: Point, facing: Direction, name: String, profile: Profile) -> Result<(), GameError> {
        let actual_pos = self.map.nearest_free_pawn_position(pos)?;
        let mut entity = Entity::human(self.entities.len(), actual_pos, facing, name);
        entity.ai = AI::Actor(ActorAI::new(profile));
        entity.paper_doll = Some(PaperDoll::MaleSilhouette);
        self.equip_pistol(&mut entity);
        entity.create_pawns(&mut self.map);
        self.entities.push(entity);
        Ok(())
    }

    pub fn create_patrol_actor(&mut self, pos: Point, facing: Direction, name: String, route_id: usize, waypoint_index: usize, tactic: CombatTactic) -> Result<(), GameError> {
        self.create_actor(pos, facing, name, Profile::Patrol {
            route_id,
            waypoint_index,
            combat_tactic: tactic,
        })
    }

    pub fn create_guard_actor(&mut self, pos: Point, facing: Direction, name: String, tactic: CombatTactic) -> Result<(), GameError> {
        let anchor = self.map.nearest_free_pawn_position(pos)?;
        self.create_actor(pos, facing, name, Profile::Guard { anchor, combat_tactic: tactic })
    }

    fn equip_pistol(&mut self, entity: &mut Entity) {
        let mut pistol = Item::pistol();
        pistol.id = self.next_item_id;
        self.next_item_id += 1;
        let _ = entity.body.equip(pistol);
        entity.body.update_armor();
    }
    // End of old actor creation functions       ----------------------

    // Enemy creation --------------------------
    pub fn create_light_guard(&mut self, pos: Point, facing: Direction) -> Result<usize, GameError> {
        let idx = self.create_enemy_base(pos, facing, "Guard".to_string())?;
        self.carry_item(idx, Item::pistol)?;
        self.carry_item(idx, Item::knife)?;
        self.carry_item(idx, Item::flashbang)?;
        self.equip_item(idx, Item::bulletproof_vest)?;
        self.equip_item(idx, Item::helmet)?;
        Ok(idx)
    }

    pub fn create_medium_guard(&mut self, pos: Point, facing: Direction) -> Result<usize, GameError> {
        let idx = self.create_enemy_base(pos, facing, "Sentinel".to_string())?;
        self.carry_item(idx, Item::assault_rifle)?;
        self.carry_item(idx, Item::ammo_bullets)?;
        self.carry_item(idx, Item::grenade)?;
        self.equip_item(idx, Item::riot_armor)?;
        self.equip_item(idx, Item::riot_pants)?;
        self.equip_item(idx, Item::helmet)?;
        Ok(idx)
    }

    pub fn create_heavy_guard(&mut self, pos: Point, facing: Direction) -> Result<usize, GameError> {
        let idx = self.create_enemy_base(pos, facing, "Paladin".to_string())?;
        self.carry_item(idx, Item::machinegun)?;
        self.carry_item(idx, Item::ammo_bullets)?;
        self.carry_item(idx, Item::grenade)?;
        self.equip_item(idx, Item::heavy_combat_suit)?;
        self.equip_item(idx, Item::heavy_helmet)?;
        Ok(idx)
    }

    pub fn create_flamer_guard(&mut self, pos: Point, facing: Direction) -> Result<usize, GameError> {
        let idx = self.create_enemy_base(pos, facing, "Purifier".to_string())?;
        self.carry_item(idx, Item::flamethrower)?;
        self.carry_item(idx, Item::ammo_fuel)?;
        self.carry_item(idx, Item::fire_grenade)?;
        self.equip_item(idx, Item::heavy_combat_suit)?;
        self.equip_item(idx, Item::heavy_helmet)?;
        Ok(idx)
    }

    pub fn create_rocket_guard(&mut self, pos: Point, facing: Direction) -> Result<usize, GameError> {
        let idx = self.create_enemy_base(pos, facing, "Tankbuster".to_string())?;
        self.carry_item(idx, Item::rocket_launcher)?;
        self.carry_item(idx, Item::ammo_rockets)?;
        self.carry_item(idx, Item::grenade)?;
        self.carry_item(idx, Item::grenade)?;
        self.equip_item(idx, Item::riot_armor)?;
        self.equip_item(idx, Item::riot_pants)?;
        self.equip_item(idx, Item::heavy_helmet)?;
        Ok(idx)
    }

    pub fn create_riot_guard(&mut self, pos: Point, facing: Direction) -> Result<usize, GameError> {
        let idx = self.create_enemy_base(pos, facing, "Peacekeeper".to_string())?;
        self.carry_item(idx, Item::shock_carbine)?;
        self.carry_item(idx, Item::ammo_batteries)?;
        self.carry_item(idx, Item::shock_grenade)?;
        self.carry_item(idx, Item::flashbang)?;
        self.equip_item(idx, Item::riot_armor)?;
        self.equip_item(idx, Item::riot_pants)?;
        self.equip_item(idx, Item::helmet)?;
        Ok(idx)
    }

    pub fn create_civilian(&mut self, pos: Point, facing: Direction) -> Result<usize, GameError> {
        let idx = self.create_enemy_base(pos, facing, "Civilian".to_string())?;
        self.carry_item(idx, Item::knife)?;
        Ok(idx)
    }

    pub fn create_pilot(&mut self, pos: Point, facing: Direction) -> Result<usize, GameError> {
        let idx = self.create_enemy_base(pos, facing, "Pilot".to_string())?;
        self.carry_item(idx, Item::knife)?;
        self.carry_item(idx, Item::shock_pistol)?;
        Ok(idx)
    }

    fn create_enemy_base(&mut self, pos: Point, facing: Direction, name: String) -> Result<usize, GameError> {
        let actual_pos = self.map.nearest_free_pawn_position(pos)?;
        let idx = self.entities.len();
        let mut entity = Entity::human(idx, actual_pos, facing, name);
        entity.paper_doll = Some(PaperDoll::MaleSilhouette);
        entity.create_pawns(&mut self.map);
        self.entities.push(entity);
        Ok(idx)
    }

    fn equip_item(&mut self, entity_idx: usize, item_maker: MakeItem) -> Result<(), GameError> {
        let mut item = item_maker();
        item.id = self.next_item_id;
        self.entities[entity_idx].body.inventory.push(item);
        self.next_item_id += 1;
        self.entities[entity_idx].body.update_armor();
        Ok(())
    }

    fn carry_item(&mut self, entity_idx: usize, item_maker: MakeItem) -> Result<(), GameError> {
        let mut item = item_maker();
        item.id = self.next_item_id;
        self.entities[entity_idx].body.inventory.push(item);
        self.next_item_id += 1;
        self.entities[entity_idx].body.update_armor();
        Ok(())
    }


    pub fn create_tank(&mut self, pos: Point, facing: Direction, name: String) -> Result<(), GameError> {
        let pos = self.map.nearest_free_pawn_position_sized(pos, 3, 3)?;

        let mut tank = Entity::tank(self.entities.len(), pos, facing, name);
        let mut cannon = Item::mounted_cannon();
        cannon.id = self.next_item_id;
        self.next_item_id += 1;
        let _ = tank.body.equip(cannon);

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
        // Step 0: Maintain shared flow fields for the goals actors will navigate
        // to — both static (patrol waypoints / guard anchors) and dynamic
        // (investigation origins / last-known positions carried over from prior
        // turns). A field only beats per-agent A* when many agents descend the
        // same one, so we count demand per exact goal cell and build only goals
        // shared by >= FIELD_DEMAND_THRESHOLD actors (beliefs derive from shared
        // events — the same sound pos, the same sighting — so groups naturally
        // land on identical cells). Others fall back to A* in navigate_to, so
        // this is a safe no-op when goals are distinct. Dynamic goals get
        // radius-bounded fields (interested agents cluster near the goal); static
        // goals get full-map fields. Fields persist across turns and are evicted
        // once their goal goes undemanded.
        //
        // Read before this turn's stimulus is processed, so a just-changed belief
        // simply misses its field for one turn. Built serially under &mut map so
        // the read-only intent loop below can read fields under a shared borrow.
        // Skipped entirely when flow fields are disabled — navigation then falls
        // back to pure A* (used by the benchmark to compare the two).
        if self.map.use_flow_fields {
            const FIELD_DEMAND_THRESHOLD:    usize = 12;   // min actors sharing a goal
            const MAX_FIELD_BUILDS_PER_TURN: usize = 4;    // backstop against spikes
            const DYNAMIC_FIELD_MAX_COST:    u32   = 800;  // ~80-tile investigation radius
            const FIELD_EVICT_TTL:           u32   = 60;   // turns undemanded before eviction
            const FIELD_CACHE_CAP:           usize = 64;   // hard backstop on resident fields

            // Count demand per goal cell, tracking whether a bounded field suffices.
            let mut demand: HashMap<usize, (usize, bool)> = HashMap::new();
            {
                let map = &self.map;
                for e in &self.entities {
                    if let AI::Actor(actor) = &e.ai {
                        if let Some((p, bounded)) = actor.nav_field_goal() {
                            let entry = demand.entry(map.pos_idx(p)).or_insert((0, bounded));
                            entry.0 += 1;
                            entry.1 &= bounded; // any full-map (static) requester wins
                        }
                    }
                }
            }

            // Evict fields whose goal is no longer demanded (TTL hysteresis + cap).
            let demanded: HashSet<usize> = demand.keys().copied().collect();
            self.map.evict_fields(&demanded, FIELD_EVICT_TTL, FIELD_CACHE_CAP);

            // Build the most-demanded new goals, gated by threshold and per-turn cap.
            let mut popular: Vec<(usize, usize, bool)> = demand.into_iter()
                .filter(|&(goal, (n, _))| n >= FIELD_DEMAND_THRESHOLD && self.map.field_for(goal).is_none())
                .map(|(goal, (n, bounded))| (goal, n, bounded))
                .collect();
            popular.sort_unstable_by(|a, b| b.1.cmp(&a.1));
            for (goal, _, bounded) in popular.into_iter().take(MAX_FIELD_BUILDS_PER_TURN) {
                if bounded {
                    self.map.ensure_field_bounded(goal, DYNAMIC_FIELD_MAX_COST);
                } else {
                    self.map.ensure_field(goal);
                }
            }
        }

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

        let compute = |(ai, entity): (&mut AI, &Entity)| -> Option<Intent> {
            match entity.driving {
                DrivingState::Driving(_)  => None,
                DrivingState::DrivenBy(_) => None,
                _ => ai.compute_intent(entity, map, entities, sounds),
            }
        };

        let mut intents: Vec<Option<Intent>> = if self.parallel_ai {
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

        // Step 3: Resolve vehicle-pilot pairs sequentially.
        for i in 0..entities.len() {
            if let DrivingState::DrivenBy(pilot_id) = entities[i].driving {
                let intent = ai_states[pilot_id].compute_intent(
                    &entities[i], map, entities, sounds,
                );
                intents[i] = intent;
            }
        }

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
                    effects.push(Effect::ApplyStatus { target_id: entity.index, status: StatusEffect::Blind(5) });
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
                        effects.push(Effect::Damage { entity_id: entity.index, bodypart_index: part, raw_damage: damage });
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
                Effect::RegenTick{entity_id: id, bodypart_index: part_index} =>
                    self.entities[*id].heal(*part_index, 1),
                Effect::SyncActiveItem{item_id, location} =>
                    self.sync_active_item(*item_id, location.clone()),
                Effect::Sound(sound) =>
                    self.sounds.push(sound.clone()),
                Effect::Twist{entity_id, direction} => {
                    self.entities[*entity_id].intent = turn_intent(*direction);
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
                    self.entities[*entity_id].clear_scanning();
                },
                Effect::SetFacing { entity_id, direction } => {
                    self.entities[*entity_id].body.facing = *direction;
                    let pos = self.entities[*entity_id].position;
                    self.entities[*entity_id].set_position(pos, &mut self.map);
                    self.entities[*entity_id].clear_aiming();
                    self.entities[*entity_id].clear_scanning();
                },
                Effect::ConsumeAmmo { entity_id, slot, shots } => {
                    if let Some(item) = self.entities[*entity_id].get_equipped_item(*slot) {
                        if let ItemKind::Firearm { ref mut ammo, .. } = item.kind {
                            *ammo = ammo.saturating_sub(*shots);
                        }
                    }
                },
                Effect::ReloadWeapon { entity_id, weapon_id } => {
                    let ent = &mut self.entities[*entity_id];
                    // Read the reloadable's ammo kind and how much it is missing (firearm or powered gear).
                    let spec = ent.find_item_by_id(*weapon_id)
                        .and_then(|i| i.kind.reloadable())
                        .map(|(cur, max, kind)| (kind, max.saturating_sub(cur)));
                    if let Some((kind, need)) = spec {
                        if need > 0 {
                            // Drain matching ammo boxes in the inventory until full or the boxes run dry.
                            let mut loaded = 0u32;
                            for item in ent.body.inventory.iter_mut() {
                                if loaded >= need { break; }
                                if let ItemKind::Ammo { kind: k, charges } = &mut item.kind {
                                    if *k == kind && *charges > 0 {
                                        let take = (*charges).min(need - loaded);
                                        *charges -= take;
                                        loaded += take;
                                    }
                                }
                            }
                            if loaded > 0 {
                                if let Some(w) = ent.find_item_by_id_mut(*weapon_id) {
                                    w.kind.add_charges(loaded);
                                }
                                ent.body.inventory.retain(|i| !matches!(&i.kind, ItemKind::Ammo { charges: 0, .. }));
                                let wname = ent.find_item_by_id(*weapon_id).map(|i| i.name.clone()).unwrap_or_default();
                                log.log(format!("{} reloaded {} (+{} {})", ent.name, wname, loaded, kind.name()));
                            }
                        }
                    }
                },
                Effect::ApplyRegeneration { entity_id, bodypart_index, turns } => {
                    let ent = &mut self.entities[*entity_id];
                    let nparts = ent.body.parts.len();
                    // Merge with any existing regeneration; the HashSet is keyed by variant,
                    // so the whole per-part vector is replaced in one entry.
                    let mut counts = match ent.body.get_status_effect(&StatusEffect::Regenerating(vec![])) {
                        Some(StatusEffect::Regenerating(v)) => v.clone(),
                        _ => vec![0u32; nparts],
                    };
                    counts.resize(nparts, 0);
                    match bodypart_index {
                        Some(i) if *i < nparts => counts[*i] = counts[*i].max(*turns),
                        Some(_) => {},
                        None => counts.iter_mut().for_each(|c| *c = (*c).max(*turns)),
                    }
                    ent.body.remove_status_effect(&StatusEffect::Regenerating(vec![]));
                    ent.body.apply_status_effect(&StatusEffect::Regenerating(counts));
                },
                Effect::ConsumeItem { entity_id, item_id } => {
                    self.entities[*entity_id].take_item_by_id(*item_id);
                },
                Effect::ConsumeCharge { entity_id, item_id } => {
                    if let Some(item) = self.entities[*entity_id].find_item_by_id_mut(*item_id) {
                        item.kind.spend_charge();
                    }
                },
                Effect::ApplyScan { entity_id, target } => {
                    // Replace any prior scan so re-aiming updates the cone direction.
                    let body = &mut self.entities[*entity_id].body;
                    body.remove_status_effect(&StatusEffect::Scanning(Point { x: 0, y: 0 }));
                    body.apply_status_effect(&StatusEffect::Scanning(*target));
                },
                Effect::SpendEnergy { entity_id, amount } => {
                    self.entities[*entity_id].body.energy =
                        self.entities[*entity_id].body.energy.saturating_sub(*amount);
                },
                Effect::RestoreEnergy { entity_id, amount } => {
                    let body = &mut self.entities[*entity_id].body;
                    body.energy = body.energy.saturating_add(*amount).min(body.max_energy);
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
                            // Removing the recon helmet ends its vision cone.
                            if slot == SlotType::Headwear {
                                self.entities[*entity_id].clear_scanning();
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

    pub fn check_levelup(&mut self) {
        if self.player_level >= 10 {
            return;
        }
        if self.player_xp >= self.player_xp_to_next_level {
            self.player_xp -= self.player_xp_to_next_level;
            self.player_xp_to_next_level += 1000;
            self.player_level += 1;
            self.pending_levelup = true;
        }
    }

    pub fn sync_active_item(&mut self, item_id: usize, location: ItemLocation) {
        if let Some(entry) = self.active_items.iter_mut().find(|e| e.item_id == item_id) {
            entry.location = location;
        } else {
            self.active_items.push(ActiveItem { item_id, location });
        }
    }

    fn handle_damage(&mut self, id: usize, part_index: usize, damage: Damage, deathlist: &mut Vec<usize>, log: &mut GameLog) {
        let disabled = |e: &Entity| {
            let p = &e.body.parts[part_index];
            p.damage > p.max_damage
        };
        let was_disabled = disabled(&self.entities[id]);
        self.entities[id].apply_damage(part_index, damage);
        if !was_disabled && disabled(&self.entities[id]) {
            self.drop_weapons_from_disabled_part(id, part_index, log);
        }
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

    /// When a body part is disabled, drop any non-locked weapon held in its slots to the
    /// ground. Proxy slots (two-handed weapons) share the real item's id, so the actual
    /// weapon is located, unequipped from its primary slot, and dropped.
    fn drop_weapons_from_disabled_part(&mut self, id: usize, part_index: usize, log: &mut GameLog) {
        // Collect the ids of non-locked weapons occupying the disabled part's slots.
        let weapon_ids: Vec<usize> = {
            let body = &self.entities[id].body;
            let mut ids = vec![];
            for &slot_idx in &body.parts[part_index].slot_index {
                let Some(slot_item) = &body.item_slots[slot_idx].item else { continue };
                let item_id = slot_item.id;
                // Locate the real (non-proxy) item sharing this id and test it, not the proxy
                // (a proxy is never flagged locked, so the real item is the source of truth).
                let real = body.item_slots.iter()
                    .find_map(|s| s.item.as_ref().filter(|it| it.id == item_id && !it.proxy));
                if let Some(real) = real {
                    let is_weapon = matches!(real.kind, ItemKind::Firearm { .. } | ItemKind::MeleeWeapon { .. });
                    if !real.locked && is_weapon && !ids.contains(&item_id) {
                        ids.push(item_id);
                    }
                }
            }
            ids
        };

        for item_id in weapon_ids {
            // Unequip from the primary (non-proxy) slot; this clears any proxy slots too.
            let real_slot = self.entities[id].body.item_slots.iter()
                .find(|s| s.item.as_ref().map_or(false, |it| it.id == item_id && !it.proxy))
                .map(|s| s.slot_type);
            let Some(slot) = real_slot else { continue };
            let Some(item) = self.entities[id].body.unequip(slot) else { continue };

            // Stop aiming if the dropped weapon was the aimed one.
            let aiming_id = self.entities[id].body.status_effects.iter().find_map(|s| match s {
                StatusEffect::AimingAtGround(_, i) => Some(i.id),
                StatusEffect::AimingAtEntity(_, i) => Some(i.id),
                _ => None,
            });
            if aiming_id == Some(item.id) {
                self.entities[id].clear_aiming();
            }
            self.entities[id].body.update_armor();

            let pos = self.entities[id].position;
            let item_name = item.name.clone();
            if let Ok(drop_pos) = self.map.nearest_free_item_position(pos) {
                let map_idx = self.map.pos_idx(drop_pos);
                self.map.items[map_idx] = Some(item);
                log.log(format!("{} dropped {} from a disabled {}",
                    self.entities[id].name, item_name, self.entities[id].body.parts[part_index].name));
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
            // Terrain changed: resident flow fields are baked over static terrain
            // and are now stale (they'd miss the opening). Drop them; Step 0
            // rebuilds the still-demanded ones next turn within its build budget.
            self.map.invalidate_fields();
            self.update_views_near_event(pos, 10);
        }
    }

    fn handle_embark(&mut self, pilot_id: usize, vehicle_id: usize, log: &mut GameLog) {
        self.entities[pilot_id].driving = DrivingState::Driving(vehicle_id);
        self.entities[pilot_id].clear_pawns(&mut self.map);
        self.entities[vehicle_id].driving = DrivingState::DrivenBy(pilot_id);

        log.log(format!("{} entered {}", self.entities[pilot_id].name, self.entities[vehicle_id].name));

        if self.entities[pilot_id].index == self.player_id.unwrap() {
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

                if self.entities[vehicle_id].index == self.player_id.unwrap() {
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

    /// The final stage of turn events.
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
                    if !item.proxy && !item.locked {
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

        for index in &deathlist {
            self.kill_entity(*index);
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
            let should_be_dead = deathlist.iter().any(|&id| id == entity.index);
            return !should_be_dead;
        });

        // TODO: This compaction causes a bug, since AI's sometimes rely on stable entity ID's. Reconsider this approach.
        for (i, entity) in self.entities.iter_mut().enumerate() {
            entity.index = i;
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
                            pawn.entity_id = entity.index;
                        }
                    }
                }
            }
        }
    }

    /// Apply death effects
    fn kill_entity(&mut self, index: usize) {
        let entity = &mut self.entities[index];
        entity.clear_pawns(&mut self.map);
        self.player_xp += entity.xp_value;
        println!("Player got {} xp", entity.xp_value);
    }

    fn update_views_near_event(&mut self, position: Point, radius: i32) {
        let entity_ids = self.map.get_entities_in_vicinity(position, radius);
        for id in entity_ids {
            self.entities[id].update_view(&mut self.map);
        }
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
                        door = Entity::door(self.entities.len(), self.map.idx_pos(index), Direction::Right, right_length as u32);
                    } else if right_length < down_length {
                        door = Entity::door(self.entities.len(), self.map.idx_pos(index), Direction::Up, down_length as u32);
                    } else if self.map.get_tile(x + 1, y) ==  TileType::Wall {
                        door = Entity::door(self.entities.len(), self.map.idx_pos(index), Direction::Right, 1);
                    } else {
                        door = Entity::door(self.entities.len(), self.map.idx_pos(index), Direction::Up, 1);
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

    fn create_zombie(world: &mut World, pos: Point, facing: Direction, name: String, ai: AI) -> Result<(), GameError>  {
        let actual_pos = world.map.nearest_free_pawn_position(pos)?;
        let mut entity = Entity::human(world.entities.len(), actual_pos, facing, name);
        entity.ai = ai;
        entity.paper_doll = Some(PaperDoll::MaleSilhouette);
        world.equip_pistol(&mut entity);
        entity.create_pawns(&mut world.map);
        world.entities.push(entity);

        Ok(())        
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
        let result = create_zombie(&mut world, pos, facing, String::from(name), AI::Rotator);

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
        let result1 = create_zombie(&mut world, pos, facing, String::from(name), AI::Rotator);

        let pos2 = Point {x: pos.x + 1, y: pos.y + 1};
        let name2 = "Entity2";
        let result2 = create_zombie(&mut world, pos2, facing, String::from(name2), AI::Rotator);

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
        let result1 = create_zombie(&mut world, pos, facing, String::from("Entity"), AI::Rotator);
        let result2 = create_zombie(&mut world, pos, facing, String::from("Entity2"), AI::Rotator);

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
            assert!(create_zombie(&mut world, Point{x: pos.x+i as i32, y: pos.y}, facing, format!("{}", i), AI::Rotator).is_ok());
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
            assert!(entity.index == index);
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

    /// Create a test world with a player holding a pistol whose ammo is set to
    /// `start_ammo`, plus `ammo_box` in the inventory. Returns (world, player_id, weapon_id).
    fn world_with_armed_player(start_ammo: u32, ammo_box: Item) -> (World, usize, usize) {
        let mut world = World::new_test();
        let _ = world.create_player(Point { x: 50, y: 50 }, Direction::Up, String::from("Player"));
        let player_id = world.player_id.unwrap();
        let weapon_id = 1;

        let player = world.get_player_mut().unwrap();
        let mut pistol = Item::pistol();
        pistol.id = weapon_id;
        if let ItemKind::Firearm { ammo, .. } = &mut pistol.kind { *ammo = start_ammo; }
        let _ = player.body.equip(pistol);
        player.body.inventory.push(ammo_box);

        (world, player_id, weapon_id)
    }

    #[test]
    fn reload_fills_weapon_and_drains_ammo_box() {
        // Pistol at 3/12, one 30-round box of bullets.
        let (mut world, player_id, weapon_id) = world_with_armed_player(3, Item::ammo_bullets());

        let mut log = GameLog { entries: vec![] };
        world.resolve_effects(&vec![Effect::ReloadWeapon { entity_id: player_id, weapon_id }], &mut log);

        let player = world.get_player().unwrap();
        let ammo = match &player.find_item_by_id(weapon_id).unwrap().kind {
            ItemKind::Firearm { ammo, .. } => *ammo,
            _ => panic!("weapon is not a firearm"),
        };
        assert_eq!(ammo, 12, "pistol should be topped up to capacity");

        // 9 rounds consumed from the 30-round box → 21 remain, box kept.
        let box_charges = player.body.inventory.iter().find_map(|i| match &i.kind {
            ItemKind::Ammo { charges, .. } => Some(*charges),
            _ => None,
        });
        assert_eq!(box_charges, Some(21), "box should retain leftover charges");
    }

    #[test]
    fn reload_removes_emptied_ammo_boxes() {
        // A box with fewer rounds (5) than the empty pistol needs (12).
        let mut small_box = Item::ammo_bullets();
        if let ItemKind::Ammo { charges, .. } = &mut small_box.kind { *charges = 5; }
        let (mut world, player_id, weapon_id) = world_with_armed_player(0, small_box);

        let mut log = GameLog { entries: vec![] };
        world.resolve_effects(&vec![Effect::ReloadWeapon { entity_id: player_id, weapon_id }], &mut log);

        let player = world.get_player().unwrap();
        let ammo = match &player.find_item_by_id(weapon_id).unwrap().kind {
            ItemKind::Firearm { ammo, .. } => *ammo,
            _ => panic!("weapon is not a firearm"),
        };
        assert_eq!(ammo, 5, "gun receives only what the box held");
        assert!(!player.body.inventory.iter().any(|i| matches!(i.kind, ItemKind::Ammo { .. })),
            "emptied ammo box should be discarded");
    }

    /// Advance one status-resolution round (heal/burn ticks + duration decay), the
    /// same sequence `World::resolve_status_effects` runs, minus viewshed upkeep.
    fn tick_status_round(world: &mut World, entity_id: usize, log: &mut GameLog) {
        let effects = world.entities[entity_id].resolve_status_effects();
        world.resolve_effects(&effects, log);
    }

    #[test]
    fn regeneration_heals_one_hp_per_turn_then_expires() {
        let mut world = World::new_test();
        let _ = world.create_player(Point { x: 50, y: 50 }, Direction::Up, String::from("Player"));
        let player_id = world.player_id.unwrap();
        world.entities[player_id].body.parts[0].damage = 3;

        let mut log = GameLog { entries: vec![] };
        world.resolve_effects(
            &vec![Effect::ApplyRegeneration { entity_id: player_id, bodypart_index: Some(0), turns: 10 }],
            &mut log);

        // Three rounds fully heal the 3 points of damage.
        for _ in 0..3 { tick_status_round(&mut world, player_id, &mut log); }
        assert_eq!(world.entities[player_id].body.parts[0].damage, 0);
        assert!(world.entities[player_id].body
            .get_status_effect(&StatusEffect::Regenerating(vec![])).is_some(),
            "regeneration should persist for its full duration");

        // Seven more rounds exhaust the 10-turn duration; the status then clears.
        for _ in 0..7 { tick_status_round(&mut world, player_id, &mut log); }
        assert!(world.entities[player_id].body
            .get_status_effect(&StatusEffect::Regenerating(vec![])).is_none(),
            "regeneration should expire after 10 turns");
    }

    #[test]
    fn elixir_regenerates_all_body_parts() {
        let mut world = World::new_test();
        let _ = world.create_player(Point { x: 50, y: 50 }, Direction::Up, String::from("Player"));
        let player_id = world.player_id.unwrap();

        let nparts = world.entities[player_id].body.parts.len();
        for i in 0..nparts { world.entities[player_id].body.parts[i].damage = 2; }

        let mut log = GameLog { entries: vec![] };
        world.resolve_effects(
            &vec![Effect::ApplyRegeneration { entity_id: player_id, bodypart_index: None, turns: 10 }],
            &mut log);

        for _ in 0..2 { tick_status_round(&mut world, player_id, &mut log); }
        for i in 0..nparts {
            assert_eq!(world.entities[player_id].body.parts[i].damage, 0, "part {} not healed", i);
        }
    }

    #[test]
    fn stimpack_restores_energy_clamped_to_max() {
        let mut world = World::new_test();
        let _ = world.create_player(Point { x: 50, y: 50 }, Direction::Up, String::from("Player"));
        let player_id = world.player_id.unwrap();

        let max = world.entities[player_id].body.max_energy;
        world.entities[player_id].body.energy = max.saturating_sub(20);

        let mut log = GameLog { entries: vec![] };
        // Restore 50 into a 20-point deficit → clamps at max, no overflow.
        world.resolve_effects(
            &vec![Effect::RestoreEnergy { entity_id: player_id, amount: 50 }],
            &mut log);

        assert_eq!(world.entities[player_id].body.energy, max);
    }

    /// Index of the body part whose slots include `slot`.
    fn part_holding_slot(world: &World, entity_id: usize, slot: SlotType) -> usize {
        let slot_idx = world.entities[entity_id].body.item_slots.iter()
            .position(|s| s.slot_type == slot).unwrap();
        world.entities[entity_id].body.parts.iter()
            .position(|p| p.slot_index.contains(&slot_idx)).unwrap()
    }

    #[test]
    fn bodypart_damage_clamped_to_twice_max() {
        let mut world = World::new_test();
        let _ = world.create_player(Point { x: 50, y: 50 }, Direction::Up, String::from("Player"));
        let id = world.player_id.unwrap();
        world.debug_mode = true; // keep the player around even when a vital part is destroyed

        let part_index = 0;
        let max = world.entities[id].body.parts[part_index].max_damage;

        let mut log = GameLog { entries: vec![] };
        world.resolve_effects(
            &vec![Effect::Damage { entity_id: id, bodypart_index: part_index, raw_damage: Damage::new(100_000, 0, 0, 0) }],
            &mut log);

        assert_eq!(world.entities[id].body.parts[part_index].damage, 2 * max);
    }

    #[test]
    fn disabled_arm_drops_equipped_firearm() {
        let mut world = World::new_test();
        let _ = world.create_player(Point { x: 50, y: 50 }, Direction::Up, String::from("Player"));
        let id = world.player_id.unwrap();
        world.debug_mode = true;

        let mut pistol = Item::pistol();
        pistol.id = 1;
        let _ = world.entities[id].body.equip(pistol);

        let part_index = part_holding_slot(&world, id, SlotType::PrimaryHand);
        let mut log = GameLog { entries: vec![] };
        world.resolve_effects(
            &vec![Effect::Damage { entity_id: id, bodypart_index: part_index, raw_damage: Damage::new(100_000, 0, 0, 0) }],
            &mut log);

        assert!(world.entities[id].get_equipped_item_ref(SlotType::PrimaryHand).is_none(),
            "disabled hand should no longer hold the pistol");
        assert!(world.map.items.iter().any(|i| i.as_ref().map_or(false, |it| it.id == 1)),
            "pistol should have been dropped to the ground");
    }

    #[test]
    fn disabled_arm_drops_equipped_melee_weapon() {
        let mut world = World::new_test();
        let _ = world.create_player(Point { x: 50, y: 50 }, Direction::Up, String::from("Player"));
        let id = world.player_id.unwrap();
        world.debug_mode = true;

        // The knife equips into the secondary hand.
        let mut knife = Item::knife();
        knife.id = 3;
        let _ = world.entities[id].body.equip(knife);

        let part_index = part_holding_slot(&world, id, SlotType::SecondaryHand);
        let mut log = GameLog { entries: vec![] };
        world.resolve_effects(
            &vec![Effect::Damage { entity_id: id, bodypart_index: part_index, raw_damage: Damage::new(100_000, 0, 0, 0) }],
            &mut log);

        assert!(world.entities[id].get_equipped_item_ref(SlotType::SecondaryHand).is_none(),
            "disabled hand should no longer hold the knife");
        assert!(world.map.items.iter().any(|i| i.as_ref().map_or(false, |it| it.id == 3)),
            "knife should have been dropped to the ground");
    }

    #[test]
    fn disabled_arm_drops_two_handed_firearm_held_as_proxy() {
        let mut world = World::new_test();
        let _ = world.create_player(Point { x: 50, y: 50 }, Direction::Up, String::from("Player"));
        let id = world.player_id.unwrap();
        world.debug_mode = true;

        // A two-handed rifle: real item in PrimaryHand, proxy in SecondaryHand.
        let mut rifle = Item::bolt_action_rifle();
        rifle.id = 7;
        let _ = world.entities[id].body.equip(rifle);

        // Disabling the arm that only holds the proxy must still drop the whole weapon.
        let part_index = part_holding_slot(&world, id, SlotType::SecondaryHand);
        assert_ne!(part_index, part_holding_slot(&world, id, SlotType::PrimaryHand));

        let mut log = GameLog { entries: vec![] };
        world.resolve_effects(
            &vec![Effect::Damage { entity_id: id, bodypart_index: part_index, raw_damage: Damage::new(100_000, 0, 0, 0) }],
            &mut log);

        assert!(world.entities[id].get_equipped_item_ref(SlotType::PrimaryHand).is_none());
        assert!(world.entities[id].get_equipped_item_ref(SlotType::SecondaryHand).is_none());
        assert!(world.map.items.iter().any(|i| i.as_ref().map_or(false, |it| it.id == 7 && !it.proxy)),
            "the real rifle should be on the ground");
    }

    #[test]
    fn multi_slot_armor_protects_every_covered_part() {
        let mut world = World::new_test();
        let _ = world.create_player(Point { x: 50, y: 50 }, Direction::Up, String::from("Player"));
        let id = world.player_id.unwrap();

        // Riot armor covers torso + both arms; the arm slots receive proxies at equip time.
        let _ = world.entities[id].body.equip(Item::riot_armor());
        world.entities[id].body.update_armor();

        for slot in [SlotType::Bodywear, SlotType::LeftArmwear, SlotType::RightArmwear] {
            let part_index = part_holding_slot(&world, id, slot);
            assert!(world.entities[id].body.parts[part_index].armor.phys_absorption > 0,
                "part holding {:?} should be armored (proxy resolved)", slot.to_string());
        }
        // A part the armor does not cover (legs) stays unarmored.
        let legs = part_holding_slot(&world, id, SlotType::Legwear);
        assert_eq!(world.entities[id].body.parts[legs].armor.phys_absorption, 0,
            "legs are not covered by riot armor");
    }

    #[test]
    fn rocket_boots_teleport_moves_player_and_makes_noise() {
        let mut world = World::new_test();
        let start = Point { x: 50, y: 50 };
        let _ = world.create_player(start, Direction::Up, String::from("Player"));
        let id = world.player_id.unwrap();
        let target = Point { x: 55, y: 52 }; // within range 8, empty tile on the open test map

        // Boots must be equipped so the action can read them (to spend a charge).
        let _ = world.entities[id].body.equip(Item::rocket_boots());

        // Drive the equipped-boots action directly through the intent path.
        world.entities[id].intent = Intent {
            phase: ExecutionPhase::Instant,
            data: IntentData::TargetWithEquipment { slot: SlotType::Footwear, target },
            action: crate::actions::rocket_boots_action,
        };
        let effects = (world.entities[id].intent.action)(&world.entities[id], &world.map, &world.entities);

        let mut log = GameLog { entries: vec![] };
        world.resolve_effects(&effects, &mut log);

        assert_eq!(world.entities[id].position, target, "player should teleport to the target");
        assert!(world.map.pawns[world.map.pos_idx(target)].is_some(), "pawn should occupy the new tile");
        assert!(world.map.pawns[world.map.pos_idx(start)].is_none(), "old tile should be vacated");
        // A loud sound was emitted for the AI to hear.
        assert!(effects.iter().any(|e| matches!(e, Effect::Sound(s) if s.volume >= 20)),
            "rocket boots should make a lot of noise");
        // One charge was spent (3 -> 2).
        let charges = match world.entities[id].get_equipped_item_ref(SlotType::Footwear).map(|i| &i.kind) {
            Some(ItemKind::Powered { charges, .. }) => *charges,
            _ => panic!("boots should still be equipped"),
        };
        assert_eq!(charges, 2, "rocket rush should consume one charge");
    }

    #[test]
    fn forward_goons_blocked_by_contested_center_tile() {
        let mut world = World::new_test();

        let center = Point { x: 10, y: 10 };

        // Place one goon on each cardinal side of the center, all facing inward.
        // On the first tick every one of them will declare an intent to enter (10, 10).
        assert!(create_zombie(&mut world, Point { x: center.x,     y: center.y - 1 }, Direction::Down,  String::from("North"), AI::Forward).is_ok());
        assert!(create_zombie(&mut world, Point { x: center.x,     y: center.y + 1 }, Direction::Up,    String::from("South"), AI::Forward).is_ok());
        assert!(create_zombie(&mut world, Point { x: center.x - 1, y: center.y     }, Direction::Right, String::from("West"), AI::Forward).is_ok());
        assert!(create_zombie(&mut world, Point { x: center.x + 1, y: center.y     }, Direction::Left,  String::from("East"), AI::Forward).is_ok());

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

    // ---------------------------------------------------------------------
    // AI performance benchmark. Ignored by default — run explicitly, release:
    //
    //   cargo test --release -- --ignored --nocapture ai_benchmark
    //
    // Env overrides (all optional):
    //   BENCH_SIZE=16       map size in 32-tile blocks
    //   BENCH_ACTORS=2000   target actor count
    //   BENCH_TICKS=60      timed ticks per config
    //   BENCH_WARMUP=20     untimed warm-up ticks (lets fields build)
    //   BENCH_PARALLEL=1    1 = rayon parallel AI, 0 = serial
    //
    // It times World::resolve_intent_declaration (where all AI pathfinding
    // lives) per tick, for two workloads × {flow fields ON, pure A* OFF}:
    //   - patrol: everyone Unaware, navigating shared ring corners (full-map
    //             fields, thousands of shared readers).
    //   - swarm:  a cluster all Alert to one shared cell (bounded dynamic field).
    // ---------------------------------------------------------------------

    #[derive(Clone, Copy)]
    enum BenchScenario {
        /// Everyone Unaware, navigating shared ring corners (full-map fields).
        Patrol,
        /// A cluster all Alert to one shared cell within the bounded field radius.
        Swarm,
        /// Worst case for A*: agents scattered across the whole map, all sharing
        /// ONE distant static goal (a guard anchor at the far outer-ring corner).
        /// The goal is out of sight (so greedy can't short-circuit) and reached
        /// only by long, obstacle-heavy paths — where A* pays its worst per-agent
        /// cost (partial paths capped at MAX_EXPANSIONS, frequent repaths), while
        /// the (static, full-map) field stays flat O(8). Covers far agents because
        /// static goals get full-map fields, unlike the bounded Swarm case.
        Gauntlet,
    }

    fn bench_env_usize(key: &str, default: usize) -> usize {
        std::env::var(key).ok().and_then(|s| s.parse().ok()).unwrap_or(default)
    }

    /// Scatter patrollers across free tiles until the world holds `target` actors.
    fn bench_fill_patrollers(world: &mut World, target: usize) {
        let (w, h) = (world.map.width as i32, world.map.height as i32);
        let mut placed = 0usize;
        let mut y = 2;
        while y < h - 2 && world.entities.len() < target {
            let mut x = 2;
            while x < w - 2 && world.entities.len() < target {
                if !world.map.blocked(x, y) {
                    let pos = Point { x, y };
                    let route_id = world.map.nearest_patrol_route(pos);
                    let wpi = world.map.nearest_waypoint_index(route_id, pos);
                    if world.create_patrol_actor(pos, Direction::Up, format!("Bench {}", placed), route_id, wpi, CombatTactic::Pursue).is_ok() {
                        placed += 1;
                    }
                }
                x += 3;
            }
            y += 3;
        }
    }

    /// Spawn a cluster of actors around `center`, all Alert and investigating that
    /// same cell — the shared-goal case a dynamic field is meant to serve. Kept
    /// within the dynamic field's bounded radius so the field actually covers them.
    fn bench_fill_swarm(world: &mut World, target: usize, center: Point) {
        const R: i32 = 60; // < DYNAMIC_FIELD_MAX_COST radius (~80 tiles)
        let (w, h) = (world.map.width as i32, world.map.height as i32);
        let mut placed = 0usize;
        let mut y = (center.y - R).max(2);
        while y < (center.y + R).min(h - 2) && world.entities.len() < target {
            let mut x = (center.x - R).max(2);
            while x < (center.x + R).min(w - 2) && world.entities.len() < target {
                if !world.map.blocked(x, y) {
                    if world.create_guard_actor(Point { x, y }, Direction::Up, format!("Swarm {}", placed), CombatTactic::Pursue).is_ok() {
                        placed += 1;
                    }
                }
                x += 2;
            }
            y += 2;
        }
        for e in world.entities.iter_mut() {
            if let AI::Actor(actor) = &mut e.ai {
                actor.alert = AlertLevel::Alert {
                    last_known: center,
                    search: SearchBehavior::MoveToLastKnown,
                };
            }
        }
    }

    /// Scatter guards across the whole map, all sharing one distant `anchor`
    /// (Unaware, so they path toward it). Long out-of-sight obstacle-heavy routes
    /// to a single shared static goal — worst case for A*, full-map field win.
    fn bench_fill_gauntlet(world: &mut World, target: usize, anchor: Point) {
        let (w, h) = (world.map.width as i32, world.map.height as i32);
        let mut placed = 0usize;
        let mut y = 2;
        while y < h - 2 && world.entities.len() < target {
            let mut x = 2;
            while x < w - 2 && world.entities.len() < target {
                if !world.map.blocked(x, y) {
                    if world.create_guard_actor(Point { x, y }, Direction::Up, format!("Gauntlet {}", placed), CombatTactic::Pursue).is_ok() {
                        placed += 1;
                    }
                }
                x += 3;
            }
            y += 3;
        }
        // Point every guard at the same far anchor.
        for e in world.entities.iter_mut() {
            if let AI::Actor(actor) = &mut e.ai {
                if let Profile::Guard { anchor: a, .. } = &mut actor.profile {
                    *a = anchor;
                }
            }
        }
    }

    fn bench_report(label: &str, mut samples: Vec<f64>, actors: usize) {
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = samples.len();
        let avg: f64 = samples.iter().sum::<f64>() / n as f64;
        let pct = |q: f64| samples[((n as f64 * q) as usize).min(n - 1)];
        println!(
            "  {:<20} actors={:>5}  avg={:>8.3}ms  p50={:>8.3}ms  p99={:>8.3}ms  max={:>8.3}ms  ({:>6.2} us/actor)",
            label, actors, avg, pct(0.50), pct(0.99), samples[n - 1], avg * 1000.0 / actors as f64,
        );
    }

    fn bench_run(label: &str, size: usize, target: usize, ticks: usize, warmup: usize, use_fields: bool, parallel: bool, scenario: BenchScenario) {
        let mut world = World::new(size, 1);
        world.parallel_ai = parallel;
        world.map.use_flow_fields = use_fields;

        match scenario {
            BenchScenario::Patrol => bench_fill_patrollers(&mut world, target),
            BenchScenario::Swarm => {
                let center = Point { x: (world.map.width / 4) as i32, y: (world.map.height / 4) as i32 };
                bench_fill_swarm(&mut world, target, center);
            }
            BenchScenario::Gauntlet => {
                // Far outer-ring corner: guaranteed walkable (snapped at map gen)
                // and about as far from map-wide scatter as a goal gets.
                let anchor = world.map.patrol_routes.last()
                    .and_then(|r| r.first().copied())
                    .unwrap_or(Point { x: 2, y: 2 });
                bench_fill_gauntlet(&mut world, target, anchor);
            }
        }
        let actors = world.entities.len();

        let mut log = GameLog { entries: vec![] };
        let run_phases = |world: &mut World, log: &mut GameLog| {
            let mut phase = ExecutionPhase::Idle;
            loop {
                world.resolve_phase(phase, log);
                match phase.next() { Some(next) => phase = next, None => break }
            }
            log.entries.clear();
        };

        for _ in 0..warmup {
            world.resolve_intent_declaration();
            run_phases(&mut world, &mut log);
        }

        let mut samples = Vec::with_capacity(ticks);
        for _ in 0..ticks {
            let t0 = std::time::Instant::now();
            world.resolve_intent_declaration();
            samples.push(t0.elapsed().as_secs_f64() * 1000.0);
            run_phases(&mut world, &mut log);
        }
        bench_report(label, samples, actors);
    }

    #[test]
    #[ignore]
    fn ai_benchmark() {
        let size     = bench_env_usize("BENCH_SIZE", 16);
        let target   = bench_env_usize("BENCH_ACTORS", 2000);
        let ticks    = bench_env_usize("BENCH_TICKS", 60);
        let warmup   = bench_env_usize("BENCH_WARMUP", 20);
        let parallel = bench_env_usize("BENCH_PARALLEL", 1) != 0;

        println!(
            "\n=== AI benchmark: size={} blocks (~{}x{} tiles), ~{} actors, {} ticks (warmup {}), parallel={} ===",
            size, size * 32, size * 32, target, ticks, warmup, parallel,
        );
        println!("Timing World::resolve_intent_declaration per tick:\n");

        bench_run("patrol   fields=ON",  size, target, ticks, warmup, true,  parallel, BenchScenario::Patrol);
        bench_run("patrol   fields=OFF", size, target, ticks, warmup, false, parallel, BenchScenario::Patrol);
        bench_run("swarm    fields=ON",  size, target, ticks, warmup, true,  parallel, BenchScenario::Swarm);
        bench_run("swarm    fields=OFF", size, target, ticks, warmup, false, parallel, BenchScenario::Swarm);
        bench_run("gauntlet fields=ON",  size, target, ticks, warmup, true,  parallel, BenchScenario::Gauntlet);
        bench_run("gauntlet fields=OFF", size, target, ticks, warmup, false, parallel, BenchScenario::Gauntlet);
        println!();
    }
}