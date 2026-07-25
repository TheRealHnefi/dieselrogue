use rltk::Point;
use crate::Map;
use crate::{navigate_cached, greedy_step};
use crate::Entity;
use crate::EntityKind;
use crate::util::adjacent;
use crate::components::*;
use crate::intent::*;
use crate::player;
use crate::Ability;

const SUSPICIOUS_TURNS: u32 = 30;

// ---------------------------------------------------------------------------
// Perception
// ---------------------------------------------------------------------------

struct Perception {
    /// Does the entity know that the source is hostile?
    /// Negative for things such as open doors or walking sounds
    confirmed_hostile: bool,
    /// Has the entity actually seen the source?
    /// Negative if heard or gathered through indirect means
    confirmed_visually: bool,
    /// Does the entity know exactly where the source is?
    /// Negative when finding dead bodies, open doors or hearing
    confirmed_origin: bool,
    /// The source coordinates of this perception
    origin: Point,
    // TODO: Could this replace confirmed_visually entirely?
    target_id: Option<usize>
}

/// Compare two perceptions and replace the most important one, with bias
/// towards left.
// TODO: Clean this up - use PartialOrd?
fn most_urgent(left: Option<Perception>, right: Option<Perception>) -> Option<Perception> {
    if right.is_none() {
        return left;
    }
    else if left.is_none() {
        return right;
    }

    let lhs = left.unwrap();
    let rhs = right.unwrap();

    if lhs.confirmed_hostile && !rhs.confirmed_hostile {
        return Some(lhs);
    } else if !lhs.confirmed_hostile && rhs.confirmed_hostile {
        return Some(rhs);
    }

    else if lhs.confirmed_visually && !rhs.confirmed_visually {
        return Some(lhs);
    } else if !lhs.confirmed_visually && rhs.confirmed_visually {
        return Some(rhs);
    }

    else if lhs.confirmed_origin && !rhs.confirmed_origin {
        return Some(lhs);
    } else if !lhs.confirmed_origin && rhs.confirmed_origin {
        return Some(rhs);
    }

    return Some(lhs);
}

// ---------------------------------------------------------------------------
// Decision
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum Decision {
    Idle,
    GoTo   { dest: Point, tolerance: u32 },
    Face   { toward: Point },
    Flee   { threat: Point },
    Engage { target_id: usize, last_seen: Point },
}

// ---------------------------------------------------------------------------
// AlertLevel
// ---------------------------------------------------------------------------

pub enum AlertLevel {
    /// Not acting on any threats
    Unaware,
    /// Has detected something potentially dangerous, but unconfirmed. Decays to Unaware.
    Suspicious { origin: Point, turns_remaining: u32 },
    /// Has detected something confirmed dangerous, but does not see it. Does not decay.
    Alert      { last_known: Point, search: SearchBehavior },
    /// Has detected something confirmed dangerous and has recently seen it or is seeing it now. 
    Combat     { target_id: usize, last_seen: Point },
}

// ---------------------------------------------------------------------------
// SearchBehavior — Copy so it can be extracted before borrowing self mutably
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub enum SearchBehavior {
    MoveToLastKnown,
    HoldAndWatch,
    Flank,
}

impl SearchBehavior {
    fn for_entity(entity_id: usize) -> Self {
        match entity_id % 3 {
            0 => SearchBehavior::MoveToLastKnown,
            1 => SearchBehavior::HoldAndWatch,
            _ => SearchBehavior::Flank,
        }
    }
}

// ---------------------------------------------------------------------------
// CombatTactic
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub enum CombatTactic {
    Pursue,
    Hold,
    Flee,
}

// ---------------------------------------------------------------------------
// Profile
// ---------------------------------------------------------------------------

pub enum Profile {
    Patrol {
        /// Index into [`Map::patrol_routes`] — the shared, read-only route this
        /// actor follows. Many patrollers share a route so their navigation can
        /// amortize onto the route's shared flow fields.
        route_id: usize,
        waypoint_index: usize,
        combat_tactic: CombatTactic,
    },
    Guard {
        anchor: Point,
        combat_tactic: CombatTactic,
    }
    // TODO: Add Pilot
}

