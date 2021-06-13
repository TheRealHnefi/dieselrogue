use rltk::{RGB, Rltk, Point};
use super::*;
use std::cmp::max;

pub const SCREEN_WIDTH: usize = 80;
pub const SCREEN_HEIGHT: usize = 50;

pub fn draw_main_screen(state: &mut State, context: &mut Rltk) -> Result<(), GameError> {

    draw_map(&state.resources, context)?;

    // TODO: Isn't it a LOT more efficient to iterate over visible tiles and draw their contents?
    {
        let mut query = <(&Position, &Renderable)>::query();
        let map = state.resources.get::<Map>().ok_or(())?;

        for (pos, renderable) in query.iter(&state.ecs) {
            if pos.valid {
                let index = map.xy_idx(pos.x, pos.y);
                if map.visible_tiles[index] {
                    context.set(pos.x, pos.y, renderable.color, renderable.background, renderable.glyph);
                }
            }
        }
    }

    // MIGRATION_TODO: Large renderables. Consider unifying into a single renderable component.
    //     for (pos, render, size) in (&positions, &large_renderables, &sizes).join() {
    //         assert!(size.x * size.y == render.glyphs.len() as i32, "Size and glyphmap size differ for object");
    //         for x in 0..size.x {
    //             for y in 0..size.y {
    //                 let idx = map.xy_idx(pos.x + x, pos.y + y);
    //                 if map.visible_tiles[idx] {
    //                     context.set(pos.x + x, pos.y + y, render.color, render.background, render.glyphs[(x + size.x * y) as usize]);
    //                 }
    //             }
    //         }
    //     }
    // }

    draw_main_ui(state, context)
}

