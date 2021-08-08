use super::*;
use rltk::Point;

/// The contents of the game world itself.
pub struct World {
    pub player: Option<Entity>,
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
            player: Option::None,
            entities: vec![],
            map: Map::new_map_rooms_and_corridors()
        }
    }

    pub fn create_player(&mut self, pos: Point, facing: Facing, name: String) -> Result<(), GameError> {
        if self.player.is_some() {
            return Err(GameError {
                error: Error::BadPrecondition,
                message: "Tried to create player, but one already exists"
            });
        }

        let player = Entity {
            position: pos,
            renderable: Renderable::new_glyph('8'),
            name: name,
            intent: Intent { action: Action::Idle, target: Point {x: 0, y: 0}},
            facing: facing
        };

        self.player = Some(player);

        Ok(())
    }

    pub fn create_entity(&mut self, pos: Point, facing: Facing, name: String) -> Result<(), GameError> {
        let entity = Entity {
            position: pos,
            renderable: Renderable::new_glyph('5'),
            name: name,
            intent: Intent { action: Action::Idle, target: Point {x: 0, y: 0}},
            facing: facing
        };

        self.entities.push(entity);

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn assert_worldsize(world: World, size: usize) -> World {
        let player_count = if world.player.is_some() { 1 } else { 0 };

        let total_size = player_count + world.entities.len();

        assert_eq!(total_size, size, "Position vector is of incorrect size");
        world
    }

    #[test]
    fn create_player() {
        let mut world = World::new();

        let pos = Point {x: 0, y: 0};
        let facing = Facing {direction: Direction::Up};
        let name = "Player";
        let result = world.create_player(pos, facing, String::from(name));

        assert!(result.is_ok());
        world = assert_worldsize(world, 1);
        let player = world.player.unwrap();
        assert_eq!(player.position, pos);
        assert_eq!(player.name, name);
    }

    #[test]
    fn create_entity() {
        let mut world = World::new();

        let pos = Point {x: 0, y: 0};
        let facing = Facing {direction: Direction::Up};
        let name = "Entity";
        let result = world.create_entity(pos, facing, String::from(name));

        assert!(result.is_ok());
        world = assert_worldsize(world, 1);
        assert_eq!(world.entities[0].position, pos);
        assert_eq!(world.entities[0].name, name);
    }
}