impl Profile {
    fn combat_tactic(&self) -> &CombatTactic {
        match self {
            Profile::Patrol    { combat_tactic, .. } => combat_tactic,
            Profile::Guard     { combat_tactic, .. } => combat_tactic
        }
    }
}

// ---------------------------------------------------------------------------
// ActorAI
// ---------------------------------------------------------------------------

pub struct ActorAI {
    pub profile: Profile,
    pub alert:   AlertLevel,
    // Shared path cache — destination tracked to avoid redundant A* calls.
    current_path: Vec<usize>,    // reversed; .last() = next step index
    path_target:  Option<usize>, // map idx of current destination
}

impl ActorAI {
    pub fn new(profile: Profile) -> Self {
        ActorAI { profile, alert: AlertLevel::Unaware, current_path: vec![], path_target: None }
    }

    /// The goal this actor wants a shared flow field for, if any, and whether a
    /// radius-bounded field suffices (`true` for dynamic goals whose interested
    /// agents cluster nearby; `false` for static patrol/guard goals wanting
    /// full-map coverage).
    /// 
    /// Combat is intentionally excluded: a combat target is currently visible
    /// (else the actor would have decayed to Alert), so `navigate_to` reaches it
    /// via the greedy line-of-sight step, not a field.
    pub fn nav_field_goal(&self, map: &Map) -> Option<(Point, bool)> {
        match &self.alert {
            AlertLevel::Unaware => match &self.profile {
                Profile::Patrol { route_id, waypoint_index, .. } =>
                    map.patrol_routes.get(*route_id)
                        .and_then(|r| r.get(*waypoint_index).copied())
                        .map(|p| (p, false)),
                Profile::Guard { anchor, .. } => Some((*anchor, false)),
            },
            AlertLevel::Suspicious { origin, .. } => Some((*origin, true)),
            AlertLevel::Alert { last_known, search, .. } => match search {
                // Flank targets a per-agent offset (not shared); HoldAndWatch
                // doesn't move — neither benefits from a shared field.
                SearchBehavior::MoveToLastKnown => Some((*last_known, true)),
                _ => None,
            },
            AlertLevel::Combat { .. } => None,
        }
    }

    /// Follows a perceive → update → decide → execute logic for easier overview.
    pub fn compute_intent(
        &mut self,
        entity:   &Entity,
        map:      &Map,
        entities: &[Entity],
        sounds:   &[SoundEvent],
    ) -> Option<Intent> {
        #[cfg(debug_assertions)]
        puffin::profile_function!();

        // Perceive: collect this turn stimuli and return perception.
        let perception = self.perceive(entity, entities, map, sounds);

        // Update beliefs according to perceptions
        self.update_beliefs(entity, entities, perception);

        // Make a decision
        self.advance_waypoint(entity, map); // Move this later
        let decision = self.decide(entity, map);

        // nav_field_goal is a hand-kept parallel copy of decide's nav goals; the
        // pre-pass builds a shared field for its cell, so decide must actually
        // head there. Assert they agree (same self state, post-advance_waypoint).
        // TODO: Unify this with decision.
        #[cfg(debug_assertions)]
        if let Some((goal, _)) = self.nav_field_goal(map) {
            debug_assert!(
                matches!(decision, Decision::GoTo { dest, .. } if dest == goal),
                "nav_field_goal cell {:?} disagrees with decide {:?}", goal, decision,
            );
        }

        // Execute the decision
        self.execute(entity, map, entities, decision)
    }

    // --- Stimulus processing ---

    /// Returns the most important Perception, if any, to be acted upon later
    fn perceive(&mut self, entity: &Entity, entities: &[Entity], map: &Map, sounds: &[SoundEvent]) -> Option<Perception> {
        let sound_candidate = self.process_sounds(entity, sounds);
        let visual_candidate = self.process_vision(entity, entities, map);

        return most_urgent(visual_candidate, sound_candidate);
    }

