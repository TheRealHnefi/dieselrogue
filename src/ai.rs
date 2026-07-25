use rltk::Point;
use crate::Body;
use crate::Map;
use crate::Entity;
use crate::util::*;
use crate::components::*;
use crate::intent::*;
use crate::actions;

pub enum AI {
    None,
    Rotator,
    Forward,
    Patrolling(PatrollingAI),
}

impl AI {
    /// Compute the next intent for this entity.
    /// Returns `None` for `AI::None` (entity keeps its existing intent).
    /// Takes an immutable view of all entities so future AI variants can
    /// react to other entities without requiring mutable world access.
    pub fn compute_intent(
        &mut self,
        entity: &Entity,
        map: &Map,
        entities: &[Entity],
        sounds: &[SoundEvent],
    ) -> Option<Intent> {
        let _ = entities; // not yet used; available for future AI variants
        let _ = sounds;   // not yet used; available for future AI variants
        match self {
            AI::None => None,
            AI::Rotator => Some(Intent {
                phase: ExecutionPhase::Movement,
                data: IntentData::Direction(entity.body.facing.clockwise()),
                action: actions::turn_action,
            }),
            AI::Forward => Some(forward_intent(entity.position, entity.body.facing)),
            AI::Patrolling(ai) => Some(ai.compute_intent(entity.position, &entity.body, map)),
        }
    }
}

fn forward_intent(pos: Point, facing: Direction) -> Intent {
    let (dx, dy) = facing.delta_pos();
    Intent {
        phase: ExecutionPhase::Movement,
        data: IntentData::Target(Point { x: pos.x + dx, y: pos.y + dy }),
        action: actions::move_action,
    }
}

pub struct PatrollingAI {
    waypoints: Vec<Point>,
    waypoint_index: usize,
    current_path: Vec<usize>,
}

impl PatrollingAI {
    pub fn new(waypoints: Vec<Point>) -> Self {
        Self {
            waypoints,
            waypoint_index: 0,
            current_path: vec![],
        }
    }

    pub fn compute_intent(&mut self, position: Point, body: &Body, map: &Map) -> Intent {
        // Consume the path step we're standing on, if any. This is how we detect that a
        // move succeeded: the entity's position now matches what we declared last tick.
        //
        // NOTE — one-tick commit: an entity is committed to its declared target for one full
        // tick. If a move is blocked (contested or check_fit fails), the same step is retried
        // next tick rather than reconsidered mid-tick. Future AI variants that need to abort
        // or redirect a move mid-cycle will need a different mechanism (e.g. a post-resolution
        // callback, or storing the last declared target and comparing against it here).
        if let Some(&next_idx) = self.current_path.last() {
            if map.idx_pos(next_idx) == position {
                self.current_path.pop();
            }
        }

        if self.waypoints[self.waypoint_index] == position {
            self.waypoint_index += 1;
            if self.waypoint_index >= self.waypoints.len() {
                self.waypoint_index = 0;
            }
            self.update_path(position, map);
        } else {
            match self.current_path.last() {
                Some(pos_index) => {
                    if map.blocked_idx(*pos_index) || !adjacent(map.idx_pos(*pos_index), position) {
                        self.update_path(position, map);
                    }
                },
                None => self.update_path(position, map),
            }
        }

        match self.decide_direction(position, map) {
            Some(direction) => {
                if direction != body.facing {
                    Intent {
                        phase: ExecutionPhase::Movement,
                        data: IntentData::Direction(direction),
                        action: actions::turn_action,
                    }
                } else {
                    Intent {
                        phase: ExecutionPhase::Movement,
                        data: IntentData::Target(map.idx_pos(*self.current_path.last().unwrap())),
                        action: actions::move_action,
                    }
                }
            },
            None => idle_intent(),
        }
    }

    fn update_path(&mut self, position: Point, map: &Map) {
        let path = rltk::a_star_search(
            map.pos_idx(position),
            map.pos_idx(self.waypoints[self.waypoint_index]),
            map);

        self.current_path = vec![];
        if path.success {
            for step in path.steps.iter().rev() {
                self.current_path.push(*step);
            }
            // Remove starting position
            self.current_path.pop();
        }
    }

    fn decide_direction(&self, position: Point, map: &Map) -> Option<Direction> {
        match self.current_path.last() {
            Some(next_step) => {
                let step = map.idx_pos(*next_step);
                match (position.x - step.x, position.y - step.y) {
                    ( 0, -1) => Some(Direction::Down),
                    ( 0,  1) => Some(Direction::Up),
                    (-1, -1) => Some(Direction::DownRight),
                    (-1,  1) => Some(Direction::UpRight),
                    (-1,  0) => Some(Direction::Right),
                    ( 1, -1) => Some(Direction::DownLeft),
                    ( 1,  1) => Some(Direction::UpLeft),
                    ( 1,  0) => Some(Direction::Left),
                    (dx, dy) => {
                        debug_assert!(false, "unexpected delta ({},{}) pos:{},{} step:{},{}",
                            dx, dy, position.x, position.y, step.x, step.y);
                        None
                    }
                }
            },
            None => None,
        }
    }
}
