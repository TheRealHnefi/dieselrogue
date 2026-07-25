use rltk::{Rltk, GameState, RGB};
use specs::prelude::*;

pub struct State {
    pub ecs: World
}

impl GameState for State {
    fn tick(&mut self, context: &mut Rltk) {
        context.cls();
        context.set(10, 10, RGB::named(rltk::RED), RGB::named(rltk::BLACK), rltk::to_cp437('A'));
    }
}