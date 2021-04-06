use rltk::{RGB, Rltk, Point};
use specs::prelude::*;
use super::{Position, Map, Enemy};

pub fn draw_ui(ecs: &World, context: &mut Rltk) {
    context.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    let mouse_pos = context.mouse_pos();
    draw_tooltip(ecs, context, Point::new(mouse_pos.0, mouse_pos.1));
}

fn draw_tooltip(ecs: &World, context: &mut Rltk, position: Point) {
    let map = ecs.fetch::<Map>();

    if position.x >= map.width || position.y >= map.height {
        return;
    }

    let positions = ecs.read_storage::<Position>();
    let enemies = ecs.read_storage::<Enemy>();
    let mut tooltip: Vec<String> = Vec::new();
    for (_enemy, pos) in (&enemies, &positions).join() {
        let index = map.xy_idx(pos.x, pos.y);
        if pos.x == position.x && pos.y == position.y && map.visible_tiles[index] {
            tooltip.push("Enemy!".to_string());
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
            let arrow_pos = Point::new(position.x - 2, position.y);
            let left_x = position.x - width;
            let mut y = position.y;
            for s in tooltip.iter() {
                context.print_color(left_x, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    context.print_color(arrow_pos.x - i, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &" ".to_string());
                }
                y += 1;
            }
            context.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &"->".to_string());
        } else {
            let arrow_pos = Point::new(position.x + 1, position.y);
            let left_x = position.x +3;
            let mut y = position.y;
            for s in tooltip.iter() {
                context.print_color(left_x + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                let padding = (width - s.len() as i32)-1;
                for i in 0..padding {
                    context.print_color(arrow_pos.x + 1 + i, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &" ".to_string());
                }
                y += 1;
            }
            context.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &"<-".to_string());
        }
    }
}