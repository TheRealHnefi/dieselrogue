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
    // Unstable index, recalculated every frame.
    // TODO: Add stable ID, for AI targeting purposes
    pub index: usize,
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
    pub ai: AI,
    pub color: Option<usize>,
    pub paper_doll: Option<PaperDoll>,
    /// Actions the entity can perform independent of items — choosable by both
    /// the player menu and the AI. Filtered by each action's precondition at
    /// query time via `get_available_actions`.
    pub innate_actions: Vec<EntityAction>,
    // XP granted for killing actor
    pub xp_value: usize
}

fn human_innate_actions() -> Vec<EntityAction> {
    vec![
        shout_action_def(),
        EntityAction {
            id: ActionId::IronBody,
            name: "Iron Body".to_string(),
            targeting: Targeting::None,
            phase: ExecutionPhase::Inventory,
            precondition: |e, _, _| e.has_ability(Ability::IronBody),
            action: actions::iron_body_action,
        },
        EntityAction {
            id: ActionId::Rush,
            name: "Rush".to_string(),
            targeting: Targeting::EntityAim { max_range: Some(3) },
            phase: ExecutionPhase::Inventory,
            precondition: |e, _, _| e.has_ability(Ability::Rush),
            action: actions::rush_action,
        },
        EntityAction {
            id: ActionId::Twist,
            name: "Twist".to_string(),
            targeting: Targeting::EntityAim { max_range: Some(1) },
            phase: ExecutionPhase::Inventory,
            precondition: |e, _, _| e.has_ability(Ability::Twist),
            action: actions::twist_action,
        },
        EntityAction {
            id: ActionId::Distract,
            name: "Distract".to_string(),
            targeting: Targeting::EntityAim { max_range: Some(10) },
            phase: ExecutionPhase::Inventory,
            precondition: |e, _, _| e.has_ability(Ability::Distract),
            action: actions::distract_action,
        },
        juke_action_def(),
        disembark_action_def(),
    ]
}

impl Entity {
    pub fn new_human(id: usize, pos: Point, facing: Direction, name: String) -> Self {
        Self {
            index: id,
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
            ai: AI::None,
            color: None,
            paper_doll: None,
            innate_actions: human_innate_actions(),
            xp_value: 1000
        }
    }

    pub fn new_patrolling_goon(id: usize, pos: Point, facing: Direction, name: String, route_id: usize) -> Self {
        Self {
            index: id,
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
            ai: AI::Actor(ActorAI::new(Profile::Patrol {
                route_id,
                waypoint_index: 0,
                combat_tactic: CombatTactic::Pursue,
            })),
            color: None,
            paper_doll: None,
            innate_actions: human_innate_actions(),
            xp_value: 1000
        }
    }

    pub fn new_tank(id: usize, pos: Point, facing: Direction, name: String) -> Self {
        Self {
            index: id,
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
            ai: AI::Rotator,
            color: None,
            paper_doll: None,
            innate_actions: vec![],
            xp_value: 1000
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
            index: id,
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
            ai: AI::None,
            color: None,
            paper_doll: None,
            innate_actions: vec![],
            xp_value: 0
        }
    }

