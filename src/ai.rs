use rltk::{Point, RandomNumberGenerator};
use crate::ActionSource::InventoryItem;
use crate::Map;
use crate::{navigate_cached, greedy_step};
use crate::Entity;
use crate::EntityKind;
use crate::DrivingState;
use crate::util::adjacent;
use crate::components::*;
use crate::intent::*;
use crate::Ability;
use crate::Item;

const SUSPICIOUS_TURNS: u32 = 30;

// --- Detection tunables: sighting the player fills a meter instead of alerting at once ---
/// Meter value at which a sighting becomes a confirmed hostile.
const DETECT_THRESHOLD: u8 = 6;
/// Within this distance the player is recognised immediately.
const DETECT_INSTANT_RADIUS: f32 = 3.0;
/// Per-turn gain by distance band: (max distance, gain). Beyond the last band: 1.
const DETECT_GAIN_BANDS: [(f32, u8); 2] = [(8.0, 3), (14.0, 2)];

// --- Guard tunables (placeholder behaviour, safe to retune) ---
/// A guard within this Pythagorean distance of its anchor is "near" its post and
/// mills about; beyond it, it heads back.
const NEAR_ANCHOR_RADIUS: f32 = 3.0;
/// Unaware guard at its anchor: idle this often (percent), else turn a random way.
const UNAWARE_IDLE_PCT: u32 = 95;
/// Alert guard near its anchor: rotate below the first cut, step forward below the
/// second, else idle. Rolls share one 0..100 draw.
const ALERT_ROTATE_PCT: u32 = 50;
const ALERT_STEP_PCT:   u32 = 75;
/// A patroller counts a waypoint it can see within this many tiles as reached.
const WAYPOINT_REACH: i32 = 3;
/// A pilot investigates suspicion only this far from its post beside the tank.
const PILOT_LEASH: f32 = 6.0;
/// Radius a guard searches for a tile from which it can re-spot a lost target.
const SPOT_SEARCH: i32 = 3;
/// How far a grenade can be thrown. Mirrors `Item::throw_action`'s max range.
const GRENADE_THROW_RANGE: u32 = 5;
/// A guard flees a live grenade whose blast radius reaches within this margin.
const GRENADE_FLEE_MARGIN: f32 = 2.0;

// --- Patrol search tunables (Alert state) ---
/// A searching patroller re-raises the alarm every this many turns.
const SHOUT_INTERVAL: u32 = 8;
/// Search sweep starts this far from the last-known position and grows outward.
const SEARCH_START_RADIUS: i32 = 2;
const SEARCH_MAX_RADIUS:   i32 = 10;
/// The sweep radius grows by one every this many turns.
const SEARCH_GROW_EVERY: u32 = 6;
/// Turns a searcher commits to one sweep heading before rotating to the next.
const SEARCH_DIR_HOLD: u32 = 3;

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

impl Perception {
    /// Urgency key, compared lexicographically (higher wins): confirmed-hostile
    /// outranks unconfirmed, then seen outranks heard, then known-origin outranks
    /// unknown.
    fn urgency(&self) -> (bool, bool, bool) {
        (self.confirmed_hostile, self.confirmed_visually, self.confirmed_origin)
    }
}

/// A door the player left open, if in view: someone passed through here.
fn notice_open_door(entity: &Entity, doors: &[Point]) -> Option<Perception> {
    let range = entity.viewshed.range as f32;
    doors.iter()
        .find(|&&door| rltk::DistanceAlg::Pythagoras.distance2d(entity.center(), door) <= range && entity.can_see(door))
        .map(|&origin| Perception {
            confirmed_hostile: false,
            confirmed_origin: false,
            confirmed_visually: false,
            origin,
            target_id: None
        })
}

/// The more urgent of two perceptions, biased toward `left` on a tie.
fn most_urgent(left: Option<Perception>, right: Option<Perception>) -> Option<Perception> {
    match (left, right) {
        (Some(l), Some(r)) => Some(if r.urgency() > l.urgency() { r } else { l }),
        (l, r) => l.or(r),
    }
}

// ---------------------------------------------------------------------------
// Decision
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum Decision {
    Idle,
    /// Holster the equipped weapon (relaxed guard at its post).
    Holster,
    /// Turn to a fixed facing (random-look / orient leaves).
    Turn   { dir: Direction },
    /// Raise the alarm (a loud shout that alerts nearby guards).
    Shout,
    /// Throw the carried explosive `item_id` at `target`.
    ThrowGrenade { item_id: usize, target: Point },
    /// Prime the carried explosive `item_id` (thrown next turn by the Always block).
    PrimeGrenade { item_id: usize },
    GetReadyForCombat,
    GoTo   { dest: Point, tolerance: u32 },
    Face   { toward: Point },
    Flee   { threat: Point },
    Engage { target_id: usize, last_seen: Point },
    /// Walk to the tank `vehicle_id` and climb in.
    Board  { vehicle_id: usize },
}

// ---------------------------------------------------------------------------
// AlertLevel
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize)]
pub enum AlertLevel {
    /// Not acting on any threats
    Unaware,
    /// Has detected something potentially dangerous, but unconfirmed. Decays to Unaware.
    Suspicious { origin: Point, turns_remaining: u32 },
    /// Has detected something confirmed dangerous, but does not see it. Does not decay.
    /// `search_ticks` counts turns spent searching — drives the widening sweep and
    /// the periodic re-alarm (patrol), and is ignored by profiles that don't search.
    Alert      { last_known: Point, search_ticks: u32 },
    /// Has detected something confirmed dangerous and has recently seen it or is seeing it now.
    Combat     { target_id: usize, last_seen: Point },
}

// ---------------------------------------------------------------------------
// CombatTactic
// ---------------------------------------------------------------------------

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum CombatTactic {
    Pursue,
    Hold,
    Flee,
}

