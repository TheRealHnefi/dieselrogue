use super::*;
use rltk::Point;
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
    next_item_id: usize
}


/*
API scratchpad:
let player = World.create_player(pos, "Player", inv, body)?;
World.set_intent(player, Intent {action: fire, target: pos});
World.set_intent(player, Intent {action: move, target: pos});

let firearm_data = FirearmData {
    damage_phys: 5,
    damage_fire: 0,
    damage_elec: 0,
    range: 10,
    damage_falloff: 0,
    burst: 1,
    clip_size: 10,
    sound: 7,
    hiteffect: {}
}
let _gun = World.create_item(pos, "Pistol", firearm(firearm_data), renderable, description)?;

let _tank = World.create_vehicle(pos, "Panzer", tank(tank_data), renderable, description)?;

let _enemy = World.create_actor(pos, "Goon #32", inv, body, renderable, description, ai)?;

World.run_ai(enemy);

pub fn run_ai(&mut self, Entity: enemy) -> Result<(), GameError> {
    for actor in actors {
        if self.map[x][y] == player {
            Intents[actor.index] = Intent {action: melee, target: Pos {x: x, y: y}};
        }
    }
}

World.resolve_melee();

pub fn resolve_melee(&mut self, Entity: entity) -> Result<(), GameError> {
  for each living and meleeing entity, create damage.
  for each damage instance, apply damage effect. Set deadflags and such as appropriate.
}

World.cleanup(); // Delete dead entries
*/

impl World {
    /// Create new world.
    /// # Arguments
    /// * `size` - Number of blocks that make up one size of the map.
    pub fn new(size: usize) -> Self {
        let mut world = World {
            player_id: Option::None,
            entities: vec![],
            next_item_id: 0,
            pending_levelup: false,
            sounds: vec![],
            sounds_last_turn: vec![],
            active_items: vec![],
            active_items_ticked: false,
            map: Map::new_game_map(size)
        };

        let pos = Point {x: (world.map.width / 2) as i32, y: (world.map.height / 2) as i32};
        let _result = world.create_player(pos,
            Direction::Up,
            String::from("Player"));

        world.init_static_entities();

        let door_ids: Vec<usize> = world.entities.iter()
            .filter(|e| e.kind == EntityKind::Door)
            .map(|e| e.id)
            .collect();
        let key_door_ids: Vec<usize> = door_ids.into_iter().step_by(2).collect();
        let _ = world.add_item(pos, Item::key(key_door_ids));

        let _result = world.create_tank(Point {x: pos.x, y: pos.y - 4},
            Direction::Up,
            String::from("Tank"));

        let _ = world.add_item(pos, Item::knife());
        let _ = world.add_item(pos, Item::machinegun());
        let _ = world.add_item(pos, Item::pistol());
        let _ = world.add_item(pos, Item::pistol());
        let _ = world.add_item(pos, Item::rocket_launcher());
        let _ = world.add_item(pos, Item::bulletproof_vest());
        let _ = world.add_item(pos, Item::flamethrower());
        let _ = world.add_item(pos, Item::grenade());
        let _ = world.add_item(pos, Item::flashbang());
        let _ = world.add_item(pos, Item::fire_grenade());
        let _ = world.add_item(pos, Item::shock_grenade());
        let _ = world.add_item(pos, Item::shock_pistol());

        // Enemies spread in front of the player to test fan fire arc
        let _ = world.create_zombie_goon(Point {x: pos.x,     y: pos.y - 3}, Direction::Down, String::from("Goon A"));
        let _ = world.create_zombie_goon(Point {x: pos.x + 2, y: pos.y - 3}, Direction::Down, String::from("Goon B"));
        let _ = world.create_zombie_goon(Point {x: pos.x - 2, y: pos.y - 3}, Direction::Down, String::from("Goon C"));
        let _ = world.create_zombie_goon(Point {x: pos.x + 5, y: pos.y - 2}, Direction::Down, String::from("Goon D"));

        assert!(world.create_forward_goon(Point { x: pos.x, y: pos.y + 2 }, Direction::Left,  String::from("Walker")).is_ok());

        // Two goons patrolling north-south along the road, north of the player.
        // TODO: This is often slow because of badly coded AI. Fix later.
        // let ns_road_x = pos.x - 1;
        // let ns_north = Point { x: ns_road_x, y: pos.y - 70 };
        // let ns_south = Point { x: ns_road_x, y: pos.y - 40 };
        // let _ = world.create_patrolling_goon(Point { x: ns_road_x, y: pos.y - 15 }, Direction::Up,   String::from("Patrol NS-1"), vec![ns_north, ns_south]);
        // let _ = world.create_patrolling_goon(Point { x: ns_road_x, y: pos.y - 14 }, Direction::Up,   String::from("Patrol NS-2"), vec![ns_north, ns_south]);

        // One goon patrolling east-west along the road, a few blocks west of the player.
        let ew_road_y = pos.y - 1;
        let ew_west  = Point { x: pos.x - 60, y: ew_road_y };
        let ew_east  = Point { x: pos.x - 30, y: ew_road_y };
        let _ = world.create_patrolling_goon(Point { x: pos.x - 30, y: ew_road_y }, Direction::Left, String::from("Patrol EW-1"), vec![ew_west, ew_east]);

        let center = Point { x: pos.x + 10, y: pos.y + 10 };

        // Place one goon on each cardinal side of the center, all facing inward.
        assert!(world.create_forward_goon(Point { x: center.x,     y: center.y - 1 }, Direction::Down,  String::from("North")).is_ok());
        assert!(world.create_forward_goon(Point { x: center.x,     y: center.y + 1 }, Direction::Up,    String::from("South")).is_ok());
        assert!(world.create_forward_goon(Point { x: center.x - 1, y: center.y     }, Direction::Right, String::from("West")).is_ok());
        assert!(world.create_forward_goon(Point { x: center.x + 1, y: center.y     }, Direction::Left,  String::from("East")).is_ok());

        return world;
    }