    fn process_sounds(&mut self, entity: &Entity, sounds: &[SoundEvent]) -> Option<Perception> {
        #[cfg(debug_assertions)]
        puffin::profile_function!();

        let mut retval: Option<Perception> = None;

        for s in sounds {
            let dist = rltk::DistanceAlg::Pythagoras.distance2d(entity.center(), s.pos);
            if dist > s.volume as f32 || entity.center() == s.pos {
                continue;
            }

            let candidate = match s.kind {
                SoundKind::Shout | SoundKind::Gunshot | SoundKind::Burst | SoundKind::Explosion =>
                    Perception {
                        confirmed_hostile: true,
                        confirmed_origin: false,
                        confirmed_visually: false,
                        origin: s.pos,
                        target_id: None
                    },
                SoundKind::Footstep | SoundKind::Engine =>
                    Perception {
                        confirmed_hostile: false,
                        confirmed_origin: true,
                        confirmed_visually: false,
                        origin: s.pos,
                        target_id: None
                    },
            };
            
            retval = most_urgent(retval, Some(candidate));
        }

        retval
    }

    fn process_vision(&mut self, entity: &Entity, entities: &[Entity], map: &Map) -> Option<Perception> {
        #[cfg(debug_assertions)]
        puffin::profile_function!();
            
        for point in &entity.viewshed.visible_tiles {
            // Return if player is seen, because it is more important than anything else
            if let Some(entity_id) = map.get_entity_id(point.x, point.y) {
                if entities[entity_id].kind == EntityKind::Player {
                    let pc = entities[entity_id].center();
                    return Some (Perception {
                        confirmed_hostile: true,
                        confirmed_origin: true,
                        confirmed_visually: true,
                        origin: pc,
                        target_id: Some(entity_id)
                    });
                }
            }
            // Return on first corpse seen, because it's the current most urgent case possible.
            if let Some(item) = map.get_item_ref(point.x, point.y) {
                if item.kind == ItemKind::Corpse {
                    return Some (Perception {
                        confirmed_hostile: true,
                        confirmed_origin: false,
                        confirmed_visually: false,
                        origin: point.clone(),
                        target_id: None
                    });
                }
            }
        }

        None
    }

    // --- Belief updates ---

    fn update_beliefs(&mut self, entity: &Entity, entities: &[Entity], perception: Option<Perception>) {
        self.decay_alertness(entity, entities);
        if let Some(p) = perception {
            self.update_target(p);
        }
    }

    fn update_target(&mut self, perception: Perception) {
        if matches!(self.alert, AlertLevel::Combat { .. }) {
            // We are already in combat. Ignore target updates.
            return;
        }
        else if perception.confirmed_visually && perception.confirmed_hostile {
            // Enemy spotted. We are in combat.
            // TODO: Handle that unwrap more gracefully? Potentially replace confirmed_visually and rely on this?
            self.alert = AlertLevel::Combat { target_id: perception.target_id.unwrap(), last_seen: perception.origin };
        }
        else if perception.confirmed_hostile {
            // We know the enemy is around here somewhere. Update position.
            // TODO: This will cause the AI to get hung up on dead bodies. Handle this with a SearchTheArea behavior.
            self.alert = AlertLevel::Alert { last_known: perception.origin, search: SearchBehavior::MoveToLastKnown };
        }
        else {
            self.alert = AlertLevel::Suspicious { origin: perception.origin, turns_remaining: SUSPICIOUS_TURNS }
        }
    }

    fn decay_alertness(&mut self, entity: &Entity, entities: &[Entity]) {
        #[cfg(debug_assertions)]
        puffin::profile_function!();
        // Combat → Alert when target leaves sight.
        if let AlertLevel::Combat { target_id, last_seen } = &self.alert {
            let (tid, ls) = (*target_id, *last_seen);
            // TODO: This is probably more efficient to do the other way around - iterate over viewshed to check for existence of target id
            // TODO: Do we even need this considering we do the same thing in the perception step?
            let still_visible = entities.iter()
                .find(|e| e.index == tid)
                .map_or(false, |t| entity.viewshed.visible_tiles.contains(&t.center()));
            if !still_visible {
                self.alert = AlertLevel::Alert {
                    last_known: ls,
                    search: SearchBehavior::for_entity(entity.index),
                };
            }
            return;
        }

        // Decay timed states.
        let transition: Option<AlertLevel> = match &self.alert {
            AlertLevel::Suspicious { turns_remaining, origin } if *turns_remaining == 0 =>
                Some(AlertLevel::Unaware),
            _ => None,
        };

        if let Some(new) = transition {
            self.alert = new;
        } else {
            match &mut self.alert {
                AlertLevel::Suspicious { turns_remaining, .. } => *turns_remaining -= 1,
                _ => {}
            }
        }
    }