    pub fn check_fit(&self, pos: Point, map: &Map) -> bool {
        for x in 0..self.size_x {
            for y in 0..self.size_y {
                let index = map.xy_idx(pos.x + x as i32, pos.y + y as i32);
                match &map.pawns[index] {
                    Some(pawn) => {
                        if pawn.entity_id != self.index {
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
                            TileType::Fence => return false,
                            TileType::Window => return false,
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
                    entity_id: self.index,
                    sprite_index: x + y * self.size_x,
                });
                map.fov_blocked[index] = self.kind == EntityKind::Door;
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
                map.fov_blocked[index] = false;
            }
        }
    }

    pub fn set_position(&mut self, pos: Point, map: &mut Map) {
        self.clear_pawns(map);
        self.position = pos;
        self.create_pawns(map);
        // Player visible tiles must update immediately for correct rendering.
        // Non-player viewsheds are refreshed at end of turn by the parallel
        // viewshed pass in resolve_status_effects — no need to do it inline.
        if self.kind == EntityKind::Player {
            self.update_view(map);
        }
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
        if let Some(item_index) = self.body.item_slots.iter().position(|s| s.slot_type == slot) {
            self.body.item_slots[item_index].item.as_mut()
        } else {
            None
        }
    }

    pub fn get_equipped_item_ref(&self, slot: SlotType) -> Option<&Item> {
        self.body.item_slots.iter()
            .find(|s| s.slot_type == slot)
            .and_then(|s| s.item.as_ref())
    }

    pub fn take_item_by_id(&mut self, item_id: usize) -> Option<Item> {
        if let Some(pos) = self.body.inventory.iter().position(|i| i.id == item_id) {
            Some(self.body.inventory.remove(pos))
        } else {
            None
        }
    }

    /// Find an item by id in the inventory or any equipped slot.
    pub fn find_item_by_id(&self, item_id: usize) -> Option<&Item> {
        self.body.inventory.iter().find(|i| i.id == item_id)
            .or_else(|| self.body.item_slots.iter()
                .filter_map(|s| s.item.as_ref())
                .find(|i| i.id == item_id))
    }

    /// Mutable lookup by id across the inventory and equipped slots.
    pub fn find_item_by_id_mut(&mut self, item_id: usize) -> Option<&mut Item> {
        if let Some(pos) = self.body.inventory.iter().position(|i| i.id == item_id) {
            return self.body.inventory.get_mut(pos);
        }
        self.body.item_slots.iter_mut()
            .filter_map(|s| s.item.as_mut())
            .find(|i| i.id == item_id)
    }

    /// Returns all actions currently available to `entity`: equipped-item actions
    /// whose preconditions pass, followed by innate actions whose preconditions pass.
    /// `Option<SlotType>` is `Some(slot)` for equipped actions, `None` for innate ones.
    /// This is the authoritative source used by both the player menu and the AI.
    pub fn get_available_actions<'a>(&'a self, map: &Map) -> Vec<(&'a EntityAction, Option<SlotType>)> {
        let mut result = Vec::new();
        for slot in &self.body.item_slots {
            let Some(item) = &slot.item else { continue };
            if item.proxy { continue; }
            for action in &item.equip_actions {
                if (action.precondition)(self, map, Some(item)) {
                    result.push((action, Some(slot.slot_type)));
                }
            }
        }
        for action in &self.innate_actions {
            if (action.precondition)(self, map, None) {
                result.push((action, None));
            }
        }
        result
    }

    pub fn update_view(&mut self, map: &mut Map) {
        if self.kind == EntityKind::Player {
            self.set_visible_tiles(map, false);
        }
        let fov = self.effective_fov();
        let range = self.effective_range();
        let facing = self.effective_facing();
        self.viewshed.update(self.center(), facing, range, &fov, map);
        if self.kind == EntityKind::Player {
            self.set_visible_tiles(map, true);
        }
    }

    /// Recomputes this entity's viewshed using a shared map reference.
    /// Does not touch `map.visible_tiles` — caller is responsible for
    /// updating the player's tile markings around the parallel viewshed pass.
    pub fn update_viewshed_only(&mut self, map: &Map) {
        let fov = self.effective_fov();
        let range = self.effective_range();
        let facing = self.effective_facing();
        self.viewshed.update(self.center(), facing, range, &fov, map);
    }

    /// The tile a recon Scanning status is aimed at, if any.
    fn scan_target(&self) -> Option<Point> {
        match self.body.get_status_effect(&StatusEffect::Scanning(Point { x: 0, y: 0 })) {
            Some(StatusEffect::Scanning(target)) => Some(*target),
            _ => None,
        }
    }

    /// Facing used for vision: while scanning, the heading toward the scan target;
    /// otherwise the body's facing.
    fn effective_facing(&self) -> Direction {
        if let Some(target) = self.scan_target() {
            if let Some(dir) = Direction::nearest(target.x - self.center().x, target.y - self.center().y) {
                return dir;
            }
        }
        self.body.facing
    }

    fn effective_fov(&self) -> FieldOfView {
        if self.scan_target().is_some() {
            FieldOfView::Fov90 // recon replaces normal (and wide) vision with a narrow cone
        } else if self.body.has_ability(Ability::WideVision) {
            FieldOfView::Fov270
        } else {
            self.viewshed.fov.clone()
        }
    }

