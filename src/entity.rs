/// The basic type of thing that can exist in the World structure.
#[derive(Copy, Clone)]
pub struct Entity {
    /// Most of the actual data is stored in the World. It is found by using the index as reference.
    pub index: usize,
    pub entity_type: EntityType
}

impl Entity {
}

#[derive(Copy, Clone)]
pub enum EntityType {
    Player(Player),
    Actor(Actor),
    Item(Item)
}

#[derive(Copy, Clone)]
pub struct Player {

}

#[derive(Copy, Clone)]
pub struct Actor {

}

#[derive(Copy, Clone)]
pub struct Item {

}