    // --- Decision ---

    /// Advance a patroller to its next waypoint once it stands on the current one.
    fn advance_waypoint(&mut self, entity: &Entity, map: &Map) {
        if !matches!(self.alert, AlertLevel::Unaware) { return; }
        if let Profile::Patrol { route_id, waypoint_index, .. } = &mut self.profile {
            if let Some(route) = map.patrol_routes.get(*route_id) {
                if !route.is_empty() && route[*waypoint_index] == entity.position {
                    *waypoint_index = (*waypoint_index + 1) % route.len();
                    self.path_target = None;
                }
            }
        }
    }

    /// The decision tree: current (alert, profile) state → a Decision. Pure, and
    /// every Decision field is Copy, so no borrow of self outlives the call and
    /// execute can then take &mut self freely.
    fn decide(&self, entity: &Entity, map: &Map) -> Decision {
        match &self.alert {
            AlertLevel::Unaware => match &self.profile {
                Profile::Patrol { route_id, waypoint_index, .. } =>
                    match map.patrol_routes.get(*route_id).and_then(|r| r.get(*waypoint_index)) {
                        Some(&dest) => Decision::GoTo { dest, tolerance: 0 },
                        None        => Decision::Idle,
                    },
                Profile::Guard { anchor, .. } => Decision::GoTo { dest: *anchor, tolerance: 0 },
            },
            AlertLevel::Suspicious { origin, .. } => Decision::GoTo { dest: *origin, tolerance: 0 },
            AlertLevel::Alert { last_known, search } => match search {
                SearchBehavior::HoldAndWatch    => Decision::Face { toward: *last_known },
                SearchBehavior::MoveToLastKnown => Decision::GoTo { dest: *last_known, tolerance: 0 },
                SearchBehavior::Flank           =>
                    Decision::GoTo { dest: self.flank_destination(entity.position, *last_known, map), tolerance: 0 },
            },
            AlertLevel::Combat { target_id, last_seen } => match self.profile.combat_tactic() {
                CombatTactic::Flee => Decision::Flee { threat: *last_seen },
                _                  => Decision::Engage { target_id: *target_id, last_seen: *last_seen },
            },
        }
    }

    /// Carry out a Decision, producing a concrete intent.
    fn execute(&mut self, entity: &Entity, map: &Map, entities: &[Entity], decision: Decision) -> Option<Intent> {
        match decision {
            Decision::Idle => None,
            Decision::GoTo { dest, tolerance } => self.navigate_to(entity, dest, map, entities, tolerance),
            Decision::Face { toward } => face_intent(entity, toward),
            Decision::Flee { threat } => {
                let dest = self.flee_pos(entity, threat, map);
                self.navigate_to(entity, dest, map, entities, 0)
            },
            Decision::Engage { target_id, last_seen } =>
                self.engage(entity, map, entities, target_id, last_seen),
        }
    }

    fn flank_destination(&self, from: Point, target: Point, map: &Map) -> Point {
        #[cfg(debug_assertions)]
        puffin::profile_function!();
        // Approach last-known from a perpendicular angle (5 tiles offset).
        let dx = target.x - from.x;
        let dy = target.y - from.y;
        let (perp_x, perp_y) = if dx.abs() >= dy.abs() {
            (0i32, if dy >= 0 { -5 } else { 5 })
        } else {
            (if dx >= 0 { -5 } else { 5 }, 0i32)
        };
        Point {
            x: (target.x + perp_x).clamp(0, map.width as i32 - 1),
            y: (target.y + perp_y).clamp(0, map.height as i32 - 1),
        }
    }

    // --- Behaviour: Combat ---

