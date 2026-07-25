use rltk::{RGB, Rltk, Point};
use specs::prelude::*;
use super::{Position, Map, HumanoidBody, GameLog, Inventory, Name};
use std::cmp::max;

pub fn draw_ui(ecs: &World, context: &mut Rltk) {
    context.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    let mouse_pos = context.mouse_pos();
    draw_tooltip(ecs, context, Point::new(mouse_pos.0, mouse_pos.1));

    let game_log = ecs.fetch::<GameLog>();
    let mut y = 44;
    let length = max(game_log.entries.len() as i32 - 5, 0) as usize;
    for message in &game_log.entries[length..] {
        context.print(2, y, message);
        y += 1;
    }

    let inventories = ecs.read_storage::<Inventory>();
    let player = ecs.fetch::<Entity>();
    let inventory = inventories.get(*player);
    let names = ecs.read_storage::<Name>();
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

fn draw_tooltip(ecs: &World, context: &mut Rltk, position: Point) {
    let map = ecs.fetch::<Map>();

    if position.x >= map.width || position.y >= map.height {
        return;
    }

    let positions = ecs.read_storage::<Position>();
    let bodies = ecs.read_storage::<HumanoidBody>();
    let mut tooltip: Vec<String> = Vec::new();
    for (body, pos) in (&bodies, &positions).join() {
        let index = map.xy_idx(pos.x, pos.y);
        if pos.x == position.x && pos.y == position.y && map.visible_tiles[index] {
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

        if position.x > 40 {
            let arrow_pos = Point::new(position.x - 1, position.y);
            let left_x = position.x - width;
            let mut y = position.y + 1;

            context.draw_box(left_x - 1, y - 1, width -1, tooltip.len() as i32 + 1, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

            for s in tooltip.iter() {
                context.print_color(left_x, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), s);
                y += 1;
            }
            context.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), &">".to_string());
        } else {
            let arrow_pos = Point::new(position.x + 1, position.y);
            let left_x = position.x + 3;
            let mut y = position.y;

            context.draw_box(left_x - 1, y - 1, width -1, tooltip.len() as i32 + 1, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

            for s in tooltip.iter() {
                context.print_color(left_x + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), s);
                y += 1;
            }
            context.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), &"<".to_string());
        }
    }
}