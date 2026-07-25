/// The basic type of thing that can exist in the world.
#[derive(Copy, Clone)]
pub struct Entity {
    /// Most of the actual data is stored in the World. It is found by using the index as reference.
    pub index: usize,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            index: 0
        }
    }
}