    fn effective_range(&self) -> i32 {
        if self.body.get_status_effect(&StatusEffect::Blind(0)).is_some() {
            return 1;
        }
        if self.scan_target().is_some() {
            return 40; // recon reaches much farther than normal sight
        }
        let base = self.viewshed.range;
        if self.body.has_ability(Ability::EagleEyes) {
            base * 3 / 2
        } else {
            base
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

    pub fn can_see(&self, pos: Point) -> bool {
        self.viewshed.visible_tiles.contains(&pos)
    }

    /// Returns the target bodypart index and final damage for a melee attack against `target`.
    /// Applies all attacker-side modifiers (Pugilism, Backstab, etc.) in one place.
    pub fn melee_strike(&self, target: &Entity) -> (usize, Damage) {
        let bodypart_index = if self.has_ability(Ability::Pugilism) { 0 } else { 1 };
        let base = self.body.item_slots.iter()
            .find(|s| s.slot_type == SlotType::SecondaryHand)
            .and_then(|s| s.item.as_ref())
            .and_then(|item| if let ItemKind::MeleeWeapon { damage } = item.kind { Some(damage) } else { None })
            .unwrap_or(Damage::new(1, 0, 0, 0));
        let mut damage = base;
        if self.has_ability(Ability::Backstab)
            && matches!(target.sprite, Sprite::Human)
            && !target.can_see(self.position)
        {
            damage.physical *= 5;
        }
        (bodypart_index, damage)
    }

    pub fn apply_damage(&mut self, bodypart_index: usize, raw_damage: Damage) {
        let iron_body = self.body.get_status_effect(&StatusEffect::IronBody(0)).is_some();
        let bodypart = &mut self.body.parts[bodypart_index];
        let effective_armor = if iron_body {
            let mut a = bodypart.armor.clone();
            a.phys_resistance = (a.phys_resistance + 0.5).min(1.0);
            a
        } else {
            bodypart.armor.clone()
        };
        let actual_damage = effective_armor.modify_damage(raw_damage);
        bodypart.damage += actual_damage;
        // Cap accumulated damage at twice the part's maximum so it can't grow unbounded.
        bodypart.damage = bodypart.damage.min(2 * bodypart.max_damage);

        if bodypart.damage > bodypart.max_damage {
            self.update_abilities();
        }

        println!("{} was hit in {} for {} damage, now has {} damage",
            self.name,
            self.body.parts[bodypart_index].name,
            actual_damage,
            self.body.parts[bodypart_index].damage);
    }

    /// Reduce a body part's accumulated damage by `amount` (toward full health),
    /// restoring abilities if the part becomes functional again.
    pub fn heal(&mut self, bodypart_index: usize, amount: u32) {
        let bodypart = &mut self.body.parts[bodypart_index];
        let was_disabled = bodypart.damage > bodypart.max_damage;
        bodypart.damage = bodypart.damage.saturating_sub(amount);
        if was_disabled && bodypart.damage <= bodypart.max_damage {
            self.update_abilities();
        }
    }

    pub fn mortally_wounded(&self) -> bool {
        for bodypart in &self.body.parts {
            if bodypart.damage >= bodypart.max_damage && bodypart.vital {
                return true;
            }
        }
        return false;
    }

    pub fn apply_status_effect(&mut self, status: &StatusEffect) {
        self.body.apply_status_effect(status);
    }

    pub fn clear_aiming(&mut self) {
        self.body.status_effects.retain(|s| !matches!(s, StatusEffect::AimingAtGround(..) | StatusEffect::AimingAtEntity(..)));
    }

    /// Ends recon vision (called when the wearer moves, turns, or removes the helmet).
    pub fn clear_scanning(&mut self) {
        self.body.status_effects.retain(|s| !matches!(s, StatusEffect::Scanning(..)));
    }

    pub fn resolve_status_effects(&mut self) -> Vec<Effect> {
        let mut effects = vec![];
        if self.body.get_status_effect(&StatusEffect::Burning(0)).is_some() {
            for i in 0..self.body.parts.len() {
                effects.push(Effect::BurnTick { entity_id: self.index, bodypart_index: i });
            }
        }
        // One heal tick per body part that still has regeneration turns left.
        if let Some(StatusEffect::Regenerating(turns)) =
            self.body.get_status_effect(&StatusEffect::Regenerating(vec![]))
        {
            for (i, &t) in turns.iter().enumerate() {
                if t > 0 {
                    effects.push(Effect::RegenTick { entity_id: self.index, bodypart_index: i });
                }
            }
        }
        self.body.resolve_status_effects();
        effects
    }
}


/// A snapshot of an [`Entity`] placed on the map grid for fast spatial lookup.
///
/// `Map::pawns` is a flat tile-indexed `Vec<Option<Pawn>>`. Looking up what occupies a tile is
/// O(1) via the tile index, without scanning `World::entities`.
///
/// Multi-tile entities (e.g. tanks) place one Pawn per occupied tile, each with its own
/// `sprite_index` for rendering. All of these Pawns share the same `entity_id`.
/// For any other entity data, look up `World::entities[pawn.entity_id]`.
#[derive(Clone)]
pub struct Pawn {
    pub entity_id: usize,
    pub sprite_index: u32,
}
