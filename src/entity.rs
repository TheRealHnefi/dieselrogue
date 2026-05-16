use rltk::Point;
use crate::components::*;
use crate::ai::*;
use crate::intent::*;
use crate::sprite::*;
use crate::viewshed::*;
use crate::Ability;
use crate::Map;
use crate::tile::TileType;
use crate::Item;
use crate::Body;
use crate::actions;

#[derive(PartialEq, Clone)]
pub enum DrivingState {
    None,
    Driving(usize),
    DrivenBy(usize),
    Drivable
}

#[derive(PartialEq, Clone)]
pub enum EntityKind {
    Player,
    Actor,
    Door
}

/// The authoritative record for everything that acts and moves in the world.
///
/// Each Entity owns its full state: position, body, inventory, AI, intent, and viewshed.
/// Entities are stored in `World::entities` and are the only place state should be mutated.
///
/// Because spatial lookups directly into `World::entities` would require a linear scan,
/// Entities project lightweight [`Pawn`] values onto `Map::pawns` for O(1) tile queries.
/// Call [`Entity::create_pawns`] after spawning or moving, and [`Entity::clear_pawns`] before
/// removing an entity. [`Entity::set_position`] handles both automatically.
pub struct Entity {
    pub id: usize,
    pub kind: EntityKind,
    pub driving: DrivingState,
    pub sprite: Sprite,
    pub size_x: u32,
    pub size_y: u32,
    pub position: Point,
    pub name: String,
    pub intent: Intent,
    pub body: Body,
    pub viewshed: Viewshed,
    pub ai: AI
}

impl Entity {
    pub fn new_human(id: usize, pos: Point, facing: Direction, name: String) -> Self {
        Self {
            id: id,
            kind: EntityKind::Actor,
            driving: DrivingState::None,
            sprite: Sprite::Human,
            size_x: 1,
            size_y: 1,
            position: pos,
            name: name,
            intent: idle_intent(),
            body: Body::human_body(facing),
            viewshed: Viewshed::new(20, FieldOfView::Fov180),
            ai: AI::None
        }
    }

    pub fn new_patrolling_goon(id: usize, pos: Point, facing: Direction, name: String, waypoints: Vec<Point>) -> Self {
        Self {
            id: id,
            kind: EntityKind::Actor,
            driving: DrivingState::None,
            sprite: Sprite::Human,
            size_x: 1,
            size_y: 1,
            position: pos,
            name: name,
            intent: idle_intent(),
            body: Body::human_body(facing),
            viewshed: Viewshed::new(20, FieldOfView::Fov180),
            ai: AI::Patrolling(PatrollingAI::new(waypoints))
        }
    }

    pub fn new_tank(id: usize, pos: Point, facing: Direction, name: String) -> Self {
        Self {
            id: id,
            kind: EntityKind::Actor,
            driving: DrivingState::Drivable,
            sprite: Sprite::Tank,
            size_x: 3,
            size_y: 3,
            position: pos,
            name: name,
            intent: idle_intent(),
            body: Body::tank_body(facing),
            viewshed: Viewshed::new(20, FieldOfView::Fov90),
            ai: AI::Rotator
        }
    }

    pub fn new_door(id: usize, pos: Point, direction: Direction, length: u32) -> Self {
        let mut size_x = 1;
        let mut size_y = 1;

        if length > 1 {
            match direction {
                Direction::Up => size_y = length,
                Direction::Down => size_y = length,
                Direction::Left => size_x = length,
                Direction::Right => size_x = length,
                _ => assert!(false, "Illegal door orientation")
            }
        }

        Self {
            id: id,
            kind: EntityKind::Door,
            driving: DrivingState::None,
            sprite: Sprite::Door,
            size_x: size_x,
            size_y: size_y,
            position: pos,
            name: "Door".to_string(),
            intent: idle_intent(),
            body: Body::door_body(direction),
            viewshed: Viewshed::new(0, FieldOfView::Fov360),
            ai: AI::None
        }
    }

