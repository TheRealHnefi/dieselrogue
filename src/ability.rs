#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Ability {
    // Passive abilities
    HumanMove,
    VehicleMove,
    PickUp,
    WideVision,
    Precognition,
    // Active abilities
    Embark,
    Disembark,
}

impl Ability {
    pub fn to_string(&self) -> String {
        match self {
            Ability::HumanMove  => "Move",
            Ability::VehicleMove => "Drive",
            Ability::PickUp     => "Pick Up",
            Ability::WideVision   => "Wide Vision",
            Ability::Precognition => "Precognition",
            Ability::Embark     => "Embark",
            Ability::Disembark  => "Disembark",
        }.to_string()
    }

    pub fn is_passive(&self) -> bool {
        match self {
            Ability::HumanMove  => true,
            Ability::VehicleMove => true,
            Ability::PickUp     => true,
            Ability::WideVision   => true,
            Ability::Precognition => true,
            Ability::Embark     => false,
            Ability::Disembark  => false,
        }
    }
}