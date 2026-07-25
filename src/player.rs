use rltk::{VirtualKeyCode, Rltk};
use specs::prelude::*;
use super::{Position, Direction, Facing, Player, State, Renderable};
use std::cmp::{min, max};

pub fn try_move_player(direction: Direction, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut facings = ecs.write_storage::<Facing>();
    let mut players = ecs.write_storage::<Player>();
    let mut renderables = ecs.write_storage::<Renderable>();

    let (delta_x, delta_y, glyph);
    match direction {
        Direction::UP => {delta_x = 0; delta_y = -1; glyph = rltk::to_cp437('8')},
        Direction::UPRIGHT => {delta_x = 1; delta_y = -1; glyph = rltk::to_cp437('9')},
        Direction::RIGHT => {delta_x = 1; delta_y = 0; glyph = rltk::to_cp437('6')},
        Direction::DOWNRIGHT => {delta_x = 1; delta_y = 1; glyph = rltk::to_cp437('3')},
        Direction::DOWN => {delta_x = 0; delta_y = 1; glyph = rltk::to_cp437('2')},
        Direction::DOWNLEFT => {delta_x = -1; delta_y = 1; glyph = rltk::to_cp437('1')},
        Direction::LEFT => {delta_x = -1; delta_y = 0; glyph = rltk::to_cp437('4')},
        Direction::UPLEFT => {delta_x = -1; delta_y = -1; glyph = rltk::to_cp437('7')},
    }

    for (_player, pos, facing, renderable) in (&mut players, &mut positions, &mut facings, &mut renderables).join() {
        if facing.direction == direction {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
        } else {
            facing.direction = direction;
            renderable.glyph = glyph;
        }
    }
}

pub fn player_input(game_state: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        None => { }
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::H => try_move_player(Direction::LEFT, &mut game_state.ecs),

            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 |
            VirtualKeyCode::L => try_move_player(Direction::RIGHT, &mut game_state.ecs),

            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 |
            VirtualKeyCode::K => try_move_player(Direction::UP, &mut game_state.ecs),

            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 |
            VirtualKeyCode::J => try_move_player(Direction::DOWN, &mut game_state.ecs),

            VirtualKeyCode::Numpad9 |
            VirtualKeyCode::Y => try_move_player(Direction::UPRIGHT, &mut game_state.ecs),

            VirtualKeyCode::Numpad7 |
            VirtualKeyCode::U => try_move_player(Direction::UPLEFT, &mut game_state.ecs),

            VirtualKeyCode::Numpad3 |
            VirtualKeyCode::N => try_move_player(Direction::DOWNRIGHT, &mut game_state.ecs),

            VirtualKeyCode::Numpad1 |
            VirtualKeyCode::B => try_move_player(Direction::DOWNLEFT, &mut game_state.ecs),

            _ => { }
        }
    }
}
