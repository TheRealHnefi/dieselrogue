/// Reference to the basic type of thing that can exist in the World structure.
#[derive(Copy, Clone)]
pub struct EntityEntry {
    /// Most of the actual data is stored in the World. It is found by using the index as reference.
    pub index: usize,
}

pub struct Intent {
    
}