    /// Create new world for performance testing.
    pub fn new_performance_test() -> Self {
        let mut world = World {
            player_id: Option::None,
            entities: vec![],
            next_item_id: 0,
            pending_levelup: false,
            sounds: vec![],
            sounds_last_turn: vec![],
            active_items: vec![],
            active_items_ticked: false,
            map: Map::new_game_map(10)
        };

        let pos = Point {x: 0, y: 0};
        let _ = world.create_player(pos,
            Direction::Up,
            String::from("Player"));

        world.init_static_entities();

        // As of 28/12/2021, 1000 rotating zombies has almost acceptable performance in release mode, but more optimiziation
        // would be good. Typical tick duration is ~88 ms. Would like to get it down to ~20 ms.
        // Latent zombies are almost free (can have upwards 100.000 with acceptable performance). Likely pawn creation
        // that is the issue.
        for x in 0..100 {
            for y in 1..10 {
                let _ = world.create_zombie_goon(Point {x: pos.x + x, y: pos.y+y}, Direction::Up, String::from("Zombie"));
            }
        }

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
            map: Map::new_empty_map(100, 100)
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
        player.body.update_abilities();

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
        self.equip_pistol(&mut entity);
        entity.create_pawns(&mut self.map);
        self.entities.push(entity);

        Ok(())
    }

