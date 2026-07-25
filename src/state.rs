use rltk::{Rltk, GameState};
use specs::prelude::*;
use super::*;

pub struct State {
    pub ecs: World
}

impl GameState for State {
    fn tick(&mut self, context: &mut Rltk) {
        context.cls();
        
        player_input(self, context);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            context.set(pos.x, pos.y, render.color, render.background, render.glyph);
        }
    }
}