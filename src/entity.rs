use crate::components::*;

#[derive(Copy, Clone)]
pub struct Player {
    /// Most of the actual data is stored in the World. It is found by using the index as reference.
    pub index: usize,
    pub facing: Facing
}

