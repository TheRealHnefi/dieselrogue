use specs::prelude::*;
use super::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (WriteExpect<'a, GameLog>,
                       ReadStorage<'a, Name>,
                       WriteStorage<'a, HumanoidBody>,
                       WriteStorage<'a, Damage>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut game_log, names, mut bodies, mut damages) = data;

        for (body, damage, name) in (&mut bodies, &mut damages, &names).join() {
            for dmg in &damage.instances {
                body.hitpoints = body.hitpoints - dmg.phys;
                game_log.entries.push(format!("{} damage done to {}", dmg.phys, name.value));
            }
        }

        damages.clear();
    }
}