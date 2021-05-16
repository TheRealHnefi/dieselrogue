use legion::*;
use rltk::{Point};

mod state;
pub use state::*;
mod components;
pub use components::*;
mod player;
pub use player::*;
mod map;
pub use map::*;
mod rect;
pub use rect::*;
mod visibility_system;
pub use visibility_system::*;
mod map_indexing_system;
pub use map_indexing_system::*;
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
mod menu;
pub use menu::*;
mod input;
pub use input::*;
// mod saveload_system;
// pub use saveload_system::*;
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

    let mut game_state = State::new();

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    game_state.ecs.push((
        Position {
            x: player_x,
            y: player_y
        },
        Renderable {
            glyph: rltk::to_cp437('8'),
            color: rltk::RGB::named(rltk::YELLOW),
            background: rltk::RGB::named(rltk::BLACK)
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: 10,
            dirty: true
        },
        Facing {direction: Direction::Up},
        Name {value: "Player".to_string()},
        BlocksTile {},
        Player {}
    ));

    game_state.ecs.push((
        Position {
            x: map.rooms[1].center().0,
            y: map.rooms[1].center().1
        },
        Renderable {
            glyph: rltk::to_cp437('8'),
            color: rltk::RGB::named(rltk::YELLOW),
            background: rltk::RGB::named(rltk::BLACK)
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: 10,
            dirty: true
        },
        Facing {direction: Direction::Left},
        Name {value: "Goon".to_string()},
        BlocksTile {}
    ));

    game_state.ecs.push((
        Position {
            x: player_x,
            y: player_y - 2
        },
        Renderable {
            glyph: rltk::to_cp437('8'),
            color: rltk::RGB::named(rltk::YELLOW),
            background: rltk::RGB::named(rltk::BLACK)
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: 10,
            dirty: true
        },
        Facing {direction: Direction::Left},
        Name {value: "Goon".to_string()},
        BlocksTile {}
    ));

    let cursor_pos = Point {x: 0, y: 0};

    game_state.resources.insert(map);
    game_state.resources.insert(cursor_pos);
    
    //game_state.ecs.insert(RunState::PreRun);
    game_state.resources.insert(GameLog {entries: vec!["Welcome!".to_string()]});
    //game_state.ecs.insert(rex_assets::RexAssets::new());

    rltk::main_loop(context, game_state)
}