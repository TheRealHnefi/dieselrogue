use super::*;
use rltk::Point;

/// The contents of the game world itself.
pub struct World {
    pub player_id: Option<usize>,
    pub entities: Vec<Entity>,
    pub map: Map,
    item_count: usize
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
            item_count: 0,
            map: Map::new_game_map(size)
        };

        let pos = Point {x: (world.map.width / 2) as i32, y: (world.map.height / 2) as i32};
        let _result = world.create_player(pos,
            Direction::Up,
            String::from("Player"));

        let _result = world.create_tank(Point {x: pos.x, y: pos.y - 4},
            Direction::Up,
            String::from("Tank"));

        let _ = world.create_grenade(pos);
        let _ = world.create_machinegun(Point{x: pos.x + 1, y: pos.y});
        let _ = world.create_pistol(Point{x: pos.x + 2, y: pos.y});
        let _ = world.create_rocket_launcher(Point{x: pos.x + 3, y: pos.y});

        return world;
    }

    /// Create new world for performance testing.
    pub fn new_performance_test() -> Self {
        let mut world = World {
            player_id: Option::None,
            entities: vec![],
            item_count: 0,
            map: Map::new_game_map(10)
        };

        let pos = Point {x: 0, y: 0};
        let _ = world.create_player(pos,
            Direction::Up,
            String::from("Player"));

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
            item_count: 0,
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
        if self.map.blocked(pos.x, pos.y) {
            return Err(GameError {
                error: Error::BadPrecondition,
                message: format!("Tried to create player at {},{}, but position is occupied", pos.x, pos.y)
            });
        }

        let mut player = Entity::new_human(0, pos, facing, name);
        player.player = true;

        player.create_pawns(&mut self.map);
        self.entities.push(player);
        self.player_id = Some(0);

        Ok(())
    }

    pub fn create_zombie_goon(&mut self, pos: Point, facing: Direction, name: String) -> Result<(), GameError> {
        if self.map.blocked(pos.x, pos.y) {
            return Err(GameError {
                error: Error::BadPrecondition,
                message: format!("Tried to create entity at {},{}, but position is occupied", pos.x, pos.y)
            });
        }

        let mut entity = Entity::new_human(self.entities.len(), pos, facing, name);
        entity.ai = AI::Rotator;
        entity.create_pawns(&mut self.map);
        self.entities.push(entity);

        Ok(())
    }

    pub fn create_patrolling_goon(&mut self, pos: Point, facing: Direction, name: String, room_indices: Vec<usize>) -> Result<(), GameError> {
        if self.map.blocked(pos.x, pos.y) {
            return Err(GameError {
                error: Error::BadPrecondition,
                message: format!("Tried to create entity at {},{}, but position is occupied", pos.x, pos.y)
            });
        }

        let mut waypoints = vec!();
        for room_index in room_indices {
            let (x, y) = self.map.rooms[room_index].center();
            waypoints.push(Point {x: x, y: y});
        }

        let entity = Entity::new_patrolling_goon(self.entities.len(), pos, facing, name, waypoints);
        entity.create_pawns(&mut self.map);
        self.entities.push(entity);

        Ok(())
    }

    pub fn create_tank(&mut self, pos: Point, facing: Direction, name: String) -> Result<(), GameError> {
        for x in 0..3 {
            for y in 0..3 {
                if self.map.blocked(pos.x + x, pos.y + y) {
                    return Err(GameError {
                        error: Error::BadPrecondition,
                        message: format!("Tried to create entity at {},{}, but position is occupied", pos.x, pos.y)
                    });     
                }
            }
        }

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

    pub fn get_player_mut(&mut self) -> Result<&mut Entity, GameError> {
        match self.player_id {
            Some(id) => return Ok(&mut self.entities[id]),
            None => return Err(GameError {
                error: Error::BadPrecondition,
                message: format!("No player exists")
            })
        }
    }

    pub fn create_grenade(&mut self, pos: Point) -> Result<(), GameError> {
        let actual_pos = self.map.nearest_free_item_position(pos)?;
        let index = self.map.xy_idx(actual_pos.x, actual_pos.y);
        self.map.items[index] = Some(Item::grenade(self.item_count));
        self.item_count += 1;
        Ok(())
    }

    pub fn create_pistol(&mut self, pos: Point) -> Result<(), GameError> {
        let actual_pos = self.map.nearest_free_item_position(pos)?;
        let index = self.map.xy_idx(actual_pos.x, actual_pos.y);
        self.map.items[index] = Some(Item::pistol(self.item_count));
        self.item_count += 1;
        Ok(())
    }

    pub fn create_machinegun(&mut self, pos: Point) -> Result<(), GameError> {
        let actual_pos = self.map.nearest_free_item_position(pos)?;
        let index = self.map.xy_idx(actual_pos.x, actual_pos.y);
        self.map.items[index] = Some(Item::machinegun(self.item_count));
        self.item_count += 1;
        Ok(())
    }

    pub fn create_rocket_launcher(&mut self, pos: Point) -> Result<(), GameError> {
        let actual_pos = self.map.nearest_free_item_position(pos)?;
        let index = self.map.xy_idx(actual_pos.x, actual_pos.y);
        self.map.items[index] = Some(Item::rocket_launcher(self.item_count));
        self.item_count += 1;
        Ok(())
    }

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

    pub fn resolve_phase(&mut self, phase: IntentPhase, log: &mut GameLog) -> Vec<Animation> {
        let mut effects: Vec<Effect> = vec!();
        for entity in self.entities.iter_mut() {
            if entity.intent.phase == phase {
                let mut entity_effects = (entity.intent.action)(entity, &mut self.map, log);
                effects.append(&mut entity_effects);
                entity.intent = idle_intent();
            }
        }

        return self.resolve_effects(&effects, log);
    }

    fn resolve_effects(&mut self, effects: &Vec<Effect>, log: &mut GameLog) -> Vec<Animation> {
        let mut animations = vec!();
        let mut deathlist: Vec<usize> = vec!();
        for effect in effects.iter() {
            match effect {
                Effect::Damage{entity_id: id, bodypart_index: part_index, raw_damage: damage} => {
                    self.entities[*id].apply_damage(*part_index, *damage);
                    if self.entities[*id].mortally_wounded() {
                        log.log(format!("{} was killed!", self.entities[*id].name));
                        deathlist.push(*id);
                    }
                },
                Effect::OpenDoor(pos) => {
                    let index = self.map.pos_idx(*pos);
                    if self.map.tiles[index] == TileType::ClosedDoor {
                        self.map.tiles[index] = TileType::OpenDoor;
                        self.update_views_near_event(*pos, 10);
                    }
                },
                Effect::DestroyWall(pos) => {
                    let index = self.map.pos_idx(*pos);
                    match self.map.tiles[index] {
                        TileType::ClosedDoor => {
                            self.map.tiles[index] = TileType::Floor;
                            self.update_views_near_event(*pos, 10);
                        },
                        TileType::Wall => {
                            self.map.tiles[index] = TileType::Floor;
                            self.update_views_near_event(*pos, 10);
                        },
                        _ => ()
                    }
                },
                Effect::Embark{pilot_id, vehicle_id} => {
                    self.entities[*pilot_id].driving = DrivingState::Driving(*vehicle_id);
                    self.entities[*pilot_id].clear_pawns(&mut self.map);
                    self.entities[*vehicle_id].driving = DrivingState::DrivenBy(*pilot_id);
                    
                    log.log(format!("{} entered {}",
                        self.entities[*pilot_id].name,
                        self.entities[*vehicle_id].name));

                    if self.entities[*pilot_id].id == self.player_id.unwrap() {
                        self.entities[*pilot_id].set_visible_tiles(&mut self.map, false);
                        self.entities[*vehicle_id].set_visible_tiles(&mut self.map, true);
                        self.entities[self.player_id.unwrap()].player = false;
                        self.player_id = Some(*vehicle_id);
                        self.entities[*vehicle_id].player = true;
                    }
                },
                Effect::Disembark{pilot_id, vehicle_id} => {
                    let vehicle_center = self.entities[*vehicle_id].center();
                    match self.map.nearest_free_pawn_position(vehicle_center) {
                        Ok(pos) => {
                            self.entities[*pilot_id].driving = DrivingState::None;
                            self.entities[*vehicle_id].driving = DrivingState::Drivable;
                            self.entities[*pilot_id].position = pos;
                            self.entities[*pilot_id].create_pawns(&mut self.map);
                            self.entities[*vehicle_id].create_pawns(&mut self.map);
                            self.entities[*pilot_id].update_view(&mut self.map);

                            if self.entities[*vehicle_id].id == self.player_id.unwrap() {
                                self.entities[*vehicle_id].set_visible_tiles(&mut self.map, false);
                                self.entities[*pilot_id].set_visible_tiles(&mut self.map, true);

                                self.player_id = Some(*pilot_id);
                                self.entities[*vehicle_id].player = false;
                                self.entities[*pilot_id].player = true;
                            }

                            log.log(format!("{} left their vehicle",
                                self.entities[*pilot_id].name));
                        },
                        Err(_) => {
                            log.log(format!("{} tried to disembark, but there is no room",
                                self.entities[*pilot_id].name));
                        }
                    }
                },
                Effect::Animation(animation) => {
                    animations.push(animation.clone());
                },
                Effect::ApplyStatus{target_id, status} => {
                    self.entities[*target_id].apply_status_effect(status);
                }
            }
        }

        self.post_resolve(deathlist);
        return animations;
    }

    pub fn resolve_status_effects(&mut self) {
        for entity in &mut self.entities {
            entity.resolve_status_effects();
        }
    }

    fn post_resolve(&mut self, deathlist: Vec<usize>) {
        for id in &deathlist {
            self.entities[*id].kill(&mut self.map);
        }
        
        self.entities.retain(|entity| {
            let should_be_dead = deathlist.iter().any(|&id| id == entity.id);
            return !should_be_dead;
        });

        for (i, entity) in self.entities.iter_mut().enumerate() {
            entity.id = i;
        }
    }

    fn update_views_near_event(&mut self, position: Point, radius: i32) {
        let entity_ids = self.map.get_entities_in_vicinity(position, radius);
        for id in entity_ids {
            self.entities[id].update_view(&mut self.map);
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
    fn create_two_entities_on_same_pos_fails() {
        let mut world = World::new_test();

        let pos = Point {x: 0, y: 0};
        let facing = Direction::Up;
        let name = "Entity";
        let result1 = world.create_zombie_goon(pos, facing, String::from(name));

        let pos2 = pos;
        let name2 = "Entity2";
        let result2 = world.create_zombie_goon(pos2, facing, String::from(name2));

        assert!(result1.is_ok());
        assert!(result2.is_err());
        world = assert_worldsize(world, 1);
        assert_eq!(world.entities[0].position, pos);
        assert_eq!(world.entities[0].name, name);
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

        let _ = world.create_grenade(pos);

        let index = world.map.xy_idx(pos.x, pos.y);
        assert!(world.map.items[index].is_some());
    }

    #[test]
    fn add_items_on_top_of_eachother_pushes_one_aside() {
        let mut world = World::new_test();
        let pos = Point {x: 1, y: 1};

        let _ = world.create_grenade(pos);
        let _ = world.create_grenade(pos);

        assert!(world.map.items.iter().filter(|i| i.is_some()).count() == 2);

        for (index, item) in world.map.items.iter().enumerate() {
            if item.is_some() {
                assert!(world.map.tiles[index] == TileType::Ground);
            }
        }
    }
}