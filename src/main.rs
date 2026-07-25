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
mod animation;
pub use animation::*;

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

    let mut state = State::new_game_state(10);
    //let mut state = State::new_performance_test();

    state.log.entries.push("Welcome!".to_string());
 
    rltk::main_loop(context, state)
}