    pub fn check_fit(&self, pos: Point, map: &Map) -> bool {
        for x in 0..self.size_x {
            for y in 0..self.size_y {
                let index = map.xy_idx(pos.x + x as i32, pos.y + y as i32);
                match &map.pawns[index] {
                    Some(pawn) => {
                        if pawn.entity_id != self.id {
                            return false;
                        }
                    },
                    None => {
                        match map.tiles[index] {
                            TileType::Wall => return false,
                            TileType::Doorway => (),
                            TileType::Floor => (),
                            TileType::Ground => (),
                            TileType::Road => (),
                        }
                    }
                }
            }
        }

        return true;
    }

    /// Writes a [`Pawn`] snapshot of this entity into every map tile it occupies.
    /// Must be called after spawning or after [`Entity::clear_pawns`] + a position change.
    pub fn create_pawns(&self, map: &mut Map) {
        for x in 0..self.size_x {
            for y in 0..self.size_y {
                let index = map.xy_idx(self.position.x + x as i32, self.position.y + y as i32);
                map.pawns[index] = Some(Pawn {
                    entity_id: self.id,
                    kind: self.kind.clone(),
                    driving: self.driving.clone(),
                    sprite: self.sprite.clone(),
                    sprite_index: x + y * self.size_x,
                    name: self.name.clone(),
                    intent: self.intent.clone(),
                    body: self.body.clone()
                });
            }
        }
    }

    /// Removes this entity's [`Pawn`] entries from every map tile it occupies.
    /// Must be called before the entity is moved or removed from the world.
    pub fn clear_pawns(&self, map: &mut Map) {
        for x in 0..self.size_x {
            for y in 0..self.size_y {
                let index = map.xy_idx(self.position.x + x as i32, self.position.y + y as i32);
                map.pawns[index] = None;
            }
        }
    }

    pub fn set_position(&mut self, pos: Point, map: &mut Map) {
        self.clear_pawns(map);
        self.position = pos;
        self.create_pawns(map);
        self.update_view(map);
    }

    pub fn center(&self) -> Point {
        Point {
            x: self.position.x + self.size_x as i32 / 2,
            y: self.position.y + self.size_y as i32 / 2
        }
    }

    pub fn take_item(&mut self, item: Item) -> Option<Item> {
        if let Some(item_index) = self.body.inventory.iter().position(|value| *value == item) {
            Some(self.body.inventory.swap_remove(item_index))
        }
        else {
            None
        }
    }

    pub fn get_equipped_item(&mut self, slot: SlotType) -> Option<&mut Item> {
        if let Some(item_index) = self.body.item_slots.iter().position(|value| value.slot_type == slot) {
            self.body.item_slots[item_index].item.as_mut()
        }
        else {
            None
        }
    }

    pub fn declare_intent_by_pilot(&mut self, map: &Map, pilot_ai: &mut AI) {
        match pilot_ai {
            AI::Patrolling(ai) => {
                self.intent = ai.declare_intent(self.position, &self.body, map);
            },
            AI::Rotator => {
                self.intent = Intent {
                    phase: ExecutionPhase::Movement,
                    data: IntentData::Direction(self.body.facing.clockwise()),
                    action: actions::turn_action
                };
            },
            AI::Forward => {
                self.intent = forward_intent(self.position, self.body.facing);
            },
            AI::None => ()
        }
    }

    pub fn declare_intent(&mut self, map: &Map) {
        match &mut self.ai {
            AI::Patrolling(ai) => {
                self.intent = ai.declare_intent(self.position, &self.body, map);
            },
            AI::Rotator => {
                self.intent = Intent {
                    phase: ExecutionPhase::Movement,
                    data: IntentData::Direction(self.body.facing.clockwise()),
                    action: actions::turn_action
                };
            },
            AI::Forward => {
                self.intent = forward_intent(self.position, self.body.facing);
            },
            AI::None => ()
        }
    }


    pub fn update_view(&mut self, map: &mut Map) {
        if self.kind == EntityKind::Player {
            self.set_visible_tiles(map, false);
        }

        self.viewshed.update(self.center(), self.body.facing, map);

        if self.kind == EntityKind::Player {
            self.set_visible_tiles(map, true);
        }
    }

