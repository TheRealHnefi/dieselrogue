use specs::prelude::*;
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
mod enemy_ai_system;
pub use enemy_ai_system::*;
mod ui;
pub use ui::*;
mod game_log;
pub use game_log::*;
mod inventory_system;
pub use inventory_system::*;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let mut context = RltkBuilder::simple80x50()
        .with_title("Diesel Rogue")
        .with_resource_path("resources")
        .with_font("rexpaint_cp437_10x10.png", 10, 10)
        .with_tile_dimensions(10, 10)
        .build()?;
    
    context.set_active_font(1, true);

    let mut game_state = State::new();

    game_state.ecs.register::<Player>();
    game_state.ecs.register::<Enemy>();
    game_state.ecs.register::<Position>();
    game_state.ecs.register::<Size>();
    game_state.ecs.register::<Direction>();
    game_state.ecs.register::<Facing>();
    game_state.ecs.register::<Renderable>();
    game_state.ecs.register::<LargeRenderable>();
    game_state.ecs.register::<Viewshed>();
    game_state.ecs.register::<Name>();
    game_state.ecs.register::<BlocksTile>();
    game_state.ecs.register::<GettableItem>();
    game_state.ecs.register::<GettingItem>();
    game_state.ecs.register::<Inventory>();
    game_state.ecs.register::<HumanoidBody>();

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let player_entity = game_state.ecs
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y
        })
        .with(Renderable {
            glyph: rltk::to_cp437('8'),
            color: rltk::RGB::named(rltk::YELLOW),
            background: rltk::RGB::named(rltk::BLACK)
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 10,
            dirty: true
        })
        .with(Facing {direction: Direction::UP})
        .with(HumanoidBody::new(20))
        .with(Inventory {items: Vec::new()})
        .with(Name {value: "Player".to_string()})
        .build();
    game_state.ecs.insert(player_entity);

    let (enemy_x, enemy_y) = map.rooms[1].center();
    game_state.ecs
        .create_entity()
        .with(Position {
            x: enemy_x,
            y: enemy_y
        })
        .with(Renderable {
            glyph: rltk::to_cp437('8'),
            color: rltk::RGB::named(rltk::RED),
            background: rltk::RGB::named(rltk::BLACK)
        })
        .with(Enemy {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 5,
            dirty: true
        })
        .with(Facing {direction: Direction::UP})
        .with(BlocksTile {})
        .with(HumanoidBody::new(20))
        .with(Inventory {items: Vec::new()})
        .with(Name {value: "Goon".to_string()})
        .build();

    let (gun_x, gun_y) = map.rooms[2].center();
    game_state.ecs
        .create_entity()
        .with(Position {
            x: gun_x,
            y: gun_y
        })
        .with(Renderable {
            glyph: 169,
            color: rltk::RGB::named(rltk::BLUE),
            background: rltk::RGB::named(rltk::BLACK)
        })
        .with(GettableItem {})
        .with(Name {value: "Gun".to_string()})
        .build();

    let (tank_x, tank_y) = map.rooms[3].center();
    game_state.ecs
        .create_entity()
        .with(Position {
            x: tank_x,
            y: tank_y
        })
        .with(Size {
            x: 3,
            y: 3
        })
        .with(LargeRenderable {
            glyphs: vec![213,
                        rltk::to_cp437('│'),
                        rltk::to_cp437('╕'),
                        rltk::to_cp437('╞'),
                        rltk::to_cp437('█'),
                        rltk::to_cp437('╡'),
                        rltk::to_cp437('╘'),
                        rltk::to_cp437('═'),
                        rltk::to_cp437('╛')],
            color: rltk::RGB::named(rltk::RED),
            background: rltk::RGB::named(rltk::BLACK)
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 5,
            dirty: true
        })
        .with(BlocksTile {})
        .with(Facing {direction: Direction::UP})
        .with(Name {value: "Tank".to_string()})
        .build();

    let cursor_pos = Point{x:0, y:0};

    game_state.ecs.insert(map);
    game_state.ecs.insert(cursor_pos);
    game_state.ecs.insert(RunState::PreRun);
    game_state.ecs.insert(GameLog {entries: vec!["Welcome!".to_string()]});

    rltk::main_loop(context, game_state)
}