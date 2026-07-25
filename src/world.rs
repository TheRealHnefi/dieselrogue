use super::*;
use rltk::Point;

/// The contents of the game world itself.
pub struct World {
    entities: Vec<EntityEntry>,

    /// All data that can be part of entities, stored as contiguous arrays.
    /// This is obviously not optimal, but simple and faster than storing all information in the objects themselves
    positions: Vec<Point>,
    renderables: Vec<Renderable>,
    names: Vec<String>,
    intents: Vec<Intent>,

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

}
*/

impl World {
    pub fn new() -> Self {
        Self {
            entities: vec![],
            positions: vec![],
            renderables: vec![],
            names: vec![],
            intents: vec![],
            map: Map::new_map_rooms_and_corridors()
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_player() {
        let world = World::new();
    }
}