    pub fn set_visible_tiles(&self, map: &mut Map, visibility: bool) {
        for tile_pos in &self.viewshed.visible_tiles {
            let index = map.pos_idx(*tile_pos);
            map.visible_tiles[index] = visibility;
            map.revealed_tiles[index] = visibility | map.revealed_tiles[index];
        }
    }    

    pub fn update_abilities(&mut self) {
        self.body.update_abilities();
    }

    pub fn has_ability(&self, ability: Ability) -> bool {
        self.body.has_ability(ability)
    }

    pub fn apply_damage(&mut self, bodypart_index: usize, raw_damage: Damage) {
        let bodypart = &mut self.body.parts[bodypart_index];

        let actual_damage = bodypart.armor.modify_damage(raw_damage);
        bodypart.damage += actual_damage;

        if bodypart.damage >= bodypart.max_damage {
            self.update_abilities();
        }

        println!("{} was hit in {} for {} damage, now has {} damage",
            self.name,
            self.body.parts[bodypart_index].name,
            actual_damage,
            self.body.parts[bodypart_index].damage);
    }

    pub fn mortally_wounded(&self) -> bool {
        for bodypart in &self.body.parts {
            if bodypart.damage > bodypart.max_damage && bodypart.vital {
                return true;
            }
        }
        return false;
    }

    pub fn kill(&mut self, map: &mut Map) {
        self.clear_pawns(map);
    }

    pub fn apply_status_effect(&mut self, status: &StatusEffect) {
        self.body.apply_status_effect(status);
    }

    pub fn get_aiming_position(&self) -> Option<Point> {
        let key = StatusEffect::AimingAtGround(Point { x: 0, y: 0 }, Item::pistol());
        match self.body.get_status_effect(&key) {
            Some(StatusEffect::AimingAtGround(pos, _)) => Some(*pos),
            _ => None,
        }
    }

    pub fn clear_aiming(&mut self) {
        self.body.status_effects.retain(|s| !matches!(s, StatusEffect::AimingAtGround(..) | StatusEffect::AimingAtEntity(..)));
    }

    pub fn resolve_status_effects(&mut self) {
        // TODO
        // for effect in &self.body.status_effects {
        //     match effect {
        //         StatusEffect::AimingAtGround(pos) => {
        //             println!("{} is aiming at pos {},{}", self.name, pos.x, pos.y);
        //         }
        //     }
        // }
    }
}

fn forward_intent(pos: Point, facing: Direction) -> Intent {
    let (dx, dy): (i32, i32) = match facing {
        Direction::Up        => ( 0, -1),
        Direction::UpRight   => ( 1, -1),
        Direction::Right     => ( 1,  0),
        Direction::DownRight => ( 1,  1),
        Direction::Down      => ( 0,  1),
        Direction::DownLeft  => (-1,  1),
        Direction::Left      => (-1,  0),
        Direction::UpLeft    => (-1, -1),
    };
    Intent {
        phase: ExecutionPhase::Movement,
        data: IntentData::Target(Point { x: pos.x + dx, y: pos.y + dy }),
        action: actions::move_action
    }
}

/// A snapshot of an [`Entity`] placed on the map grid for fast spatial lookup.
///
/// `Map::pawns` is a flat tile-indexed `Vec<Option<Pawn>>`. Looking up what occupies a tile is
/// O(1) via the tile index, without scanning `World::entities`.
///
/// Pawn data mirrors its Entity at the moment `create_pawns` was last called. It is **read-only
/// from the map's perspective** — never mutate a Pawn directly. Apply changes to the owning
/// Entity and then call `set_position` or `create_pawns`/`clear_pawns` to resync.
///
/// Multi-tile entities (e.g. tanks) place one Pawn per occupied tile, each with its own
/// `sprite_index` for rendering. All of these Pawns share the same `entity_id`.
#[derive(Clone)]
pub struct Pawn {
    pub entity_id: usize,
    pub kind: EntityKind,
    pub driving: DrivingState,
    pub sprite: Sprite,
    pub sprite_index: u32,
    pub name: String,
    pub intent: Intent,
    pub body: Body
}