    pub fn create_forward_goon(&mut self, pos: Point, facing: Direction, name: String) -> Result<(), GameError> {
        let actual_pos = self.map.nearest_free_pawn_position(pos)?;
        let mut entity = Entity::new_human(self.entities.len(), actual_pos, facing, name);
        entity.ai = AI::Forward;
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

    fn equip_pistol(&mut self, entity: &mut Entity) {
        let mut pistol = Item::pistol();
        pistol.id = self.next_item_id;
        self.next_item_id += 1;
        let _ = entity.body.equip(pistol);
        entity.body.update_armor();
    }

    pub fn create_tank(&mut self, pos: Point, facing: Direction, name: String) -> Result<(), GameError> {
        let pos = self.map.nearest_free_pawn_position_sized(pos, 3, 3)?;

        let tank = Entity::new_tank(self.entities.len(), pos, facing, name);
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
        for i in 0..self.entities.len() {
            match self.entities[i].driving {
                DrivingState::Driving(_vehicle_id) => (),
                DrivingState::DrivenBy(pilot_id) => {
                    // TODO: This could be made simpler if I split at the higher ID instead...
                    if i < pilot_id {
                        let split_index = i + 1;
                        let (e1, e2) = self.entities.split_at_mut(split_index);
                        let pilot_ai = &mut e2[pilot_id - split_index].ai;
                        e1[i].declare_intent_by_pilot(&self.map, pilot_ai);
                    } else if i > pilot_id {
                        let split_index = pilot_id + 1;
                        let (e1, e2) = self.entities.split_at_mut(split_index);
                        let pilot_ai = &mut e1[pilot_id].ai;
                        e2[i - split_index].declare_intent_by_pilot(&self.map, pilot_ai);
                    } else {
                        assert!(false);
                    }
                },
                DrivingState::Drivable => {
                    self.entities[i].declare_intent(&self.map);
                },
                DrivingState::None => {
                    self.entities[i].declare_intent(&self.map);
                }
            }
        }
    }

    pub fn resolve_phase(&mut self, phase: ExecutionPhase, log: &mut GameLog) -> Vec<Animation> {
        if phase == ExecutionPhase::Movement {
            self.cancel_contested_moves();
        }

        if phase == ExecutionPhase::ActiveItems {
            return self.resolve_active_items(log);
        }

        let mut effects: Vec<Effect> = vec!();
        for entity in self.entities.iter_mut() {
            if entity.intent.phase == phase {
                if entity.body.get_status_effect(&StatusEffect::Shocked(0)).is_none() {
                    let mut entity_effects = (entity.intent.action)(entity, &mut self.map, log);
                    effects.append(&mut entity_effects);
                }
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
            flash: bool,
        }

        let mut ticks: Vec<Tick> = vec!();
        for active in &self.active_items {
            let found = match &active.location {
                ItemLocation::OnMap(pos) => {
                    let idx = self.map.pos_idx(*pos);
                    self.map.items[idx].as_ref()
                        .filter(|i| i.id == active.item_id)
                        .and_then(|i| if let ItemKind::FusedExplosive { damage, timeout, flash } = i.kind {
                            Some((damage, timeout, flash))
                        } else { None })
                        .map(|(d, t, f)| (active.location.clone(), d, t, f))
                },
                ItemLocation::InInventory(eid) => {
                    self.entities.get(*eid)
                        .and_then(|e| e.body.inventory.iter().find(|i| i.id == active.item_id))
                        .and_then(|i| if let ItemKind::FusedExplosive { damage, timeout, flash } = i.kind {
                            Some((damage, timeout, flash))
                        } else { None })
                        .map(|(d, t, f)| (active.location.clone(), d, t, f))
                },
            };
            if let Some((location, damage, timeout, flash)) = found {
                ticks.push(Tick { item_id: active.item_id, location, damage, timeout, flash });
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
                if tick.flash {
                    log.log(String::from("A flashbang goes off!"));
                    const FLASH_RADIUS: i32 = 5;
                    for entity in &self.entities {
                        let dx = entity.position.x - pos.x;
                        let dy = entity.position.y - pos.y;
                        if dx * dx + dy * dy <= FLASH_RADIUS * FLASH_RADIUS {
                            effects.push(Effect::ApplyStatus {
                                target_id: entity.id,
                                status: StatusEffect::Blind(5),
                            });
                        }
                    }
                } else {
                    log.log(String::from("A grenade explodes!"));
                    const RADIUS: i32 = 3;
                    for entity in &self.entities {
                        let dx = entity.position.x - pos.x;
                        let dy = entity.position.y - pos.y;
                        if dx * dx + dy * dy <= RADIUS * RADIUS {
                            for part in 0..entity.body.parts.len() {
                                effects.push(Effect::Damage {
                                    entity_id: entity.id,
                                    bodypart_index: part,
                                    raw_damage: tick.damage,
                                });
                            }
                        }
                    }
                }
                effects.push(Effect::Animation(explosion_animation(pos)));
                effects.push(Effect::Sound(SoundEvent { kind: SoundKind::Explosion, pos, volume: 25 }));
                self.remove_item_from_location(tick.item_id, &tick.location);
                exploded.push(tick.item_id);
            } else {
                self.set_fuse_timeout(tick.item_id, &tick.location, new_timeout);
            }
        }

        self.active_items.retain(|a| !exploded.contains(&a.item_id));

        self.resolve_effects(&effects, log)
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
        if self.entities[id].mortally_wounded() && !deathlist.contains(&id) {
            log.log(format!("{} was killed!", self.entities[id].name));
            deathlist.push(id);
        }
    }

    fn handle_open_door(&mut self, pos: Point, actor_id: usize, log: &mut GameLog) {
        let index = self.map.pos_idx(pos);
        let entity_id = match &self.map.pawns[index] {
            Some(pawn) if pawn.kind == EntityKind::Door => pawn.entity_id,
            _ => return,
        };
        if self.entities[entity_id].locked {
            let has_key = self.entities[actor_id].body.inventory.iter().any(|item| {
                matches!(&item.kind, ItemKind::Key { door_ids } if door_ids.contains(&entity_id))
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

    pub fn resolve_status_effects(&mut self, log: &mut GameLog) {
        self.apply_noise_deafness();
        self.active_items_ticked = false;
        let mut effects: Vec<Effect> = vec![];
        for entity in &mut self.entities {
            effects.extend(entity.resolve_status_effects());
        }
        self.resolve_effects(&effects, log);
        for i in 0..self.entities.len() {
            self.entities[i].update_view(&mut self.map);
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
        if let Some(old_id) = self.player_id {
            self.player_id = self.entities.iter().position(|e| {
                matches!(e.kind, crate::entity::EntityKind::Player)
            }).or(Some(old_id));
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