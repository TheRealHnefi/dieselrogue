use specs::prelude::*;
use super::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (WriteExpect<'a, GameLog>,
                       ReadExpect<'a, Map>,
                       ReadStorage<'a, Name>,
                       WriteStorage<'a, HumanoidBody>,
                       WriteStorage<'a, Position>,
                       WriteStorage<'a, Damage>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut game_log, map, names, mut bodies, mut positions, mut damages) = data;

        for (damage, position) in (&mut damages, &mut positions).join() {
            //NYI
        }
    }
}