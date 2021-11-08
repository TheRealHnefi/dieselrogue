use rltk::{RGB, Rltk};
use super::*;
use std::cmp::max;

pub const SCREEN_WIDTH: usize = 80;
pub const SCREEN_HEIGHT: usize = 50;

pub fn draw_main_screen(state: &mut State, context: &mut Rltk) {
    draw_map(&state.world.map, context);
    draw_main_ui(state, context);
}

pub fn draw_inventory_screen(_state: &mut State, _context: &mut Rltk) {
    // const LEFT_DIVIDER_X: i32 = 20;
    // const RIGHT_DIVIDER_X: i32 = 60;
    // const BOT_DIVIDER_Y: i32 = 40;
    // const FOREGROUND: RGB = RGB {r: 1., g: 1., b: 1.};
    // const BACKGROUND: RGB = RGB {r: 0., g: 0., b: 0.};

    // let _corner_top_left = rltk::to_cp437('╔');
    // let _corner_top_right = rltk::to_cp437('╗');
    // let _corner_bot_left = rltk::to_cp437('╚');
    // let _corner_bot_left = rltk::to_cp437('╝');
    // let vertical_border = rltk::to_cp437('║');
    // let horizontal_border = rltk::to_cp437('═');
    // let divider_top = rltk::to_cp437('╦');
    // let divider_bot = rltk::to_cp437('╩');
    // let _divider_mid = rltk::to_cp437('╬');
    // let divider_right = rltk::to_cp437('╣');
    // let divider_left = rltk::to_cp437('╠');
    
    // context.draw_box_double(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    // for y in 1 .. SCREEN_HEIGHT - 1 {
    //     context.set(LEFT_DIVIDER_X, y, FOREGROUND, BACKGROUND, vertical_border);
    //     context.set(RIGHT_DIVIDER_X, y, FOREGROUND, BACKGROUND, vertical_border);
    // }
    // for x in LEFT_DIVIDER_X .. RIGHT_DIVIDER_X {
    //     context.set(x, BOT_DIVIDER_Y, FOREGROUND, BACKGROUND, horizontal_border);
    // }

    // context.set(LEFT_DIVIDER_X, 0, FOREGROUND, BACKGROUND, divider_top);
    // context.set(LEFT_DIVIDER_X, BOT_DIVIDER_Y, FOREGROUND, BACKGROUND, divider_left);
    // context.set(LEFT_DIVIDER_X, SCREEN_HEIGHT - 1, FOREGROUND, BACKGROUND, divider_bot);

    // context.set(RIGHT_DIVIDER_X, 0, FOREGROUND, BACKGROUND, divider_top);
    // context.set(RIGHT_DIVIDER_X, BOT_DIVIDER_Y, FOREGROUND, BACKGROUND, divider_right);
    // context.set(RIGHT_DIVIDER_X, SCREEN_HEIGHT - 1, FOREGROUND, BACKGROUND, divider_bot);

    // context.print((LEFT_DIVIDER_X / 2) - 5, 0, " INVENTORY ");
    // context.print((SCREEN_WIDTH / 2) - 5, 0, " EQUIPMENT ");
    // context.print(RIGHT_DIVIDER_X + 3, 0, " PLAYER STATS ");
    // context.print((SCREEN_WIDTH / 2) - 9, BOT_DIVIDER_Y, " ITEM INFORMATION ");

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

fn draw_main_ui(state: &mut State, context: &mut Rltk) {
    context.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    if state.run_state == RunState::AwaitingPositionalTargetingInput {
        context.set_bg(state.cursor_pos.x, state.cursor_pos.y, RGB::named(rltk::PINK));
        //    draw_tooltip(&state.ecs, context, *cursor_pos);
    }

    let mut y = 44;
    let length = max(state.log.entries.len() as i32 - 5, 0) as usize;
    for message in &state.log.entries[length..] {
        context.print(2, y, message);
        y += 1;
    }
}

// fn draw_tooltip(ecs: &World, context: &mut Rltk, cursor_position: Point) {
//     let map = ecs.fetch::<Map>();

//     if cursor_position.x >= map.width || cursor_position.y >= map.height {
//         return;
//     }

//     let positions = ecs.read_storage::<Position>();
//     let bodies = ecs.read_storage::<HumanoidBody>();
//     let names = ecs.read_storage::<Name>();
//     let sizes = ecs.read_storage::<Size>();
//     let mut tooltip: Vec<String> = Vec::new();

//     for (name, pos, size_option) in (&names, &positions, (&sizes).maybe()).join() {
//         let index = map.xy_idx(cursor_position.x, cursor_position.y);
//         let size = match size_option {
//             None => {
//                 Point::new(1, 1)
//             },
//             Some(s) => {
//                 Point::new(s.x, s.y)
//             }
//         };
//         if cursor_position.x >= pos.x
//             && cursor_position.x < pos.x + size.x
//             && cursor_position.y >= pos.y
//             && cursor_position.y < pos.y + size.y
//             && map.visible_tiles[index] {
//             tooltip.push(format!("=== {} ===", name.value));
//         }
//     }
//     for (body, pos) in (&bodies, &positions).join() {
//         let index = map.xy_idx(pos.x, pos.y);
//         if pos.x == cursor_position.x && pos.y == cursor_position.y && map.visible_tiles[index] {
//             tooltip.push(format!("Hitpoints: {}/{}", body.hitpoints, body.max_hitpoints));
//             tooltip.push(format!("Head:      {}/{}", body.head.hitpoints, body.head.max_hitpoints));
//             tooltip.push(format!("Torso:     {}/{}", body.torso.hitpoints, body.torso.max_hitpoints));
//             tooltip.push(format!("Left arm:  {}/{}", body.left_arm.hitpoints, body.left_arm.max_hitpoints));
//             tooltip.push(format!("Right arm: {}/{}", body.right_arm.hitpoints, body.right_arm.max_hitpoints));
//             tooltip.push(format!("Legs:      {}/{}", body.legs.hitpoints, body.legs.max_hitpoints));
//         }
//     }
//     draw_infobox(context, cursor_position, tooltip)
// }

// fn draw_infobox(context: &mut Rltk, position: Point, contents: Vec<String>) {
//     if contents.is_empty() {
//         return;
//     }

//     let width = (contents.iter().max_by_key(|x| x.len()).unwrap_or(&"".to_string()).len() + 3) as i32;
//     let height = contents.len() as i32 + 1;
    
//     let left = if position.x < 40 {
//         position.x + 1
//     } else {
//         position.x - width - 1 as i32
//     };
//     let top = if position.y < 25 {
//         position.y
//     } else {
//         position.y - height
//     };

//     context.draw_box(left, top, width, height, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

//     for (i, line) in contents.iter().enumerate() {
//         context.print(left + 2, top + 1 + i as i32, line.to_string());
//     }
// }

fn draw_map(map: &Map, ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        // TODO - remove true
        if true || map.revealed_tiles[idx] {
            let glyph;
            let mut foreground;
            let mut background = RGB::from_f32(0.0, 0.0, 0.0);
            match tile {
                TileType::Floor => {
                    match &map.pawns[idx] {
                        Some(pawn) => {
                            glyph = pawn.renderable.glyph;
                            foreground = pawn.renderable.color;
                            background = pawn.renderable.background;
                        },

                        None => {
                            match &map.items[idx] {
                                Some(item) => {
                                    glyph = item.renderable.glyph;
                                    foreground = item.renderable.color;
                                    background = item.renderable.background;
                                }
                                None => {
                                    glyph = rltk::to_cp437('.');
                                    foreground = RGB::from_f32(0.25, 0.25, 0.25);
                                }
                            }
                        }
                    };
                }
                TileType::Wall => {
                    glyph = rltk::to_cp437('█');
                    foreground = RGB::from_f32(0.0, 1.0, 0.0);
                }
            }
            if !map.visible_tiles[idx] {
                foreground = foreground.to_greyscale();
            }
            ctx.set(x, y, foreground, background, glyph);
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
        menu.draw(context);
    }
}