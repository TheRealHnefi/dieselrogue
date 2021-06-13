use rltk::{Point};

mod state;
pub use state::*;
mod world;
pub use world::*;
mod entity;
pub use entity::*;
mod components;
pub use components::*;
// mod player;
// pub use player::*;
// mod map;
// pub use map::*;
// mod rect;
// pub use rect::*;
// mod visibility_system;
// pub use visibility_system::*;
// mod map_indexing_system;
// pub use map_indexing_system::*;
// mod enemy_ai_system;
// pub use enemy_ai_system::*;
// mod tank_ai_system;
// pub use tank_ai_system::*;
// mod damage_system;
// pub use damage_system::*;
mod ui;
pub use ui::*;
mod game_log;
pub use game_log::*;
// mod inventory_system;
// pub use inventory_system::*;
// mod menu;
// pub use menu::*;
// mod input;
// pub use input::*;
// mod saveload_system;
// pub use saveload_system::*;
// mod serde_collections;
// pub use serde_collections::*;
// mod rex_assets;
// pub use rex_assets::*;

#[derive(Debug)]
pub struct GameError {
}

impl From<()> for GameError {
    fn from(_err: ()) -> Self {
        GameError {}
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    // Set size to 80x50 for now and design UI for that. Allow for upscaled UIs later.
    let mut context = RltkBuilder::simple(ui::SCREEN_WIDTH, ui::SCREEN_HEIGHT)?
        .with_title("Diesel Rogue")
        .with_resource_path("resources")
        .with_font("rexpaint_cp437_10x10.png", 10, 10)
        .with_tile_dimensions(10, 10)
        .with_fullscreen(true)
        .build()?;
    
    context.set_active_font(1, true);

    let mut state = State::new();

    let _player = state.world.create_player();

    state.log.entries.push("Welcome!".to_string());
 
    rltk::main_loop(context, state)
}