pub fn draw_inventory_screen(state: &mut State, context: &mut Rltk) {
    const LEFT_DIVIDER_X: i32 = 20;
    const RIGHT_DIVIDER_X: i32 = 60;
    const BOT_DIVIDER_Y: i32 = 40;
    const FOREGROUND: RGB = RGB {r: 1., g: 1., b: 1.};
    const BACKGROUND: RGB = RGB {r: 0., g: 0., b: 0.};

    let _corner_top_left = rltk::to_cp437('╔');
    let _corner_top_right = rltk::to_cp437('╗');
    let _corner_bot_left = rltk::to_cp437('╚');
    let _corner_bot_left = rltk::to_cp437('╝');
    let vertical_border = rltk::to_cp437('║');
    let horizontal_border = rltk::to_cp437('═');
    let divider_top = rltk::to_cp437('╦');
    let divider_bot = rltk::to_cp437('╩');
    let _divider_mid = rltk::to_cp437('╬');
    let divider_right = rltk::to_cp437('╣');
    let divider_left = rltk::to_cp437('╠');
    
    context.draw_box_double(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    for y in 1 .. SCREEN_HEIGHT - 1 {
        context.set(LEFT_DIVIDER_X, y, FOREGROUND, BACKGROUND, vertical_border);
        context.set(RIGHT_DIVIDER_X, y, FOREGROUND, BACKGROUND, vertical_border);
    }
    for x in LEFT_DIVIDER_X .. RIGHT_DIVIDER_X {
        context.set(x, BOT_DIVIDER_Y, FOREGROUND, BACKGROUND, horizontal_border);
    }

    context.set(LEFT_DIVIDER_X, 0, FOREGROUND, BACKGROUND, divider_top);
    context.set(LEFT_DIVIDER_X, BOT_DIVIDER_Y, FOREGROUND, BACKGROUND, divider_left);
    context.set(LEFT_DIVIDER_X, SCREEN_HEIGHT - 1, FOREGROUND, BACKGROUND, divider_bot);

    context.set(RIGHT_DIVIDER_X, 0, FOREGROUND, BACKGROUND, divider_top);
    context.set(RIGHT_DIVIDER_X, BOT_DIVIDER_Y, FOREGROUND, BACKGROUND, divider_right);
    context.set(RIGHT_DIVIDER_X, SCREEN_HEIGHT - 1, FOREGROUND, BACKGROUND, divider_bot);

    context.print((LEFT_DIVIDER_X / 2) - 5, 0, " INVENTORY ");
    context.print((SCREEN_WIDTH / 2) - 5, 0, " EQUIPMENT ");
    context.print(RIGHT_DIVIDER_X + 3, 0, " PLAYER STATS ");
    context.print((SCREEN_WIDTH / 2) - 9, BOT_DIVIDER_Y, " ITEM INFORMATION ");

    // let assets = state.ecs.fetch::<RexAssets>();
    // context.render_xp_sprite(&assets.male_silhouette, 34, 3);

    // let player = state.ecs.fetch::<Entity>();
    // let bodies = state.ecs.read_storage::<HumanoidBody>();
    // let body_maybe = bodies.get(*player);
    // match body_maybe {
    //     Some(body) => {
    //         fn draw_equipment_box(ecs: &World, x: i32, y: i32, title: String, contents: EntityOption<Entity>, context: &mut Rltk) {
    //             let names = ecs.read_storage::<Name>();
    //             const EQ_BOX_WIDTH: i32 = 13;
    //             context.draw_box(x, y, EQ_BOX_WIDTH, 2, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    //             context.print(x + (EQ_BOX_WIDTH - title.len() as i32 + 1) / 2, y, title);
    //             match *contents {
    //                 Some(item) => {
    //                     let name_maybe = names.get(item);
    //                     match name_maybe {
    //                         Some(name) => context.print(x + 1, y + 1, name.value.to_string()),
    //                         None => context.print(x + 1, y + 1, "ITEM ERROR"),
    //                     }
    //                 },
    //                 None => ()
    //             }
    //         }
    //         draw_equipment_box(&state.ecs, 21, 4, " HEAD ".to_string(), body.head.equipped_item, context);
    //         draw_equipment_box(&state.ecs, 21, 8, " LEFT ARM ".to_string(), body.left_arm.equipped_item, context);
    //         draw_equipment_box(&state.ecs, 21, 26, " LEGS ".to_string(), body.legs.equipped_item, context);
    //         draw_equipment_box(&state.ecs, 46, 10, " TORSO ".to_string(), body.torso.equipped_item, context);
    //         draw_equipment_box(&state.ecs, 46, 22, " RIGHT ARM ".to_string(), body.right_arm.equipped_item, context);
    //     }
    //     None => {
    //         panic!("Player lacks body");
    //     }
    // }


    let mut y = 2;
    let mut items: Vec<Entity> = Vec::new();

    // MIGRATION_TODO: Make player unique.
    let mut query = <(&Inventory, &Player)>::query();
    for (inventory, _player) in query.iter(&state.ecs) {
        for item in inventory.items.iter() {
            items.push(*item);
        }
    }

    for (i, item) in items.iter().enumerate() {
        let mut foreground = RGB::named(rltk::WHITE);
        let mut background = RGB::named(rltk::BLACK);
        let entry = state.ecs.entry(*item).unwrap();
        let name = entry.get_component::<Name>().unwrap();

        if i == state.inventory_screen_selection as usize {
            background = RGB::named(rltk::MAGENTA);
        }
        context.print_color(2, y, foreground, background, &name.value);
        y += 1;
    }
    // let names = state.ecs.read_storage::<Name>();
    // let equippables = state.ecs.read_storage::<Equippable>();
    // let inventories = state.ecs.read_storage::<Inventory>();
    // let inventory = inventories.get(*player);
    // match inventory {
    //     Some(inv) => {
    //         let mut y = 2;
    //         // Required since EntityVec does not implement IntoIterator
    //         for (i, item) in (&*inv.items).iter().enumerate() {
    //             let mut foreground = RGB::named(rltk::WHITE);
    //             let mut background = RGB::named(rltk::BLACK);
    //             let name = names.get(*item);
    //             assert!(name.is_some(), "Item name expected but not found");
    //             let equippable = equippables.get(*item);

    //             match equippable {
    //                 Some(equip) => {
    //                     if equip.equipped {
    //                         foreground = RGB::named(rltk::GREEN);
    //                     }
    //                 }
    //                 None => ()
    //             }
    //             if i == state.inventory_screen_selection as usize {
    //                 background = RGB::named(rltk::MAGENTA);
    //             }
    //             context.print_color(2, y, foreground, background, &name.unwrap().value);
    //             y += 1;
    //         }
    //     },
    //     None => {
    //         panic!("Player lacks inventory");
    //     }
    // }
}

pub fn draw_main_ui(state: &mut State, context: &mut Rltk) -> Result<(), GameError> {
    context.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    {
        let mut cursor_pos = state.resources.get_mut::<Point>().ok_or(())?;
        let new_mouse_pos = context.mouse_pos();
        if state.mouse_pos.x != new_mouse_pos.0 || state.mouse_pos.y != new_mouse_pos.1 {
            cursor_pos.x = new_mouse_pos.0;
            cursor_pos.y = new_mouse_pos.1;
            state.mouse_pos.x = new_mouse_pos.0;
            state.mouse_pos.y = new_mouse_pos.1;
        }
        context.set_bg(cursor_pos.x, cursor_pos.y, RGB::named(rltk::PINK));
    }

    draw_tooltip(state, context)?;

    let game_log = state.resources.get::<GameLog>().ok_or(())?;
    let mut y = 44;
    let length = max(game_log.entries.len() as i32 - 5, 0) as usize;
    for message in &game_log.entries[length..] {
        context.print(2, y, message);
        y += 1;
    }

    Ok(())
}

fn draw_tooltip(state: &mut State, context: &mut Rltk) -> Result<(), GameError> {
    let map = state.resources.get::<Map>().ok_or(())?;

    if state.mouse_pos.x >= map.width || state.mouse_pos.y >= map.height {
        return Ok(());
    }

    let index = map.xy_idx(state.mouse_pos.x, state.mouse_pos.y);
    let mut tooltip: Vec<String> = Vec::new();

    if !map.visible_tiles[index] {
        return Ok(());
    }

    if map.blocked_tiles[index] {
        match map.tile_blockers[index] {
            Some(_actor) => tooltip.push(format!("=== {} ===", "person")),
            None => tooltip.push(format!("=== {} ===", "wall"))
        }
    }

    match map.tile_items[index] {
        Some(_item) => tooltip.push(format!("=== {} ===", "thing")),
        None => ()
    }

    draw_infobox(context, state.mouse_pos, tooltip);

    Ok(())
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

pub fn draw_map(resources: &Resources, ctx: &mut Rltk) -> Result<(), GameError>{
    let map = resources.get::<Map>().ok_or(())?;
    

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
                    glyph = rltk::to_cp437('█');
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

    Ok(())
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