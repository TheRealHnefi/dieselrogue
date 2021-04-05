use rltk::{Rltk};
use specs::prelude::*;

mod state;
use state::*;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let context = RltkBuilder::simple80x50()
        .with_title("Diesel Rogue")
        .build()?;
    
    let mut game_state = State {
        ecs: World::new()
    };

    rltk::main_loop(context, game_state)
}