// ---------------------------------------------------------------------------
// Profile
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Profile {
    Patrol {
        /// Index into [`Map::patrol_routes`] — the shared, read-only route this
        /// actor follows.
        route_id: usize,
        waypoint_index: usize,
        combat_tactic: CombatTactic,
    },
    Guard {
        anchor: Point,
        combat_tactic: CombatTactic,
    },
    /// Crew of one tank (entity index; None once destroyed). `anchor` is its post
    /// beside the parked tank.
    Pilot {
        vehicle: Option<usize>,
        anchor: Point,
    },
}

impl Profile {
    fn combat_tactic(&self) -> &CombatTactic {
        match self {
            Profile::Patrol    { combat_tactic, .. } => combat_tactic,
            Profile::Guard     { combat_tactic, .. } => combat_tactic,
            Profile::Pilot     { .. }                => &CombatTactic::Pursue,
        }
    }

    /// The tank this pilot crews, for index remapping.
    pub fn vehicle_mut(&mut self) -> Option<&mut Option<usize>> {
        match self {
            Profile::Pilot { vehicle, .. } => Some(vehicle),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// ActorAI
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ActorAI {
    pub profile: Profile,
    pub alert:   AlertLevel,
    // Shared path cache — destination tracked to avoid redundant A* calls.
    current_path: Vec<usize>,    // reversed; .last() = next step index
    path_target:  Option<usize>, // map idx of current destination
    /// Per-actor RNG for the probabilistic idle/look leaves. Seeded lazily from the
    /// entity index (each actor owns its own stream, so the parallel AI pass never
    /// shares a generator). Not saved: a loaded game reseeds it lazily.
    #[serde(skip)]
    rng:          Option<RandomNumberGenerator>,
    /// How well the player has been recognised, 0..=DETECT_THRESHOLD. Fills while the
    /// player is in view, drains out of view; full = confirmed hostile.
    detection:    u8,
    /// The tile our last move was cancelled at because someone else stepped for it too.
    /// Set by World, consumed next turn to break the tie (see `react_to_bump`).
    #[serde(default)]
    bumped_at:    Option<Point>,
}

impl ActorAI {
    pub fn new(profile: Profile) -> Self {
        ActorAI { profile, alert: AlertLevel::Unaware, current_path: vec![], path_target: None, rng: None, detection: 0, bumped_at: None }
    }

    /// Our move into `tile` was cancelled because another actor stepped for it too.
    pub fn note_bumped(&mut self, tile: Point) {
        self.bumped_at = Some(tile);
    }

    /// Recognition progress in 0.0..=1.0, for the UI.
    pub fn detection_level(&self) -> f32 {
        self.detection as f32 / DETECT_THRESHOLD as f32
    }

    /// Follows a perceive → update → decide → execute logic for easier overview.
    pub fn compute_intent(
        &mut self,
        entity:   &Entity,
        map:      &Map,
        entities: &[Entity],
        sounds:   &[SoundEvent],
        grenades: &[(Point, u32)],
        doors:    &[Point],
    ) -> Option<Intent> {
        #[cfg(debug_assertions)]
        puffin::profile_function!();

        // Perceive: collect this turn stimuli and return perception.
        let perception = self.perceive(entity, entities, map, sounds, doors);

        // Update beliefs according to perceptions
        self.update_beliefs(entity, entities, map, perception);

        // Make a decision
        self.advance_waypoint(entity, map); // Move this later
        // Roll once up front (in this &mut context) so `decide` stays pure.
        let (roll, rand_dir, bump_roll) = {
            let rng = self.rng.get_or_insert_with(|| RandomNumberGenerator::seeded(scramble(entity.index as u64)));
            (rng.range(0, 100) as u32, Direction::ALL[rng.range(0, 8) as usize], rng.range(0, 3) as u32)
        };
        let decision = self.decide(entity, map, entities, grenades, roll, rand_dir);

        // Execute the decision
        let mut intent = self.execute(entity, map, entities, decision);
        if let Some(tile) = self.bumped_at.take() {
            intent = self.react_to_bump(entity, map, entities, intent, tile, bump_roll, rand_dir);
        }

        // Advance the search clock for next turn (drives sweep growth + re-alarm).
        if let AlertLevel::Alert { search_ticks, .. } = &mut self.alert {
            *search_ticks += 1;
        }
        intent
    }

    /// Last turn someone else stepped for the same tile and both moves were cancelled.
    /// Deciding the same way again would repeat the stand-off forever, so, like two
    /// people walking into each other, roll: try again, wait a beat, or sidestep to a
    /// random free tile other than the contested one. Non-move intents pass through.
    fn react_to_bump(&mut self, entity: &Entity, map: &Map, entities: &[Entity], intent: Option<Intent>,
                     contested: Point, roll: u32, rand_dir: Direction) -> Option<Intent> {
        let is_move = intent.as_ref().is_some_and(|i| std::ptr::fn_addr_eq(i.action, crate::actions::move_action as crate::actions::Action));
        if !is_move {
            return intent;
        }
        match roll {
            0 => intent,
            1 => None,
            _ => {
                let start = Direction::ALL.iter().position(|&d| d == rand_dir).unwrap_or(0);
                let dir = (0..8).map(|k| Direction::ALL[(start + k) % 8]).find(|dir| {
                    let (dx, dy) = dir.delta_pos();
                    let tile = Point { x: entity.position.x + dx, y: entity.position.y + dy };
                    tile != contested && entity.check_fit(tile, map)
                })?;
                // We leave the cached path; repath from wherever the sidestep lands.
                self.path_target = None;
                resolve_step(entity, dir, map, entities).ok().flatten()
            },
        }
    }

    // --- Stimulus processing ---

    /// Returns the most important Perception, if any, to be acted upon later
    fn perceive(&mut self, entity: &Entity, entities: &[Entity], map: &Map, sounds: &[SoundEvent], doors: &[Point]) -> Option<Perception> {
        let sound_candidate = self.process_sounds(entity, sounds);
        let visual_candidate = self.process_vision(entity, entities, map);
        let door_candidate = notice_open_door(entity, doors);

        most_urgent(most_urgent(visual_candidate, sound_candidate), door_candidate)
    }

    fn process_sounds(&mut self, entity: &Entity, sounds: &[SoundEvent]) -> Option<Perception> {
        #[cfg(debug_assertions)]
        puffin::profile_function!();

        let mut retval: Option<Perception> = None;

        for s in sounds {
            // Guards tell the player's footsteps from their comrades' (deliberately unrealistic).
            if s.kind == SoundKind::Footstep && !s.from_player {
                continue;
            }
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
            
        // Look for both the player and a body; neither may cut the scan short for the other.
        let mut player: Option<(usize, Point)> = None;
        let mut corpse: Option<Point> = None;
        for point in &entity.viewshed.visible_tiles {
            if player.is_none() {
                if let Some(entity_id) = map.get_entity_id(point.x, point.y) {
                    if entities[entity_id].kind == EntityKind::Player {
                        player = Some((entity_id, entities[entity_id].center()));
                    }
                }
            }
            if corpse.is_none() {
                if matches!(map.get_item_ref(point.x, point.y), Some(item) if item.kind == ItemKind::Corpse) {
                    corpse = Some(*point);
                }
            }
            if player.is_some() && corpse.is_some() {
                break;
            }
        }

        // A body is proof of a threat, though not of where it went.
        let body = corpse.map(|origin| Perception {
            confirmed_hostile: true,
            confirmed_origin: false,
            confirmed_visually: false,
            origin,
            target_id: None
        });
        most_urgent(self.register_sighting(entity, player), body)
    }

    /// Fills the detection meter while the player is in view and drains it otherwise.
    /// Below full, a sighting is only a glimpse: seen, but not yet recognised as hostile.
    fn register_sighting(&mut self, entity: &Entity, seen: Option<(usize, Point)>) -> Option<Perception> {
        let Some((target_id, origin)) = seen else {
            self.detection = self.detection.saturating_sub(1);
            return None;
        };
        let dist = rltk::DistanceAlg::Pythagoras.distance2d(entity.center(), origin);
        let mut gain = if dist <= DETECT_INSTANT_RADIUS {
            DETECT_THRESHOLD
        } else {
            DETECT_GAIN_BANDS.iter().find(|(max, _)| dist <= *max).map_or(1, |(_, g)| *g)
        };
        match self.alert {
            AlertLevel::Combat { .. } => gain = DETECT_THRESHOLD, // already knows exactly who it's fighting
            AlertLevel::Alert { .. }  => gain *= 2,               // primed by a confirmed threat
            _ => {}
        }
        self.detection = self.detection.saturating_add(gain).min(DETECT_THRESHOLD);
        Some(Perception {
            confirmed_hostile: self.detection >= DETECT_THRESHOLD,
            confirmed_origin: true,
            confirmed_visually: true,
            origin,
            target_id: Some(target_id)
        })
    }

    // --- Belief updates ---

    fn update_beliefs(&mut self, entity: &Entity, entities: &[Entity], map: &Map, perception: Option<Perception>) {
        self.decay_alertness(entity, entities, map);
        if let Some(p) = perception {
            self.update_target(p);
        }
    }

    fn update_target(&mut self, perception: Perception) {
        // Already engaging: a fresh sighting refreshes the target's position;
        // lesser stimuli (noise, corpses) are ignored so combat isn't pulled off.
        if let AlertLevel::Combat { target_id, last_seen } = &mut self.alert {
            if perception.confirmed_visually && perception.confirmed_hostile {
                if let Some(id) = perception.target_id {
                    *target_id = id;
                    *last_seen = perception.origin;
                }
            }
            return;
        }
        // A confirmed sighting (re)enters combat from any lower state.
        if perception.confirmed_visually && perception.confirmed_hostile {
            // TODO: Handle that unwrap more gracefully? Potentially replace confirmed_visually and rely on this?
            self.alert = AlertLevel::Combat { target_id: perception.target_id.unwrap(), last_seen: perception.origin };
            return;
        }
        // Already searching: a comrade's shout or distant noise adds nothing — keep
        // the current search rather than restarting its clock (else shouts feed back
        // into each other and every guard re-alarms every turn).
        if matches!(self.alert, AlertLevel::Alert { .. }) {
            return;
        }
        // Unaware/Suspicious escalate on the stimulus.
        if perception.confirmed_hostile {
            // TODO: This will cause the AI to get hung up on dead bodies. Handle this with a SearchTheArea behavior.
            self.alert = AlertLevel::Alert { last_known: perception.origin, search_ticks: 0 };
        } else {
            self.alert = AlertLevel::Suspicious { origin: perception.origin, turns_remaining: SUSPICIOUS_TURNS };
        }
    }

    fn decay_alertness(&mut self, entity: &Entity, entities: &[Entity], map: &Map) {
        #[cfg(debug_assertions)]
        puffin::profile_function!();
        // Combat fallback. A guard is sticky: it holds the engagement through lost
        // sight and only drops to Alert once driven off its post with no way left to
        // reacquire. Other profiles fall back the moment the target breaks sight.
        if let AlertLevel::Combat { target_id, last_seen } = &self.alert {
            let (tid, ls) = (*target_id, *last_seen);
            let lost = !self.can_see_target(entity, entities, tid);
            let give_up = match &self.profile {
                Profile::Guard { .. } =>
                    lost && self.far_from_anchor(entity.position)
                        && !entity.can_see(ls) && !self.can_turn_to_see(entity, map, ls),
                // Patrol pursues hard: only breaks off (to shout + search) once it has
                // reached where the enemy was last seen and still can't find them.
                _ => lost && rltk::DistanceAlg::Pythagoras.distance2d(entity.center(), ls) <= 1.5,
            };
            if give_up {
                self.alert = AlertLevel::Alert { last_known: ls, search_ticks: 0 };
            }
            return;
        }

        // A guard won't chase a mere hunch away from its post: drop suspicion once
        // it strays past its anchor leash.
        if matches!(self.alert, AlertLevel::Suspicious { .. }) && self.far_from_anchor(entity.position) {
            self.alert = AlertLevel::Unaware;
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

    /// Advance a patroller to its next waypoint once it reaches the current one: stands
    /// on it, or is within WAYPOINT_REACH of it with it in sight (another guard may be
    /// standing on it). The navigation target itself stays the exact waypoint tile.
    fn advance_waypoint(&mut self, entity: &Entity, map: &Map) {
        if !matches!(self.alert, AlertLevel::Unaware) { return; }
        if let Profile::Patrol { route_id, waypoint_index, .. } = &mut self.profile {
            if let Some(route) = map.patrol_routes.get(*route_id) {
                let reached = |wp: Point| {
                    let pos = entity.position;
                    wp == pos || ((wp.x - pos.x).abs().max((wp.y - pos.y).abs()) <= WAYPOINT_REACH && entity.can_see(wp))
                };
                if !route.is_empty() && reached(route[*waypoint_index]) {
                    *waypoint_index = (*waypoint_index + 1) % route.len();
                    self.path_target = None;
                }
            }
        }
    }

    /// The decision tree: current (alert, profile) state → a Decision. Pure, and
    /// every Decision field is Copy, so no borrow of self outlives the call and
    /// execute can then take &mut self freely.
    fn decide(&self, entity: &Entity, map: &Map, entities: &[Entity], grenades: &[(Point, u32)], roll: u32, rand_dir: Direction) -> Decision {
        let pos = entity.position;
        // "Always": grenades trump every alert state (both guard profiles).
        if let Some(d) = self.grenade_reaction(entity, map, grenades) {
            return d;
        }
        if let Some(d) = self.pilot_decision(entity, map, entities) {
            return d;
        }
        match &self.alert {
            AlertLevel::Unaware => match &self.profile {
                // A patroller holsters, then walks its route.
                Profile::Patrol { route_id, waypoint_index, .. } =>
                    if entity.get_primary_weapon().is_some() {
                        Decision::Holster
                    } else {
                        match map.patrol_routes.get(*route_id).and_then(|r| r.get(*waypoint_index)) {
                            Some(&dest) => Decision::GoTo { dest, tolerance: 0 },
                            None        => Decision::Idle,
                        }
                    },
                // At its post a relaxed guard (or a pilot by its tank) holsters, then
                // mostly stands watch, glancing around now and then.
                Profile::Guard { anchor, .. } | Profile::Pilot { anchor, .. } =>
                    if pos != *anchor {
                        Decision::GoTo { dest: *anchor, tolerance: 0 }
                    } else if entity.get_primary_weapon().is_some() {
                        Decision::Holster
                    } else if roll < UNAWARE_IDLE_PCT {
                        Decision::Idle
                    } else {
                        Decision::Turn { dir: rand_dir }
                    },
            },
            AlertLevel::Suspicious { origin, .. } => {
                if !self.is_combat_ready(entity) {
                    Decision::GetReadyForCombat
                } else if self.far_from_anchor(pos) {
                    // Won't chase a hunch off its post — head back.
                    Decision::GoTo { dest: self.anchor().unwrap_or(*origin), tolerance: 0 }
                } else {
                    Decision::GoTo { dest: *origin, tolerance: 0 }
                }
            },
            AlertLevel::Alert { last_known, search_ticks } => {
                if !self.is_combat_ready(entity) {
                    Decision::GetReadyForCombat
                } else {
                    match &self.profile {
                        // A guard never forgets a confirmed threat, but holds near its
                        // post rather than searching: mostly watch, sometimes shift.
                        Profile::Guard { anchor, .. } =>
                            if self.far_from_anchor(pos) {
                                Decision::GoTo { dest: *anchor, tolerance: 0 }
                            } else if roll < ALERT_ROTATE_PCT {
                                Decision::Turn { dir: rand_dir }
                            } else if roll < ALERT_STEP_PCT {
                                Decision::GoTo { dest: forward_tile(pos, entity.body.facing), tolerance: 0 }
                            } else {
                                Decision::Idle
                            },
                        // A patroller sweeps a widening area around the last-known spot,
                        // re-raising the alarm on a timer to draw comrades in.
                        Profile::Patrol { .. } =>
                            if *search_ticks % SHOUT_INTERVAL == 0 {
                                Decision::Shout
                            } else {
                                Decision::GoTo { dest: patrol_search_target(*last_known, *search_ticks, map), tolerance: 0 }
                        },
                        // A pilot whose tank is gone searches like a patroller, but silently.
                        Profile::Pilot { .. } =>
                            Decision::GoTo { dest: patrol_search_target(*last_known, *search_ticks, map), tolerance: 0 },
                    }
                }
            },
            AlertLevel::Combat { target_id, last_seen } => match self.profile.combat_tactic() {
                CombatTactic::Flee => Decision::Flee { threat: *last_seen },
                _ => match &self.profile {
                    Profile::Guard { .. } =>
                        self.guard_combat(entity, map, entities, *target_id, *last_seen),
                    Profile::Patrol { .. } | Profile::Pilot { .. } =>
                        self.patrol_combat(entity, map, entities, *target_id, *last_seen),
                },
            },
        }
    }

    /// Carry out a Decision, producing a concrete intent.
    fn execute(&mut self, entity: &Entity, map: &Map, entities: &[Entity], decision: Decision) -> Option<Intent> {
        match decision {
            Decision::Idle => None,
            Decision::Holster => holster_intent(entity),
            Decision::Turn { dir } => (entity.body.facing != dir).then(|| turn_toward_intent(entity, dir)),
            Decision::Shout => shout_intent(entity),
            Decision::ThrowGrenade { item_id, target } => throw_grenade_intent(entity, item_id, target),
            Decision::PrimeGrenade { item_id } => prime_grenade_intent(entity, item_id),
            Decision::GetReadyForCombat => self.get_ready_for_combat(entity, map),
            Decision::GoTo { dest, tolerance } => self.navigate_to(entity, dest, map, entities, tolerance),
            Decision::Face { toward } => face_intent(entity, toward),
            Decision::Flee { threat } => {
                let dest = self.flee_pos(entity, threat, map);
                self.navigate_to(entity, dest, map, entities, 0)
            },
            Decision::Engage { target_id, last_seen } =>
                self.engage(entity, map, entities, target_id, last_seen),
            Decision::Board { vehicle_id } => self.board(entity, map, entities, vehicle_id),
        }
    }

    // --- Behaviour: Pilot (doc/ai.md "Pilot") ---

    /// The pilot's own branches: fighting from the tank, or running to board it on a
    /// confirmed threat. None defers to the shared tree (relaxed or suspicious by the
    /// tank, or fighting on foot once the tank is gone).
    fn pilot_decision(&self, entity: &Entity, map: &Map, entities: &[Entity]) -> Option<Decision> {
        let Profile::Pilot { vehicle, .. } = &self.profile else { return None };
        let driving = matches!(entity.driving, DrivingState::DrivenBy(_));
        match &self.alert {
            // Searching from the tank: silent, and the engine noise draws the curious.
            AlertLevel::Alert { last_known, search_ticks } if driving =>
                Some(Decision::GoTo { dest: patrol_search_target(*last_known, *search_ticks, map), tolerance: 0 }),
            AlertLevel::Combat { target_id, last_seen } if driving =>
                Some(self.tank_combat(entity, map, entities, *target_id, *last_seen)),
            AlertLevel::Alert { .. } | AlertLevel::Combat { .. } => {
                let tank_id = vehicle.filter(|&v| entities.get(v).is_some_and(|t| t.driving == DrivingState::Drivable))?;
                if let AlertLevel::Combat { target_id, last_seen } = &self.alert {
                    if entities.get(*target_id).is_some_and(|t| adjacent(entity.position, t.center())) {
                        return Some(Decision::Engage { target_id: *target_id, last_seen: *last_seen });
                    }
                }
                Some(Decision::Board { vehicle_id: tank_id })
            },
            _ => None,
        }
    }

    /// Combat from a tank: shoot on sight, else swing the turret toward the last
    /// sighting, else drive there. Falling back to Alert happens in `decay_alertness`.
    fn tank_combat(&self, entity: &Entity, map: &Map, entities: &[Entity], target_id: usize, last_seen: Point) -> Decision {
        if self.is_combat_ready(entity) && self.can_see_target(entity, entities, target_id) {
            return Decision::Engage { target_id, last_seen };
        }
        if !entity.can_see(last_seen) && self.can_turn_to_see(entity, map, last_seen) {
            return Decision::Face { toward: last_seen };
        }
        Decision::GoTo { dest: last_seen, tolerance: 0 }
    }

    /// Climb into the tank when beside it, else walk to the nearest free tile next to it.
    fn board(&mut self, entity: &Entity, map: &Map, entities: &[Entity], vehicle_id: usize) -> Option<Intent> {
        let tank = entities.get(vehicle_id)?;
        if let Some(tile) = footprint(tank).find(|&t| adjacent(entity.position, t)) {
            let dir = direction_to(entity.position, tile)?;
            return resolve_step(entity, dir, map, entities).ok().flatten();
        }
        let spot = footprint_ring(tank)
            .filter(|p| p.x >= 0 && p.y >= 0 && p.x < map.width as i32 && p.y < map.height as i32 && !map.blocked(p.x, p.y))
            .min_by_key(|&p| sq_dist(p, entity.position))?;
        self.navigate_to(entity, spot, map, entities, 0)
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
            // Vehicles can't melee; they shoot at any range.
            if adjacent(entity.position, tc) && !entity.has_ability(Ability::VehicleMove) {
                return match direction_to(entity.position, tc) {
                    Some(dir) => resolve_step(entity, dir, map, entities).ok().flatten(),
                    None      => Some(melee_intent(tc)),
                };
            }

            // Ranged: fire if aim is ready, else spend the turn acquiring it
            // (fire actions require an active aim status, same as the player menu).
            if let Some((slot, range, damage)) = find_weapon(entity) {
                let dist = rltk::DistanceAlg::Pythagoras.distance2d(entity.center(), tc);
                if dist <= range as f32 {
                    let available = entity.get_available_actions(map);
                    let fire = available.iter().find(|(a, s)| *s == Some(slot) && matches!(a.targeting, Targeting::UseExistingAim { .. }));
                    let aim  = available.iter().find(|(a, s)| *s == Some(slot) && matches!(a.targeting, Targeting::EntityAim { .. }));
                    if let Some(&(action, _)) = fire.or(aim) {
                        // Only the fire step carries a body part; aim-acquire has no target part yet.
                        let resolution = if fire.is_some() {
                            Resolution::Bodypart { target: tc, bodypart_index: aim_bodypart(target, damage) }
                        } else {
                            Resolution::Position(tc)
                        };
                        return Some(build_intent(action, Some(ActionSource::EquippedSlot(slot)), resolution));
                    }
                }
            }
        }

        // No shot available — close on the target (or its last-seen tile) to gain
        // range and line of sight. Flee is routed upstream to Decision::Flee.
        let dest = entities.iter().find(|e| e.index == target_id)
            .map(|t| t.center()).unwrap_or(last_seen);
        self.navigate_to(entity, dest, map, entities, 1)
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
        if entity.has_ability(Ability::VehicleMove) {
            let dir = vehicle_step(entity, destination, map)?;
            return resolve_step(entity, dir, map, entities).ok().flatten();
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
        } else {
            self.astar_step(from_idx, dest_idx, map, tolerance)
        }?;

        match direction_to(entity.position, next_pos) {
            Some(dir) => resolve_step(entity, dir, map, entities).ok().flatten(),
            None => None,
        }
    }

    /// This actor's post, if it holds one (guards and pilots).
    fn anchor(&self) -> Option<Point> {
        match &self.profile {
            Profile::Guard { anchor, .. } | Profile::Pilot { anchor, .. } => Some(*anchor),
            _ => None,
        }
    }

    /// Whether `pos` lies beyond the actor's leash around its post. False for
    /// actors without a post.
    fn far_from_anchor(&self, pos: Point) -> bool {
        let leash = if matches!(self.profile, Profile::Pilot { .. }) { PILOT_LEASH } else { NEAR_ANCHOR_RADIUS };
        self.anchor().map_or(false, |a| rltk::DistanceAlg::Pythagoras.distance2d(pos, a) > leash)
    }

    /// Reload the equipped weapon if that action is available (its precondition
    /// already vets ammo + capacity), otherwise equip a loaded weapon from
    /// inventory. None if neither is possible.
    fn get_ready_for_combat(&self, entity: &Entity, map: &Map) -> Option<Intent> {
        for (action, slot) in entity.get_available_actions(map) {
            if let (ActionId::Reload, Some(s)) = (action.id, slot) {
                return Some(build_intent(action, Some(ActionSource::EquippedSlot(s)), Resolution::None));
            }
        }
        let weapons = self.equippable_weapons(entity, map);
        weapons.first().map(|(action, item)| build_intent(action, Some(InventoryItem((*item).clone())), Resolution::None))
    }

    /// The "Always" grenade block: get rid of a grenade we've primed (ideally onto
    /// the enemy), else run from a live grenade about to go off nearby. None when
    /// no grenade concerns us this turn.
    fn grenade_reaction(&self, entity: &Entity, map: &Map, grenades: &[(Point, u32)]) -> Option<Decision> {
        if let Some((item_id, radius)) = primed_grenade(entity) {
            return Some(self.throw_primed(entity, map, item_id, radius, self.known_enemy()));
        }
        if let Some(threat) = nearest_grenade(entity.position, grenades) {
            return Some(Decision::Flee { threat });
        }
        None
    }

    /// The confirmed enemy position the guard would aim a grenade at, if any.
    fn known_enemy(&self) -> Option<Point> {
        match &self.alert {
            AlertLevel::Combat { last_seen, .. } => Some(*last_seen),
            AlertLevel::Alert  { last_known, .. } => Some(*last_known),
            _ => None,
        }
    }

    /// Where to lob an already-primed grenade of blast `radius`: at the enemy if it's
    /// in range and the blast won't reach us, otherwise the best safe tile (nearest
    /// the enemy if known, else farthest from us). Flee our own blast if we can't
    /// throw it clear at all.
    fn throw_primed(&self, entity: &Entity, map: &Map, item_id: usize, radius: u32, target: Option<Point>) -> Decision {
        let (from, center) = (entity.position, entity.center());
        if let Some(t) = target {
            let in_range = rltk::DistanceAlg::Pythagoras.distance2d(center, t) <= GRENADE_THROW_RANGE as f32;
            if in_range && has_los(center, t, map) {
                return Decision::ThrowGrenade { item_id, target: t };
            }
        }
        match self.grenade_spot(map, from, radius, target) {
            Some(spot) => Decision::ThrowGrenade { item_id, target: spot },
            None       => Decision::Flee { threat: from },
        }
    }

    /// Best in-throw-range tile for a grenade of blast `radius`: clear line of sight
    /// and beyond our own blast, scored nearest the enemy (`toward`) if known else
    /// farthest from us. Bounded to the throw range, and only reached while holding a
    /// primed grenade, so the O(area) sweep stays off the general hot path.
    fn grenade_spot(&self, map: &Map, from: Point, radius: u32, toward: Option<Point>) -> Option<Point> {
        let r = GRENADE_THROW_RANGE as i32;
        let mut best: Option<(Point, i32)> = None;
        for dy in -r..=r {
            for dx in -r..=r {
                let p = Point { x: from.x + dx, y: from.y + dy };
                if p == from { continue; }
                if p.x < 0 || p.y < 0 || p.x >= map.width as i32 || p.y >= map.height as i32 { continue; }
                if rltk::DistanceAlg::Pythagoras.distance2d(from, p) > GRENADE_THROW_RANGE as f32 { continue; }
                if rltk::DistanceAlg::Pythagoras.distance2d(from, p) <= radius as f32 { continue; }
                if !has_los(from, p, map) { continue; }
                let score = match toward {
                    Some(t) => -sq_dist(p, t), // nearest the enemy
                    None    =>  sq_dist(p, from), // farthest from us
                };
                if best.map_or(true, |(_, bs)| score > bs) { best = Some((p, score)); }
            }
        }
        best.map(|(p, _)| p)
    }

    /// The sentinel guard's combat reacquire ladder (see doc/ai.md): see → attack,
    /// last-known in view → close half the gap, could-see-by-turning → turn, else
    /// reposition for a shot or hold and watch. Giving up far from post is handled
    /// upstream by `decay_alertness`, so reaching the tail means we're near our post.
    fn guard_combat(&self, entity: &Entity, map: &Map, entities: &[Entity], target_id: usize, last_seen: Point) -> Decision {
        // Prefer a grenade: prime one whenever the enemy is within throw range and we
        // have a clear lob to it. The Always block throws it next turn.
        if let Some(item_id) = carries_grenade(entity) {
            let center = entity.center();
            if rltk::DistanceAlg::Pythagoras.distance2d(center, last_seen) <= GRENADE_THROW_RANGE as f32
                && has_los(center, last_seen, map) {
                return Decision::PrimeGrenade { item_id };
            }
        }
        // The rest of the ladder needs a working firearm.
        if !self.is_combat_ready(entity) {
            return Decision::GetReadyForCombat;
        }
        if self.can_see_target(entity, entities, target_id) {
            return Decision::Engage { target_id, last_seen };
        }
        if entity.can_see(last_seen) {
            return Decision::GoTo { dest: halfway(entity.position, last_seen), tolerance: 0 };
        }
        if self.can_turn_to_see(entity, map, last_seen) {
            return Decision::Face { toward: last_seen };
        }
        match self.spot_to_see(entity, map, last_seen) {
            Some(spot) => Decision::GoTo { dest: spot, tolerance: 0 },
            None       => Decision::Face { toward: last_seen },
        }
    }

    /// The patroller's combat behaviour (doc/ai.md): prefer a grenade in throw
    /// range, shoot while the enemy is in sight, otherwise chase to where it was
    /// last seen. Breaking off to shout + search happens in `decay_alertness` once
    /// the guard reaches that spot and still can't find them.
    fn patrol_combat(&self, entity: &Entity, map: &Map, entities: &[Entity], target_id: usize, last_seen: Point) -> Decision {
        if let Some(item_id) = carries_grenade(entity) {
            let center = entity.center();
            if rltk::DistanceAlg::Pythagoras.distance2d(center, last_seen) <= GRENADE_THROW_RANGE as f32
                && has_los(center, last_seen, map) {
                return Decision::PrimeGrenade { item_id };
            }
        }
        if !self.is_combat_ready(entity) {
            return Decision::GetReadyForCombat;
        }
        if self.can_see_target(entity, entities, target_id) {
            return Decision::Engage { target_id, last_seen };
        }
        Decision::GoTo { dest: last_seen, tolerance: 0 }
    }

    /// Whether the entity with `target_id` currently sits in this actor's viewshed.
    fn can_see_target(&self, entity: &Entity, entities: &[Entity], target_id: usize) -> bool {
        entities.iter().find(|e| e.index == target_id)
            .map_or(false, |t| entity.can_see(t.center()))
    }

    /// True if `target` lies within vision range with a clear sight line — i.e. the
    /// actor would see it after turning to face it (vision is a facing cone, so
    /// facing is the only thing turning changes).
    fn can_turn_to_see(&self, entity: &Entity, map: &Map, target: Point) -> bool {
        let from = entity.center();
        rltk::DistanceAlg::Pythagoras.distance2d(from, target) <= entity.viewshed.range as f32
            && has_los(from, target, map)
    }

    /// The nearest free tile within `SPOT_SEARCH` that has a clear sight line to
    /// `target` in range. Bounded search — only a guard that lost sight near its
    /// post runs it, so the O(area) LOS sweep stays off the general hot path.
    fn spot_to_see(&self, entity: &Entity, map: &Map, target: Point) -> Option<Point> {
        let from = entity.position;
        let range = entity.viewshed.range as f32;
        let mut best: Option<(Point, i32)> = None;
        for dy in -SPOT_SEARCH..=SPOT_SEARCH {
            for dx in -SPOT_SEARCH..=SPOT_SEARCH {
                if dx == 0 && dy == 0 { continue; }
                let p = Point { x: from.x + dx, y: from.y + dy };
                if p.x < 0 || p.y < 0 || p.x >= map.width as i32 || p.y >= map.height as i32 { continue; }
                if map.blocked(p.x, p.y) { continue; }
                if rltk::DistanceAlg::Pythagoras.distance2d(p, target) > range { continue; }
                if !has_los(p, target, map) { continue; }
                let d = dx * dx + dy * dy;
                if best.map_or(true, |(_, bd)| d < bd) { best = Some((p, d)); }
            }
        }
        best.map(|(p, _)| p)
    }

    fn is_combat_ready(&self, entity: &Entity) -> bool {
        match entity.get_primary_weapon() {
            Some(weapon) => match weapon.kind {
                ItemKind::Firearm { ammo, .. } if ammo >= 1 => true,
                _ => false
            }
            None => false
        }
        // TODO: Consider melee
    }

    fn equippable_weapons<'a>(&self, entity: &'a Entity, map: &'a Map) -> Vec<(&'a EntityAction, &'a Item)> {
        let mut retval = vec!();
        for (action, item) in entity.get_available_inventory_actions(map) {
            match action.id {
                ActionId::Equip => {
                    match item.kind {
                        ItemKind::Firearm { ammo, .. } if ammo >= 1 => retval.push((action, item)),
                        _ => ()
                    }
                },
                _ => ()
            }
        }
        retval
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A resolved AI decision: produced by `decide`, carried out by `execute`.

/// Turn to face `toward` (any distance), or None if already facing it.
fn face_intent(entity: &Entity, toward: Point) -> Option<Intent> {
    match direction_toward(entity.center(), toward) {
        Some(dir) if entity.body.facing != dir => Some(turn_toward_intent(entity, dir)),
        _ => None,
    }
}

/// Every tile an entity occupies.
fn footprint(entity: &Entity) -> impl Iterator<Item = Point> + '_ {
    (0..entity.size_y as i32).flat_map(move |dy| (0..entity.size_x as i32)
        .map(move |dx| Point { x: entity.position.x + dx, y: entity.position.y + dy }))
}

/// The ring of tiles just outside an entity's footprint (may fall off the map).
fn footprint_ring(entity: &Entity) -> impl Iterator<Item = Point> + '_ {
    let (w, h) = (entity.size_x as i32, entity.size_y as i32);
    (-1..=h).flat_map(move |dy| (-1..=w).map(move |dx| (dx, dy)))
        .filter(move |&(dx, dy)| dx == -1 || dy == -1 || dx == w || dy == h)
        .map(move |(dx, dy)| Point { x: entity.position.x + dx, y: entity.position.y + dy })
}

/// Greedy step for a multi-tile vehicle: the direction whose move fits and brings its
/// center strictly closer to `dest`. The tile pathfinders assume 1×1 walkers (and see
/// the vehicle's own body as blocking), so vehicles don't use them; a vehicle with no
/// improving move waits.
fn vehicle_step(entity: &Entity, dest: Point, map: &Map) -> Option<Direction> {
    let center_offset = Point { x: entity.center().x - entity.position.x, y: entity.center().y - entity.position.y };
    let mut best = (sq_dist(entity.center(), dest), None);
    for dir in Direction::ALL {
        let (dx, dy) = dir.delta_pos();
        let pos = Point { x: entity.position.x + dx, y: entity.position.y + dy };
        if !entity.check_fit(pos, map) {
            continue;
        }
        let d = sq_dist(Point { x: pos.x + center_offset.x, y: pos.y + center_offset.y }, dest);
        if d < best.0 {
            best = (d, Some(dir));
        }
    }
    best.1
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

/// Returns the first equipped firearm with remaining ammo, its range, and its damage.
fn find_weapon(entity: &Entity) -> Option<(SlotType, u32, Damage)> {
    entity.body.item_slots.iter().find_map(|slot| {
        if let Some(item) = &slot.item {
            if let ItemKind::Firearm { ammo, range, damage, .. } = item.kind {
                if ammo > 0 { return Some((slot.slot_type, entity.firearm_range(range), damage)); }
            }
        }
        None
    })
}

/// Picks the target body part that takes the most damage from `weapon_damage`
/// (i.e. least armored against it). Ties resolve to the lowest index.
fn aim_bodypart(target: &Entity, weapon_damage: Damage) -> usize {
    let mut best_index = 0;
    let mut most_damage = 0;
    for (index, part) in target.body.parts.iter().enumerate() {
        let dealt = part.armor.modify_damage(weapon_damage);
        if dealt > most_damage {
            most_damage = dealt;
            best_index = index;
        }
    }
    best_index
}

fn forward_intent(pos: Point, facing: Direction) -> Intent {
    move_intent(forward_tile(pos, facing))
}

/// The tile one step ahead of `pos` when facing `dir`.
fn forward_tile(pos: Point, dir: Direction) -> Point {
    let (dx, dy) = dir.delta_pos();
    Point { x: pos.x + dx, y: pos.y + dy }
}

/// Integer midpoint between two tiles.
fn halfway(a: Point, b: Point) -> Point {
    Point { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 }
}

/// A widening search sweep target around `origin`: the heading rotates every
/// `SEARCH_DIR_HOLD` turns and the radius grows over time, so the searcher spirals
/// outward from the last-known position.
fn patrol_search_target(origin: Point, ticks: u32, map: &Map) -> Point {
    let radius = (SEARCH_START_RADIUS + (ticks / SEARCH_GROW_EVERY) as i32).min(SEARCH_MAX_RADIUS);
    let (dx, dy) = Direction::ALL[((ticks / SEARCH_DIR_HOLD) % 8) as usize].delta_pos();
    Point {
        x: (origin.x + dx * radius).clamp(0, map.width as i32 - 1),
        y: (origin.y + dy * radius).clamp(0, map.height as i32 - 1),
    }
}

/// Raise the alarm: a loud shout heard by nearby guards.
fn shout_intent(entity: &Entity) -> Option<Intent> {
    entity.has_ability(Ability::Shout).then(|| {
        build_intent(&shout_action_def(), None, Resolution::None)
    })
}

/// Squared tile distance (cheap ordering key, no sqrt).
fn sq_dist(a: Point, b: Point) -> i32 {
    let (dx, dy) = (a.x - b.x, a.y - b.y);
    dx * dx + dy * dy
}

/// (item_id, blast radius) of a primed explosive in the entity's inventory, if any.
fn primed_grenade(entity: &Entity) -> Option<(usize, u32)> {
    entity.body.inventory.iter().find_map(|i| match i.kind {
        ItemKind::FusedExplosive { radius, .. } if i.active => Some((i.id, radius)),
        _ => None,
    })
}

/// item_id of a throwable (un-primed) explosive in the entity's inventory, if any.
fn carries_grenade(entity: &Entity) -> Option<usize> {
    entity.body.inventory.iter().find_map(|i| match i.kind {
        ItemKind::FusedExplosive { .. } if !i.active => Some(i.id),
        _ => None,
    })
}

/// Prime the carried explosive `item_id`, via its own Prime action.
fn prime_grenade_intent(entity: &Entity, item_id: usize) -> Option<Intent> {
    let item = entity.body.inventory.iter().find(|i| i.id == item_id)?;
    let action = item.inventory_actions.iter().find(|a| a.id == ActionId::Prime)?;
    Some(build_intent(action, Some(ActionSource::InventoryItem(item.clone())), Resolution::None))
}

/// The nearest live grenade whose blast reaches `from` (within radius + margin).
fn nearest_grenade(from: Point, grenades: &[(Point, u32)]) -> Option<Point> {
    grenades.iter()
        .filter(|(pos, radius)| rltk::DistanceAlg::Pythagoras.distance2d(from, *pos)
            <= *radius as f32 + GRENADE_FLEE_MARGIN)
        .min_by_key(|(pos, _)| sq_dist(*pos, from))
        .map(|(pos, _)| *pos)
}

/// Throw the carried explosive `item_id` at `target`, via its own Throw action.
fn throw_grenade_intent(entity: &Entity, item_id: usize, target: Point) -> Option<Intent> {
    let item = entity.body.inventory.iter().find(|i| i.id == item_id)?;
    let action = item.inventory_actions.iter().find(|a| a.id == ActionId::Throw)?;
    Some(build_intent(action, Some(ActionSource::InventoryItem(item.clone())), Resolution::Position(target)))
}

/// Clear line of sight between two tiles: no opaque tile strictly between them.
fn has_los(from: Point, to: Point, map: &Map) -> bool {
    let ray = rltk::line2d(rltk::LineAlg::Bresenham, from, to);
    let n = ray.len();
    if n <= 2 { return true; }
    !ray[1..n - 1].iter().any(|p| map.is_opaque(map.pos_idx(*p)))
}

/// Unequip the entity's held primary weapon back to inventory, if any.
fn holster_intent(entity: &Entity) -> Option<Intent> {
    let slot = equipped_weapon_slot(entity)?;
    let unequip = unequip_action_def();
    Some(build_intent(&unequip, Some(ActionSource::EquippedSlot(slot)), Resolution::None))
}

/// The slot holding the entity's primary weapon (hand or turret), if equipped.
fn equipped_weapon_slot(entity: &Entity) -> Option<SlotType> {
    [SlotType::PrimaryHand, SlotType::TurretMount].iter().copied()
        .find(|&s| entity.body.get_item(s).is_some())
}

/// Scatter sequential entity indices into well-separated RNG seeds (splitmix64
/// finalizer) so neighbouring guards don't roll in lockstep.
fn scramble(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

// ---------------------------------------------------------------------------
// AI enum — public entry point
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize)]
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
        grenades: &[(Point, u32)],
        doors:    &[Point],
    ) -> Option<Intent> {
        match self {
            AI::None => None,
            AI::Rotator => Some(turn_intent(entity.body.facing.clockwise())),
            AI::Forward => Some(forward_intent(entity.position, entity.body.facing)),
            AI::Actor(actor) => actor.compute_intent(entity, map, entities, sounds, grenades, doors),
        }
    }
}
