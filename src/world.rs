use super::*;
use rltk::Point;

/// The contents of the game world itself.
pub struct World {
    entities: Vec<Entity>,

    /// All data that can be part of entities, stored as contiguous arrays.
    /// This is obviously not optimal, but simple and faster than storing all information in the objects themselves
    positions: Vec<Point>,
    renderables: Vec<Renderable>,

    pub map: Map
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: vec![],
            positions: vec![],
            renderables: vec![],
            map: Map::new_map_rooms_and_corridors()
        }
    }

    pub fn create_player(&mut self) -> Entity {
        let entity = Entity {
            index: self.entities.len(),
            entity_type: EntityType::Player(Player {})
        };

        self.entities.push(entity);

        self.positions.push(Point {x: 0, y: 0});
        self.renderables.push(Renderable::new_glyph('8'));

        entity
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Entity {
        self.entities.swap_remove(entity.index);
        self.entities[entity.index].index = entity.index;

        self.positions.swap_remove(entity.index);
        self.renderables.swap_remove(entity.index);

        entity
    }
}