    fn engage(
        &mut self,
        entity:    &Entity,
        map:       &Map,
        entities:  &[Entity],
        target_id: usize,
        last_seen: Point,
    ) -> Option<Intent> {
        #[cfg(debug_assertions)]
        puffin::profile_function!();

        if let Some(target) = entities.iter().find(|e| e.index == target_id) {
            let tc = target.center();

            // Melee if adjacent — via resolve_step so the AI turns to face first.
            if adjacent(entity.position, tc) {
                return match direction_to(entity.position, tc) {
                    Some(dir) => resolve_step(entity, dir, map, entities).ok().flatten(),
                    None      => Some(melee_intent(tc)),
                };
            }

            // Ranged: fire if aim is ready, else spend the turn acquiring it
            // (fire actions require an active aim status, same as the player menu).
            if let Some((slot, range)) = find_weapon(entity) {
                let dist = rltk::DistanceAlg::Pythagoras.distance2d(entity.center(), tc);
                if dist <= range as f32 {
                    let available = player::get_entity_available_actions(entity, map);
                    let fire = available.iter().find(|(a, s)| *s == Some(slot) && matches!(a.targeting, Targeting::UseExistingAim { .. }));
                    let aim  = available.iter().find(|(a, s)| *s == Some(slot) && matches!(a.targeting, Targeting::EntityAim { .. }));
                    if let Some(&(action, _)) = fire.or(aim) {
                        return Some(build_intent(action, Some(ActionSource::EquippedSlot(slot)), Resolution::Position(tc)));
                    }
                }
            }
        }

        // No attack available — pursue or hold.
        match self.profile.combat_tactic() {
            CombatTactic::Pursue => {
                let dest = entities.iter().find(|e| e.index == target_id)
                    .map(|t| t.center()).unwrap_or(last_seen);
                self.navigate_to(entity, dest, map, entities, 1)
            },
            CombatTactic::Hold => None,
            CombatTactic::Flee => unreachable!("Flee is routed to Decision::Flee"),
        }
    }

    fn flee_pos(&self, entity: &Entity, threat: Point, map: &Map) -> Point {
        #[cfg(debug_assertions)]
        puffin::profile_function!();
        let deltas: [(i32,i32);8] = [(-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)];
        deltas.iter()
            .map(|(dx,dy)| Point { x: entity.position.x + dx, y: entity.position.y + dy })
            .filter(|&p| {
                p.x >= 0 && p.y >= 0
                && p.x < map.width as i32 && p.y < map.height as i32
                && !map.blocked(p.x, p.y)
            })
            .max_by_key(|p| {
                let dx = p.x - threat.x;
                let dy = p.y - threat.y;
                dx * dx + dy * dy
            })
            .unwrap_or(entity.position)
    }

    // --- Navigation ---

    /// A* fallback shared by every branch of `navigate_to`: repath (respecting
    /// the cache/tolerance) and return the next tile to step onto, if any.
    fn astar_step(&mut self, from_idx: usize, dest_idx: usize, map: &Map, tolerance: u32) -> Option<Point> {
        navigate_cached(from_idx, dest_idx, map, &mut self.current_path, &mut self.path_target, tolerance);
        self.current_path.last().map(|&i| map.idx_pos(i))
    }

