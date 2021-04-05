use specs::prelude::*;

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

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let context = RltkBuilder::simple80x50()
        .with_title("Diesel Rogue")
        .build()?;
    
    let mut game_state = State::new();

    game_state.ecs.register::<Player>();
    game_state.ecs.register::<Position>();
    game_state.ecs.register::<Direction>();
    game_state.ecs.register::<Facing>();
    game_state.ecs.register::<Renderable>();
    game_state.ecs.register::<Viewshed>();

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let player_entity = game_state.ecs
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            color: rltk::RGB::named(rltk::YELLOW),
            background: rltk::RGB::named(rltk::BLACK)
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true
        })
        .with(Facing {direction: Direction::UP})
        .build();
    game_state.ecs.insert(player_entity);
    game_state.ecs.insert(map);

    rltk::main_loop(context, game_state)
}