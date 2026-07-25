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
    waypoint_index: usize
}

impl PatrollingAI {
    pub fn new(waypoints: Vec<Point>) -> Self {
        Self {
            waypoints: waypoints,
            waypoint_index: 0
        }
    }

    pub fn declare_intent(&mut self, body: &Body, map: &Map) -> Intent {
        if self.waypoints[self.waypoint_index] == body.position {
            self.waypoint_index += 1;
            if self.waypoint_index >= self.waypoints.len() {
                self.waypoint_index = 0;
            }
        }

        let path = rltk::a_star_search(
            map.pos_idx(body.position),
            map.pos_idx(self.waypoints[self.waypoint_index]),
            map);

        if path.success && path.steps.len() > 1 {
            let walk_direction;
            let step = map.idx_pos(path.steps[1]);
            if body.position.x - step.x == 0 {
                if body.position.y - step.y == -1 {
                    walk_direction = Direction::Up;
                }
                else if body.position.y - step.y == 1 {
                    walk_direction = Direction::Down;
                }
                else {
                    panic!("X was 0, but Y is not -1 or 1");
                }
            }
            else if body.position.x - step.x == -1 {
                if body.position.y - step.y == -1 {
                    walk_direction = Direction::UpLeft;
                }
                else if body.position.y - step.y == 1 {
                    walk_direction = Direction::DownLeft;
                }
                else if body.position.y - step.y == 0 {
                    walk_direction = Direction::Left;
                }
                else {
                    panic!("X was -1, but Y is not 0, -1 or 1");
                }
            }
            else if body.position.x - step.x == 1 {
                if body.position.y - step.y == -1 {
                    walk_direction = Direction::UpRight;
                }
                else if body.position.y - step.y == 1 {
                    walk_direction = Direction::DownRight;
                }
                else if body.position.y - step.y == 0 {
                    walk_direction = Direction::Right;
                }
                else {
                    panic!("X was 1, but Y is not 0, -1 or 1");
                }
            }
            else {
                panic!("X was not 0, -1 or 1");
            }

            if walk_direction != body.facing {
                return Intent {
                    phase: IntentPhase::Movement,
                    data: IntentData::Direction(walk_direction),
                    action: Entity::resolve_turn
                };
            }
            else {
                return Intent {
                    phase: IntentPhase::Movement,
                    data: IntentData::Target(step),
                    action: Entity::resolve_move
                };
            }
        }

        return Intent {
            phase: IntentPhase::Idle,
            data: IntentData::Void,
            action: declare_intent_noop
        }
    }
}

fn declare_intent_noop(_entity: &mut Entity, _map: &mut Map) -> Vec<Effect> {
    return vec!();
}