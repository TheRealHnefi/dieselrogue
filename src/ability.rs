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
    Juke, // Allow player to move out of order and without turning. Allows dodging shots.
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
            Ability::Juke       => "Juke",
        }.to_string()
    }

    pub fn is_innate(&self) -> bool {
        match self {
            Ability::HumanMove  => true,
            Ability::VehicleMove => true,
            Ability::PickUp     => true,
            Ability::Disembark  => true,
            _                   => false,
        }
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
            Ability::Juke       => false,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Ability::HumanMove =>
                "Allows movement on foot.",
            Ability::VehicleMove =>
                "Allows driving a vehicle. Required to drive tanks and other motorized units.",
            Ability::PickUp =>
                "Allows picking up items from the ground.",
            Ability::WideVision =>
                "Expands your field of view to 270 degrees, leaving only a small blind spot directly behind you.",
            Ability::Precognition =>
                "When examining an entity with the look command, you can see what action they are planning to take this turn.",
            Ability::Embark =>
                "Allows entering a vehicle. Move into a drivable vehicle to take the controls.",
            Ability::Disembark =>
                "Allows exiting a vehicle you are currently driving.",
            Ability::Juke =>
                "Move instantly out of turn order. Costs 25 energy. Useful for dodging incoming fire.",
        }
    }

    /// The body part index (into human_body parts) where this ability is stored when gained.
    pub fn default_body_part(&self) -> usize {
        match self {
            Ability::WideVision   => 0, // Head
            Ability::Precognition => 0, // Head
            Ability::VehicleMove  => 1, // Torso
            Ability::Disembark    => 1, // Torso
            Ability::PickUp       => 2, // R. Arm
            Ability::Embark       => 2, // R. Arm
            Ability::HumanMove    => 4, // Legs
            Ability::Juke         => 4, // Legs
        }
    }
}