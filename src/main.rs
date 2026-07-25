use rltk::Point;

mod state;
pub use state::*;
mod world;
pub use world::*;
mod entity;
pub use entity::*;
mod item;
pub use item::*;
mod components;
pub use components::*;
mod player;
pub use player::*;
mod map;
pub use map::*;
mod rect;
pub use rect::*;
mod ui;
pub use ui::*;
mod gamelog;
pub use gamelog::*;
mod menu;
pub use menu::*;
mod input;
pub use input::*;
mod error;
pub use error::*;
mod body;
pub use body::*;
mod ability;
pub use ability::*;
mod ai;
pub use ai::*;
mod viewshed;
pub use viewshed::*;
mod util;
pub use util::*;
mod intent;
pub use intent::*;
mod sprite;
pub use sprite::*;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    // Set size to 80x50 for now and design UI for that. Allow for upscaled UIs later.
    let mut context = RltkBuilder::simple(ui::SCREEN_WIDTH, ui::SCREEN_HEIGHT)?
        .with_title("Diesel Rogue")
        .with_resource_path("resources")
        .with_font("rexpaint_cp437_10x10.png", 10, 10)
        .with_tile_dimensions(10, 10)
        //.with_fullscreen(true)
        .build()?;
    
    context.set_active_font(1, true);

    let mut state = State::new();

    let pos = Point {x: state.world.map.rooms[0].x1+1, y: state.world.map.rooms[0].y1+1};
    let _result = state.world.create_player(pos,
        Direction::Up,
        String::from("Player"));

    let max_room_index = std::cmp::min(state.world.map.rooms.len(), 5);

    let _result = state.world.create_patrolling_goon(Point {x: pos.x + 1, y: pos.y+1},
        Direction::Up,
        String::from("Goon"),
        (0..state.world.map.rooms.len()).collect());

    let _result = state.world.create_patrolling_goon(Point {x: state.world.map.rooms[2].center().0, y: state.world.map.rooms[2].center().1},
        Direction::Up,
        String::from("Goon"),
        (0..max_room_index).collect());

    let _result = state.world.create_tank(Point {x: state.world.map.rooms[3].x1 + 1, y: state.world.map.rooms[3].y1 + 1},
        Direction::Up,
        String::from("Tank"));
    
    let _ = state.world.add_item(pos, Item::grenade());
    let _ = state.world.add_item(Point{x: pos.x + 1, y: pos.y}, Item::machinegun());
    let _ = state.world.add_item(Point{x: pos.x + 2, y: pos.y}, Item::pistol());

    state.log.entries.push("Welcome!".to_string());
 
    rltk::main_loop(context, state)
}