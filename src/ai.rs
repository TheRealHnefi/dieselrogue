use rltk::Point;
use crate::Body;
use crate::Map;
use crate::Entity;
use crate::components::*;

pub enum AI {
    None,
    Patrolling(PatrollingAI)
}

pub struct PatrollingAI {
    waypoints: Vec<Point>,
    waypoint_index: usize,
    current_path: Vec<usize>
}

impl PatrollingAI {
    pub fn new(waypoints: Vec<Point>) -> Self {
        Self {
            waypoints: waypoints,
            waypoint_index: 0,
            current_path: vec!()
        }
    }

    pub fn declare_intent(&mut self, body: &Body, map: &Map) -> Intent {
        if self.waypoints[self.waypoint_index] == body.position {
            self.waypoint_index += 1;
            if self.waypoint_index >= self.waypoints.len() {
                self.waypoint_index = 0;
            }
            self.update_path(body, map)
        }
        else {
            match self.current_path.last() {
                Some(pos_index) => {
                    if map.blocked_idx(*pos_index) {
                        self.update_path(body, map);
                    }
                },
                None => self.update_path(body, map)
            }
        }

        let walk_direction = self.decide_direction(body.position, map);

        match walk_direction {
            Some(direction) => {
                if direction != body.facing {
                    return Intent {
                        phase: IntentPhase::Movement,
                        data: IntentData::Direction(direction),
                        action: Entity::resolve_turn
                    };
                }
                else {
                    return Intent {
                        phase: IntentPhase::Movement,
                        data: IntentData::Target(map.idx_pos(self.current_path.pop().unwrap())),
                        action: Entity::resolve_move
                    };
                }
            },
            None => {
                return Intent {
                    phase: IntentPhase::Idle,
                    data: IntentData::Void,
                    action: declare_intent_noop
                }
            }
        }
    }

    fn update_path(&mut self, body: &Body, map: &Map) {
        let path = rltk::a_star_search(
            map.pos_idx(body.position),
            map.pos_idx(self.waypoints[self.waypoint_index]),
            map);

        self.current_path = vec!();
        if path.success {
            for step in path.steps.iter().rev() {
                self.current_path.push(*step);
            }
            // Remove starting position
            self.current_path.pop();
        }
    }

    // Debug assert branches can hit if body has been forcibly moved or failed to move.
    // Not handled in current state of the AI.
    fn decide_direction(&self, position: Point, map: &Map) -> Option<Direction> {
        match self.current_path.last() {
            Some(next_step) => {
                let step = map.idx_pos(*next_step);
                if position.x - step.x == 0 {
                    if position.y - step.y == -1 {
                        return Some(Direction::Down);
                    }
                    else if position.y - step.y == 1 {
                        return Some(Direction::Up);
                    }
                    else {
                        debug_assert!(false, "X was 0, but Y was -1 or 1. Pos: {},{} Step: {}, {}",
                            position.x, position.y, step.x, step.y);
                        return None;
                    }
                }
                else if position.x - step.x == -1 {
                    if position.y - step.y == -1 {
                        return Some(Direction::DownRight);
                    }
                    else if position.y - step.y == 1 {
                        return Some(Direction::UpRight);
                    }
                    else if position.y - step.y == 0 {
                        return Some(Direction::Right);
                    }
                    else {
                        debug_assert!(false, "X was -1, but Y is not 0, -1 or 1. Pos: {},{} Step: {}, {}",
                        position.x, position.y, step.x, step.y);
                        return None;
                    }
                }
                else if position.x - step.x == 1 {
                    if position.y - step.y == -1 {
                        return Some(Direction::DownLeft);
                    }
                    else if position.y - step.y == 1 {
                        return Some(Direction::UpLeft);
                    }
                    else if position.y - step.y == 0 {
                        return Some(Direction::Left);
                    }
                    else {
                        debug_assert!(false, "X was 1, but Y is not 0, -1 or 1. Pos: {},{} Step: {}, {}",
                        position.x, position.y, step.x, step.y);
                        return None;
                    }
                }
                else {
                    debug_assert!(false, "X was not 0, -1 or 1. Pos: {},{} Step: {}, {}",
                    position.x, position.y, step.x, step.y);
                    return None;
                }
            },
            None => None
        }
    }
}

fn declare_intent_noop(_entity: &mut Entity, _map: &mut Map) -> Vec<Effect> {
    return vec!();
}