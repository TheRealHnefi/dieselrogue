use rltk::{RGB, Rltk, Point};
use specs::prelude::*;
use super::{Position, Map, HumanoidBody, GameLog, Inventory, Name, State, Size};
use std::cmp::max;

pub fn draw_ui(state: &mut State, context: &mut Rltk) {
    context.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    let mut cursor_pos = state.ecs.fetch_mut::<Point>();
    let new_mouse_pos = context.mouse_pos();
    if state.mouse_pos.x != new_mouse_pos.0 || state.mouse_pos.y != new_mouse_pos.1 {
        cursor_pos.x = new_mouse_pos.0;
        cursor_pos.y = new_mouse_pos.1;
        state.mouse_pos.x = new_mouse_pos.0;
        state.mouse_pos.y = new_mouse_pos.1;
    }
    context.set_bg(cursor_pos.x, cursor_pos.y, RGB::named(rltk::PINK));
    draw_tooltip(&state.ecs, context, *cursor_pos);

    let game_log = state.ecs.fetch::<GameLog>();
    let mut y = 44;
    let length = max(game_log.entries.len() as i32 - 5, 0) as usize;
    for message in &game_log.entries[length..] {
        context.print(2, y, message);
        y += 1;
    }

    let inventories = state.ecs.read_storage::<Inventory>();
    let player = state.ecs.fetch::<Entity>();
    let inventory = inventories.get(*player);
    let names = state.ecs.read_storage::<Name>();
    match inventory {
        Some(inv) => {
            let mut y = 44;
            for item in &inv.items {
                let name = names.get(*item);
                assert!(name.is_some(), "Item name expected but not found");
                context.print(50, y, &name.unwrap().value);
                y += 1;
            }
        },
        None => {
            panic!("Player lacks inventory");
        }
    }
}

fn draw_tooltip(ecs: &World, context: &mut Rltk, cursor_position: Point) {
    let map = ecs.fetch::<Map>();

    if cursor_position.x >= map.width || cursor_position.y >= map.height {
        return;
    }

    let positions = ecs.read_storage::<Position>();
    let bodies = ecs.read_storage::<HumanoidBody>();
    let names = ecs.read_storage::<Name>();
    let sizes = ecs.read_storage::<Size>();
    let mut tooltip: Vec<String> = Vec::new();

    for (name, pos, size_option) in (&names, &positions, (&sizes).maybe()).join() {
        let index = map.xy_idx(cursor_position.x, cursor_position.y);
        let size = match size_option {
            None => {
                Point::new(1, 1)
            },
            Some(s) => {
                Point::new(s.x, s.y)
            }
        };
        if cursor_position.x >= pos.x
            && cursor_position.x < pos.x + size.x
            && cursor_position.y >= pos.y
            && cursor_position.y < pos.y + size.y
            && map.visible_tiles[index] {
            tooltip.push(format!("=== {} ===", name.value));
        }
    }
    for (body, pos) in (&bodies, &positions).join() {
        let index = map.xy_idx(pos.x, pos.y);
        if pos.x == cursor_position.x && pos.y == cursor_position.y && map.visible_tiles[index] {
            tooltip.push(format!("Hitpoints: {}/{}", body.hitpoints, body.max_hitpoints));
            tooltip.push(format!("Head:      {}/{}", body.head.hitpoints, body.head.max_hitpoints));
            tooltip.push(format!("Torso:     {}/{}", body.torso.hitpoints, body.torso.max_hitpoints));
            tooltip.push(format!("Left arm:  {}/{}", body.left_arm.hitpoints, body.left_arm.max_hitpoints));
            tooltip.push(format!("Right arm: {}/{}", body.right_arm.hitpoints, body.right_arm.max_hitpoints));
            tooltip.push(format!("Left leg:  {}/{}", body.left_leg.hitpoints, body.left_leg.max_hitpoints));
            tooltip.push(format!("Right leg: {}/{}", body.right_leg.hitpoints, body.right_leg.max_hitpoints));
        }
    }
    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 {
                width = s.len() as i32;
            }
        }
        width += 3;

        if cursor_position.x > 40 {
            let arrow_pos = Point::new(cursor_position.x - 1, cursor_position.y);
            let left_x = cursor_position.x - width;
            let mut y = cursor_position.y + 1;

            context.draw_box(left_x - 1, y - 1, width -1, tooltip.len() as i32 + 1, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

            for s in tooltip.iter() {
                context.print_color(left_x, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), s);
                y += 1;
            }
            context.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), &">".to_string());
        } else {
            let arrow_pos = Point::new(cursor_position.x + 1, cursor_position.y);
            let left_x = cursor_position.x + 3;
            let mut y = cursor_position.y;

            context.draw_box(left_x - 1, y - 1, width -1, tooltip.len() as i32 + 1, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

            for s in tooltip.iter() {
                context.print_color(left_x + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), s);
                y += 1;
            }
            context.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), &"<".to_string());
        }
    }
}