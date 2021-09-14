use super::*;
use rltk::Point;

/// The contents of the game world itself.
pub struct World {
    pub player_id: Option<usize>,
    pub entities: Vec<Entity>,

    // TODO: Copy relevant data to map when adding/moving actors. Might be faster, since moving
    // is relatively uncommon compared to rendering/dereferencing.
    pub map: Map
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
    pub fn new() -> Self {
        Self {
            player_id: Option::None,
            entities: vec![],
            map: Map::new_map_rooms_and_corridors()
        }
    }

    pub fn create_player(&mut self, pos: Point, facing: Facing, name: String) -> Result<(), GameError> {
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

        let player = Entity {
            id: 0,
            position: pos,
            renderable: Renderable::new_glyph('8'),
            name: name,
            intent: Intent { action: Action::Idle},
            facing: facing,
            inventory: vec!()
        };

        let index = self.map.xy_idx(pos.x, pos.y);
        self.map.pawns[index] = Some(player.create_pawn());
        self.entities.push(player);
        self.player_id = Some(0);

        Ok(())
    }

    pub fn create_entity(&mut self, pos: Point, facing: Facing, name: String) -> Result<(), GameError> {
        if self.map.blocked(pos.x, pos.y) {
            return Err(GameError {
                error: Error::BadPrecondition,
                message: format!("Tried to create entity at {},{}, but position is occupied", pos.x, pos.y)
            });
        }

        let entity = Entity {
            id: self.entities.len(),
            position: pos,
            renderable: Renderable::new_glyph('5'),
            name: name,
            intent: Intent { action: Action::Idle},
            facing: facing,
            inventory: vec!()
        };

        let index = self.map.xy_idx(pos.x, pos.y);
        self.map.pawns[index] = Some(entity.create_pawn());
        self.entities.push(entity);

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

    pub fn add_item(&mut self, pos: Point, item: Item) -> Result<(), GameError> {
        let actual_pos = self.map.nearest_free_item_position(pos)?;
        let index = self.map.xy_idx(actual_pos.x, actual_pos.y);
        self.map.items[index] = Some(item);
        Ok(())
    }

    pub fn resolve_movement(&mut self) -> Result<(), GameError> {

        let mut effects: Vec<Effect> = vec!();
        for entity in self.entities.iter_mut() {
            match entity.resolve_movement(&mut self.map) {
                Some(effect) => effects.push(effect),
                None => ()
            }
        }

        self.resolve_effects(&effects);

        Ok(())
    }

    pub fn resolve_melee(&mut self) -> Result<(), GameError> {

        let mut effects: Vec<Effect> = vec!();

        for entity in self.entities.iter_mut() {
            match entity.resolve_melee(&mut self.map) {
                Some(effect) => effects.push(effect),
                None => ()
            }
        }

        self.resolve_effects(&effects);

        Ok(())
    }

    pub fn resolve_throw(&mut self) -> Result<(), GameError> {

        let mut effects: Vec<Effect> = vec!();

        for entity in self.entities.iter_mut() {
            match entity.resolve_throw(&mut self.map) {
                Some(effect) => effects.push(effect),
                None => ()
            }
        }

        self.resolve_effects(&effects);

        Ok(())
    }

    pub fn resolve_inventory(&mut self) -> Result<(), GameError> {
        let mut effects: Vec<Effect> = vec!();

        for entity in self.entities.iter_mut() {
            match entity.resolve_inventory(&mut self.map) {
                Some(effect) => effects.push(effect),
                None => ()
            }
        }

        self.resolve_effects(&effects);

        Ok(())
    }

    fn resolve_effects(&mut self, effects: &Vec<Effect>) {
        let mut deathlist: Vec<usize> = vec!();
        for effect in effects.iter() {
            match effect {
                Effect::Damage(id) => {
                    deathlist.push(*id);
                }
            }
        }

        self.post_resolve(deathlist);
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
        let mut world = World::new();

        let pos = Point {x: world.map.rooms[0].x1+1, y: world.map.rooms[0].y1+1};
        let facing = Facing {direction: Direction::Up};
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
        let mut world = World::new();

        let pos = Point {x: world.map.rooms[0].x1+1, y: world.map.rooms[0].y1+1};
        let facing = Facing {direction: Direction::Up};
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
        let mut world = World::new();

        let pos = Point {x: world.map.rooms[0].x1+1, y: world.map.rooms[0].y1+1};
        let facing = Facing {direction: Direction::Up};
        let name = "Entity";
        let result = world.create_entity(pos, facing, String::from(name));

        assert!(result.is_ok());
        world = assert_worldsize(world, 1);
        assert_eq!(world.entities[0].position, pos);
        assert_eq!(world.entities[0].name, name);
    }

    #[test]
    fn create_two_entities() {
        let mut world = World::new();

        let pos = Point {x: world.map.rooms[0].x1+1, y: world.map.rooms[0].y1+1};
        let facing = Facing {direction: Direction::Up};
        let name = "Entity";
        let result1 = world.create_entity(pos, facing, String::from(name));

        let pos2 = Point {x: pos.x + 1, y: pos.y + 1};
        let name2 = "Entity2";
        let result2 = world.create_entity(pos2, facing, String::from(name2));

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
        let mut world = World::new();

        let pos = Point {x: world.map.rooms[0].x1+1, y: world.map.rooms[0].y1+1};
        let facing = Facing {direction: Direction::Up};
        let name = "Entity";
        let result1 = world.create_entity(pos, facing, String::from(name));

        let pos2 = pos;
        let name2 = "Entity2";
        let result2 = world.create_entity(pos2, facing, String::from(name2));

        assert!(result1.is_ok());
        assert!(result2.is_err());
        world = assert_worldsize(world, 1);
        assert_eq!(world.entities[0].position, pos);
        assert_eq!(world.entities[0].name, name);
    }

    #[test]
    fn deathlisted_entities_die_others_reordered() {
        let number_of_entities:usize = 5;
        let mut world = World::new();

        // Create a bunch of entities, named after their id
        let pos = Point {x: world.map.rooms[0].x1+1, y: world.map.rooms[0].y1+1};
        let facing = Facing {direction: Direction::Up};
        for i in 0..number_of_entities {
            assert!(world.create_entity(Point{x: pos.x+i as i32, y: pos.y}, facing, format!("{}", i)).is_ok());
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
        let mut world = World::new();
        let pos = Point {x: world.map.rooms[0].x1+1, y: world.map.rooms[0].y1+1};

        let item = Item::grenade();
        let _ = world.add_item(pos, item);

        let index = world.map.xy_idx(pos.x, pos.y);
        assert!(world.map.items[index].is_some());
    }

    #[test]
    fn add_items_on_top_of_eachother_pushes_one_aside() {
        let mut world = World::new();
        let pos = Point {x: world.map.rooms[0].x1+1, y: world.map.rooms[0].y1+1};

        let item = Item::grenade();
        let _ = world.add_item(pos, item.clone());
        let _ = world.add_item(pos, item);

        assert!(world.map.items.iter().filter(|i| i.is_some()).count() == 2);

        for (index, item) in world.map.items.iter().enumerate() {
            if item.is_some() {
                assert!(world.map.tiles[index] == TileType::Floor);
            }
        }
    }
}