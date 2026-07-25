use rltk::Point;
use crate::Map;
use crate::{navigate_cached, greedy_step};
use crate::Entity;
use crate::EntityKind;
use crate::util::adjacent;
use crate::components::*;
use crate::intent::*;
use crate::actions;
use crate::player;
use crate::Ability;

const SUSPICIOUS_TURNS: u32 = 15;
const ALERT_TURNS:      u32 = 30;
const SHOUT_VOLUME:     u32 = 15;

// ---------------------------------------------------------------------------
// AlertLevel
// ---------------------------------------------------------------------------

pub enum AlertLevel {
    Unaware,
    Suspicious { origin: Point, turns_remaining: u32 },
    Alert      { last_known: Point, turns_remaining: u32, search: SearchBehavior },
    Combat     { target_id: usize, last_seen: Point },
}

impl AlertLevel {
    fn priority(&self) -> u8 {
        match self {
            AlertLevel::Unaware           => 0,
            AlertLevel::Suspicious { .. } => 1,
            AlertLevel::Alert { .. }      => 2,
            AlertLevel::Combat { .. }     => 3,
        }
    }

    /// Only escalate; never de-escalate through this method.
    fn try_escalate(&mut self, candidate: AlertLevel) {
        if candidate.priority() >= self.priority() {
            *self = candidate;
        }
    }
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
    },
    Follow {
        target_id: usize,
        last_known_pos: Point,
        combat_tactic: CombatTactic,
    },
    Stationary {
        combat_tactic: CombatTactic,
    },
}