    fn navigate_to(&mut self, entity: &Entity, destination: Point, map: &Map, entities: &[Entity], tolerance: u32) -> Option<Intent> {
        #[cfg(debug_assertions)]
        puffin::profile_function!();
        if entity.position == destination {
            return None;
        }
        if !entity.has_ability(Ability::HumanMove) {
            return None;
        }

        let dest_idx = map.pos_idx(destination);

        // Consume the path step we just reached.
        if let Some(&next_idx) = self.current_path.last() {
            if map.idx_pos(next_idx) == entity.position {
                self.current_path.pop();
            }
        }

        let from_idx = map.pos_idx(entity.position);

        // If the destination tile is visible, try O(8) greedy neighbour first.
        // Fall back to A* only when greedy is stuck (no adjacent tile is closer).
        // The A* cache is left untouched on a greedy success so it stays warm
        // for when the target goes out of sight.
        let next_pos = if entity.viewshed.visible_tiles.contains(&destination) {
            if let Some(idx) = greedy_step(from_idx, dest_idx, map) {
                // Invalidate the A* cache: the entity is moving off the cached
                // path, so reusing it later would produce a non-adjacent first
                // step and crash direction_to.
                self.path_target = None;
                Some(map.idx_pos(idx))
            } else {
                // Stuck on a corner with a visible target — fall through to A*.
                self.astar_step(from_idx, dest_idx, map, tolerance)
            }
        } else if let Some(idx) = map.field_step(from_idx, dest_idx) {
            // A resident static-terrain flow field covers this goal (e.g. a
            // patrol waypoint or guard anchor): obstacle-aware O(8) descent,
            // shared across every agent heading here, with no per-agent A*.
            // Falls through to A* below only if the field can't produce a step
            // (or flow fields are disabled).
            self.path_target = None;
            Some(map.idx_pos(idx))
        } else {
            self.astar_step(from_idx, dest_idx, map, tolerance)
        }?;

        match direction_to(entity.position, next_pos) {
            Some(dir) => resolve_step(entity, dir, map, entities).ok().flatten(),
            None => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A resolved AI decision: produced by `decide`, carried out by `execute`.

/// Turn to face `toward` (any distance), or None if already facing it.
fn face_intent(entity: &Entity, toward: Point) -> Option<Intent> {
    match direction_toward(entity.position, toward) {
        Some(dir) if entity.body.facing != dir => Some(turn_intent(dir)),
        _ => None,
    }
}

/// Direction from an adjacent `to`; debug-asserts adjacency.
fn direction_to(from: Point, to: Point) -> Option<Direction> {
    match (to.x - from.x, to.y - from.y) {
        ( 0, -1) => Some(Direction::Up),
        ( 1, -1) => Some(Direction::UpRight),
        ( 1,  0) => Some(Direction::Right),
        ( 1,  1) => Some(Direction::DownRight),
        ( 0,  1) => Some(Direction::Down),
        (-1,  1) => Some(Direction::DownLeft),
        (-1,  0) => Some(Direction::Left),
        (-1, -1) => Some(Direction::UpLeft),
        (dx, dy) => {
            debug_assert!(false, "non-adjacent delta ({},{}) from {:?} to {:?}", dx, dy, from, to);
            None
        }
    }
}

/// Nearest 8-way direction pointing from `from` toward `to` (any distance).
fn direction_toward(from: Point, to: Point) -> Option<Direction> {
    match ((to.x - from.x).signum(), (to.y - from.y).signum()) {
        ( 0, -1) => Some(Direction::Up),
        ( 1, -1) => Some(Direction::UpRight),
        ( 1,  0) => Some(Direction::Right),
        ( 1,  1) => Some(Direction::DownRight),
        ( 0,  1) => Some(Direction::Down),
        (-1,  1) => Some(Direction::DownLeft),
        (-1,  0) => Some(Direction::Left),
        (-1, -1) => Some(Direction::UpLeft),
        _        => None,
    }
}

/// Returns the first equipped firearm with remaining ammo, and its range.
fn find_weapon(entity: &Entity) -> Option<(SlotType, u32)> {
    entity.body.item_slots.iter().find_map(|slot| {
        if let Some(item) = &slot.item {
            if let ItemKind::Firearm { ammo, range, .. } = item.kind {
                if ammo > 0 { return Some((slot.slot_type, range)); }
            }
        }
        None
    })
}

fn forward_intent(pos: Point, facing: Direction) -> Intent {
    let (dx, dy) = facing.delta_pos();
    move_intent(Point { x: pos.x + dx, y: pos.y + dy })
}

// ---------------------------------------------------------------------------
// AI enum — public entry point
// ---------------------------------------------------------------------------

pub enum AI {
    None,
    Rotator,
    Forward,
    Actor(ActorAI),
}

impl AI {
    pub fn compute_intent(
        &mut self,
        entity:   &Entity,
        map:      &Map,
        entities: &[Entity],
        sounds:   &[SoundEvent],
    ) -> Option<Intent> {
        match self {
            AI::None => None,
            AI::Rotator => Some(turn_intent(entity.body.facing.clockwise())),
            AI::Forward => Some(forward_intent(entity.position, entity.body.facing)),
            AI::Actor(actor) => actor.compute_intent(entity, map, entities, sounds),
        }
    }
}
