#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Ability {
    // Passive abilities
    HumanMove,
    VehicleMove,
    PickUp,
    // Active abilities
    Embark,
    Disembark
}

impl Ability {
    pub fn to_string(&self) -> String {
        match self {
            Ability::HumanMove => "Move",
            Ability::VehicleMove => "Drive",
            Ability::PickUp => "Pick Up",
            Ability::Embark => "Embark",
            Ability::Disembark => "Disembark"
        }.to_string()
    }
}