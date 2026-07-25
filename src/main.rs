use specs::prelude::*;

mod state;
pub use state::*;
mod components;
pub use components::*;
mod player;
pub use player::*;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let context = RltkBuilder::simple80x50()
        .with_title("Diesel Rogue")
        .build()?;
    
    let mut game_state = State {
        ecs: World::new()
    };

    game_state.ecs.register::<Player>();
    game_state.ecs.register::<Position>();
    game_state.ecs.register::<Direction>();
    game_state.ecs.register::<Facing>();
    game_state.ecs.register::<Renderable>();

    let player_entity = game_state.ecs
        .create_entity()
        .with(Position {
            x: 15,
            y: 15
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            color: rltk::RGB::named(rltk::YELLOW),
            background: rltk::RGB::named(rltk::BLACK)
        })
        .with(Player {})
        .with(Facing {direction: Direction::UP})
        .build();
    game_state.ecs.insert(player_entity);

    rltk::main_loop(context, game_state)
}