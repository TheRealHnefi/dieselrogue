#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Ability {
    // Passive abilities
    Move,
    PickUp,
    // Active abilities
    Disembark
}

impl Ability {
    pub fn to_string(&self) -> String {
        match self {
            Ability::Move => "Move",
            Ability::PickUp => "Pick Up",
            Ability::Disembark => "Disembark"
        }.to_string()
    }
}