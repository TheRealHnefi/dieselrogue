use super::*;
use rltk::Point;

/// The contents of the game world itself.
pub struct World {
    player: Option<Player>,

    // For ease of bookkeeping, keep explicit track of the number of extant entities
    extant_entities: usize,

    /// All data that can be part of entities, stored as contiguous arrays.
    /// This is obviously not optimal, but simple and faster than storing all information in the objects themselves
    positions: Vec<Point>,
    renderables: Vec<Renderable>,
    names: Vec<String>,
    intents: Vec<Intent>,

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
            extant_entities: 0,
            positions: vec![],
            renderables: vec![],
            names: vec![],
            intents: vec![],
            map: Map::new_map_rooms_and_corridors()
        }
    }

    pub fn create_player(&mut self, pos: Point, facing: Facing, name: String) -> Result<(), GameError> {
        self.player = Some(Player {
            index: self.extant_entities,
            facing: facing
        });

        self.positions.push(pos);
        self.renderables.push(Renderable::new_glyph('8'));
        self.names.push(name);
        self.intents.push(Intent { action: Action::Idle, target: Point {x: 0, y: 0}});

        self.extant_entities += 1;
        Ok(())
    }


}


#[cfg(test)]
mod tests {
    use super::*;

    fn assert_worldsize(world: World, size: usize) -> World {
        assert_eq!(world.positions.len(), size, "Position vector is of incorrect size");
        assert_eq!(world.renderables.len(), size, "Renderable vector is of incorrect size");
        assert_eq!(world.names.len(), size, "Names vector is of incorrect size");
        assert_eq!(world.intents.len(), size, "Intents vector is of incorrect size");
        world
    }

    #[test]
    fn test_create_player() {
        let mut world = World::new();

        let pos = Point {x: 0, y: 0};
        let facing = Facing {direction: Direction::Up};
        let name = "Player";
        let result = world.create_player(pos, facing, String::from(name));

        assert!(result.is_ok());
        world = assert_worldsize(world, 1);
        assert_eq!(world.positions[0], pos);
        assert_eq!(world.names[0], name);
    }
}