impl Profile {
    fn combat_tactic(&self) -> &CombatTactic {
        match self {
            Profile::Patrol    { combat_tactic, .. } => combat_tactic,
            Profile::Guard     { combat_tactic, .. } => combat_tactic,
            Profile::Follow    { combat_tactic, .. } => combat_tactic,
            Profile::Stationary{ combat_tactic }     => combat_tactic,
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
    /// full-map coverage). Read by the field pre-pass *before* this turn's
    /// stimulus is processed, so it reflects last turn's belief — a just-changed
    /// belief simply misses its field for one turn and falls back to A*.
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
                _ => None,
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

    pub fn compute_intent(
        &mut self,
        entity:   &Entity,
        map:      &Map,
        entities: &[Entity],
        sounds:   &[SoundEvent],
    ) -> (Option<Intent>, Vec<SoundEvent>) {
        #[cfg(debug_assertions)]
        puffin::profile_function!();

        let prev_priority = self.alert.priority();

        self.process_sounds(entity, sounds);
        self.process_vision(entity, entities);
        self.check_follow_target(entities);
        self.tick_alert(entity, entities);

        // Emit shout when first reaching Alert or Combat.
        let mut emitted = vec![];
        if self.alert.priority() >= 2 && prev_priority < 2 {
            emitted.push(SoundEvent { kind: SoundKind::Shout, pos: entity.center(), volume: SHOUT_VOLUME });
        }

        let intent = self.dispatch_intent(entity, map, entities);
        (intent, emitted)
    }

    // --- Stimulus processing ---

    fn process_sounds(&mut self, entity: &Entity, sounds: &[SoundEvent]) {
        #[cfg(debug_assertions)]
        puffin::profile_function!();
        for s in sounds {
            let dist = rltk::DistanceAlg::Pythagoras.distance2d(entity.center(), s.pos);
            if dist > s.volume as f32 { continue; }

            let candidate = match s.kind {
                SoundKind::Shout =>
                    AlertLevel::Suspicious { origin: s.pos, turns_remaining: SUSPICIOUS_TURNS },
                SoundKind::Gunshot | SoundKind::Burst | SoundKind::Explosion =>
                    AlertLevel::Alert {
                        last_known: s.pos,
                        turns_remaining: ALERT_TURNS,
                        search: SearchBehavior::for_entity(entity.id),
                    },
                SoundKind::Footstep | SoundKind::Engine =>
                    AlertLevel::Suspicious { origin: s.pos, turns_remaining: SUSPICIOUS_TURNS },
            };
            self.alert.try_escalate(candidate);
        }
    }

    fn process_vision(&mut self, entity: &Entity, entities: &[Entity]) {
        #[cfg(debug_assertions)]
        puffin::profile_function!();
        if let Some(player) = entities.iter().find(|e| e.kind == EntityKind::Player) {
            let pc = player.center();
            if entity.viewshed.visible_tiles.contains(&pc) {
                self.alert.try_escalate(AlertLevel::Combat { target_id: player.id, last_seen: pc });
            }
        }
    }

    fn check_follow_target(&mut self, entities: &[Entity]) {
        #[cfg(debug_assertions)]
        puffin::profile_function!();
        if let Profile::Follow { target_id, last_known_pos, .. } = &mut self.profile {
            match entities.iter().find(|e| e.id == *target_id) {
                Some(t) => *last_known_pos = t.center(),
                None    => {
                    let lkp = *last_known_pos;
                    self.alert.try_escalate(AlertLevel::Alert {
                        last_known: lkp,
                        turns_remaining: ALERT_TURNS,
                        search: SearchBehavior::MoveToLastKnown,
                    });
                }
            }
        }
    }

    fn tick_alert(&mut self, entity: &Entity, entities: &[Entity]) {
        #[cfg(debug_assertions)]
        puffin::profile_function!();
        // Combat → Alert when target leaves sight.
        if let AlertLevel::Combat { target_id, last_seen } = &self.alert {
            let (tid, ls) = (*target_id, *last_seen);
            let still_visible = entities.iter()
                .find(|e| e.id == tid)
                .map_or(false, |t| entity.viewshed.visible_tiles.contains(&t.center()));
            if !still_visible {
                self.alert = AlertLevel::Alert {
                    last_known: ls,
                    turns_remaining: ALERT_TURNS,
                    search: SearchBehavior::for_entity(entity.id),
                };
            }
            return;
        }

        // Decay timed states.
        let transition: Option<AlertLevel> = match &self.alert {
            AlertLevel::Suspicious { turns_remaining, origin } if *turns_remaining == 0 =>
                Some(AlertLevel::Unaware),
            AlertLevel::Alert { turns_remaining, last_known, .. } if *turns_remaining == 0 =>
                Some(AlertLevel::Suspicious { origin: *last_known, turns_remaining: SUSPICIOUS_TURNS }),
            _ => None,
        };

        if let Some(new) = transition {
            self.alert = new;
        } else {
            match &mut self.alert {
                AlertLevel::Suspicious { turns_remaining, .. } => *turns_remaining -= 1,
                AlertLevel::Alert      { turns_remaining, .. } => *turns_remaining -= 1,
                _ => {}
            }
        }
    }

    // --- Intent dispatch ---

    /// Extracts a Copy snapshot of the alert state so we can call &mut self methods
    /// without a lingering borrow on self.alert.
    fn alert_snapshot(&self) -> AlertSnapshot {
        match &self.alert {
            AlertLevel::Unaware =>
                AlertSnapshot::Unaware,
            AlertLevel::Suspicious { origin, .. } =>
                AlertSnapshot::Suspicious { origin: *origin },
            AlertLevel::Alert { last_known, search, .. } =>
                AlertSnapshot::Alert { last_known: *last_known, search: *search },
            AlertLevel::Combat { target_id, last_seen } =>
                AlertSnapshot::Combat { target_id: *target_id, last_seen: *last_seen },
        }
    }

    fn dispatch_intent(&mut self, entity: &Entity, map: &Map, entities: &[Entity]) -> Option<Intent> {
        match self.alert_snapshot() {
            AlertSnapshot::Unaware =>
                self.unaware_intent(entity, map, entities),
            AlertSnapshot::Suspicious { origin } =>
                self.navigate_to(entity, origin, map, 0),
            AlertSnapshot::Alert { last_known, search } =>
                self.search_intent(entity, map, last_known, search),
            AlertSnapshot::Combat { target_id, last_seen } =>
                self.combat_intent(entity, map, entities, target_id, last_seen),
        }
    }

    // --- Behaviour: Unaware ---

    fn unaware_intent(&mut self, entity: &Entity, map: &Map, entities: &[Entity]) -> Option<Intent> {
        #[cfg(debug_assertions)]
        puffin::profile_function!();
        match &mut self.profile {
            Profile::Patrol { route_id, waypoint_index, .. } => {
                let route = &map.patrol_routes[*route_id];
                if route.is_empty() {
                    return None;
                }
                // Advance waypoint if arrived.
                if route[*waypoint_index] == entity.position {
                    *waypoint_index = (*waypoint_index + 1) % route.len();
                    self.path_target = None;
                }
                let dest = route[*waypoint_index];
                self.navigate_to(entity, dest, map, 0)
            },
            Profile::Guard { anchor, .. } => {
                let anchor = *anchor;
                if entity.position == anchor {
                    None // already at post
                } else {
                    self.navigate_to(entity, anchor, map, 0)
                }
            },
            Profile::Follow { target_id, last_known_pos, .. } => {
                let dest = entities.iter()
                    .find(|e| e.id == *target_id)
                    .map(|t| t.center())
                    .unwrap_or(*last_known_pos);
                if adjacent(entity.position, dest) {
                    None
                } else {
                    self.navigate_to(entity, dest, map, 2)
                }
            },
            Profile::Stationary { .. } => None,
        }
    }

    // --- Behaviour: Alert (lost target / investigating) ---

    fn search_intent(&mut self, entity: &Entity, map: &Map, last_known: Point, search: SearchBehavior) -> Option<Intent> {
        #[cfg(debug_assertions)]
        puffin::profile_function!();
        match search {
            SearchBehavior::HoldAndWatch => None, // stand still, weapon ready
            SearchBehavior::MoveToLastKnown => self.navigate_to(entity, last_known, map, 0),
            SearchBehavior::Flank => {
                let flank_dest = self.flank_destination(entity.position, last_known, map);
                self.navigate_to(entity, flank_dest, map, 0)
            },
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

    fn combat_intent(
        &mut self,
        entity:    &Entity,
        map:       &Map,
        entities:  &[Entity],
        target_id: usize,
        last_seen: Point,
    ) -> Option<Intent> {
        #[cfg(debug_assertions)]
        puffin::profile_function!();
        let tactic = self.profile.combat_tactic().clone();

        if let CombatTactic::Flee = tactic {
            let flee_pos = self.flee_pos(entity, last_seen, map);
            return self.navigate_to(entity, flee_pos, map, 0);
        }

        // Try melee if adjacent to target.
        if let Some(target) = entities.iter().find(|e| e.id == target_id) {
            let tc = target.center();
            if adjacent(entity.position, tc) {
                return Some(Intent {
                    phase:  ExecutionPhase::Attack,
                    data:   IntentData::Target(tc),
                    action: actions::melee_action,
                });
            }

            // Try ranged attack if weapon equipped and target in range.
            // Uses the same precondition system as the player menu: fire actions
            // require an active aim status, so the AI must spend a turn aiming first.
            if let Some((slot, range)) = find_weapon(entity) {
                let dist = rltk::DistanceAlg::Pythagoras.distance2d(entity.center(), tc);
                if dist <= range as f32 {
                    let available = player::get_entity_available_actions(entity, map);

                    // If a fire action is available (precondition_is_aiming passed), fire.
                    if let Some((fire_action, _)) = available.iter()
                        .find(|(a, s)| *s == Some(slot) && matches!(a.targeting, Targeting::UseExistingAim { .. }))
                    {
                        return Some(Intent {
                            phase:  ExecutionPhase::Attack,
                            data:   IntentData::TargetWithEquipment { slot, target: tc },
                            action: fire_action.action,
                        });
                    }

                    // Not yet aiming: spend this turn acquiring aim on the target.
                    if available.iter().any(|(a, s)| *s == Some(slot) && matches!(a.targeting, Targeting::EntityAim { .. })) {
                        return Some(Intent {
                            phase:  ExecutionPhase::Attack,
                            data:   IntentData::TargetWithEquipment { slot, target: tc },
                            action: actions::aim_action,
                        });
                    }
                }
            }
        }

        // No attack available — move or hold.
        match tactic {
            CombatTactic::Pursue => {
                let dest = entities.iter().find(|e| e.id == target_id)
                    .map(|t| t.center())
                    .unwrap_or(last_seen);
                self.navigate_to(entity, dest, map, 1)
            },
            CombatTactic::Hold => None,
            CombatTactic::Flee => unreachable!(),
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

    fn navigate_to(&mut self, entity: &Entity, destination: Point, map: &Map, tolerance: u32) -> Option<Intent> {
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
            Some(dir) if dir != entity.body.facing => Some(Intent {
                phase:  ExecutionPhase::Movement,
                data:   IntentData::Direction(dir),
                action: actions::turn_action,
            }),
            Some(_) => Some(Intent {
                phase:  ExecutionPhase::Movement,
                data:   IntentData::Target(next_pos),
                action: actions::move_action,
            }),
            None => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[derive(Copy, Clone)]
enum AlertSnapshot {
    Unaware,
    Suspicious { origin: Point },
    Alert      { last_known: Point, search: SearchBehavior },
    Combat     { target_id: usize, last_seen: Point },
}

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
    Intent {
        phase:  ExecutionPhase::Movement,
        data:   IntentData::Target(Point { x: pos.x + dx, y: pos.y + dy }),
        action: actions::move_action,
    }
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
    ) -> (Option<Intent>, Vec<SoundEvent>) {
        match self {
            AI::None => (None, vec![]),
            AI::Rotator => (Some(Intent {
                phase:  ExecutionPhase::Movement,
                data:   IntentData::Direction(entity.body.facing.clockwise()),
                action: actions::turn_action,
            }), vec![]),
            AI::Forward => (Some(forward_intent(entity.position, entity.body.facing)), vec![]),
            AI::Actor(actor) => actor.compute_intent(entity, map, entities, sounds),
        }
    }
}
