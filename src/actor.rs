use legion::*;
use super::*;

pub fn set_intent(ecs: &mut World, entity: Entity, intent: Intent) -> Result<(), GameError> {
    let entry = ecs.entry(entity).ok_or(GameError{})?;
    let old_intent = entry.into_component_mut::<Intent>()?;
    old_intent.action = intent.action;

    Ok(())
}
