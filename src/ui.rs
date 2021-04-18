use rltk::{RGB, Rltk, Point};
use specs::prelude::*;
use super::{Position, Map, HumanoidBody, GameLog, Inventory, Name, State, Size, TileType, Renderable, LargeRenderable};
use std::cmp::max;

pub fn draw_main_screen(state: &mut State, context: &mut Rltk) {
    draw_map(&state.ecs, context);

    {
        let positions = state.ecs.read_storage::<Position>();
        let renderables = state.ecs.read_storage::<Renderable>();
        let large_renderables = state.ecs.read_storage::<LargeRenderable>();
        let sizes = state.ecs.read_storage::<Size>();
        let map = state.ecs.fetch::<Map>();

        // TODO: Unify these, for efficiency?
        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                context.set(pos.x, pos.y, render.color, render.background, render.glyph);
            }
        }

        for (pos, render, size) in (&positions, &large_renderables, &sizes).join() {
            assert!(size.x * size.y == render.glyphs.len() as i32, "Size and glyphmap size differ for object");
            for x in 0..size.x {
                for y in 0..size.y {
                    let idx = map.xy_idx(pos.x + x, pos.y + y);
                    if map.visible_tiles[idx] {
                        context.set(pos.x + x, pos.y + y, render.color, render.background, render.glyphs[(x + size.x * y) as usize]);
                    }
                }
            }
        }
    }

    draw_main_ui(state, context);
}

pub fn draw_inventory_screen(_state: &mut State, context: &mut Rltk) {
    context.draw_box(0, 0, 79, 49, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
}

pub fn draw_main_ui(state: &mut State, context: &mut Rltk) {
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
            // Required since EntityVec does not implement IntoIterator
            let items: &Vec<Entity> = &*inv.items;
            for item in items {
                let name = names.get(*item);
                assert!(name.is_some(), "Item name expected but not found");
                context.print(60, y, &name.unwrap().value);
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
    draw_infobox(context, cursor_position, tooltip)
}

fn draw_infobox(context: &mut Rltk, position: Point, contents: Vec<String>) {
    if contents.is_empty() {
        return;
    }

    let width = (contents.iter().max_by_key(|x| x.len()).unwrap_or(&"".to_string()).len() + 3) as i32;
    let height = contents.len() as i32 + 1;
    
    let left = if position.x < 40 {
        position.x + 1
    } else {
        position.x - width - 1 as i32
    };
    let top = if position.y < 25 {
        position.y
    } else {
        position.y - height
    };

    context.draw_box(left, top, width, height, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    for (i, line) in contents.iter().enumerate() {
        context.print(left + 2, top + 1 + i as i32, line.to_string());
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut y = 0;
    let mut x = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.revealed_tiles[idx] {
            let glyph;
            let mut foreground;
            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    foreground = RGB::from_f32(0.5, 1.0, 0.5);
                }
                TileType::Wall => {
                    glyph = rltk::to_cp437('#');
                    foreground = RGB::from_f32(0.0, 1.0, 0.0);
                }
            }
            if !map.visible_tiles[idx] {
                foreground = foreground.to_greyscale();
            }
            ctx.set(x, y, foreground, RGB::from_f32(0.0, 0.0, 0.0), glyph);
        }
        x += 1;
        if x >= map.width {
            x = 0;
            y += 1;
        }
    }
}

pub fn draw_menu(state: &State, context: &mut Rltk) {
    for menu in &state.menu_stack {
        let mut width = 0;
        for row in &menu.rows {
            if row.text.len() > width {
                width = row.text.len();
            }
        }
        context.draw_box(menu.x, menu.y, width + 3, menu.rows.len() + 1, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
        for (i, row) in menu.rows.iter().enumerate() {
            if menu.selected_row == i {
                context.print_color(menu.x + 2, menu.y + 1 + i as i32, RGB::named(rltk::WHITE), RGB::named(rltk::MAGENTA), row.text.to_string());
            } else {
                context.print_color(menu.x + 2, menu.y + 1 + i as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), row.text.to_string());
            }
        }
    }
}