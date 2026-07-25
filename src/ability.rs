#[derive(Clone, Eq, PartialEq, Hash, strum::EnumIter)]
pub enum Ability {
    // Passive abilities
    HumanMove,
    VehicleMove,
    PickUp,
    WideVision,
    Precognition,
    IronEars,
    EagleEyes,
    Pugilism,
    Backstab,
    Stealth,
    Tough,
    // Active abilities
    Embark,
    Disembark,
    Throw,
    Juke, // Allow player to move out of order and without turning. Allows dodging shots.
    IronBody,
}

impl Ability {
    pub fn to_string(&self) -> String {
        match self {
            Ability::HumanMove    => "Move",
            Ability::VehicleMove  => "Drive",
            Ability::PickUp       => "Pick Up",
            Ability::WideVision   => "Wide Vision",
            Ability::Precognition => "Precognition",
            Ability::IronEars     => "Iron Ears",
            Ability::EagleEyes    => "Eagle Eyes",
            Ability::Pugilism     => "Pugilism",
            Ability::Backstab     => "Backstab",
            Ability::Stealth      => "Stealth",
            Ability::Tough        => "Tough",
            Ability::Embark       => "Embark",
            Ability::Disembark    => "Disembark",
            Ability::Throw        => "Throw",
            Ability::Juke         => "Juke",
            Ability::IronBody     => "Iron Body",
        }.to_string()
    }

    pub fn is_innate(&self) -> bool {
        match self {
            Ability::HumanMove   => true,
            Ability::VehicleMove => true,
            Ability::PickUp      => true,
            Ability::Disembark   => true,
            Ability::Throw       => true,
            Ability::IronBody    => false,
            _                    => false,
        }
    }

    pub fn is_passive(&self) -> bool {
        match self {
            Ability::HumanMove    => true,
            Ability::VehicleMove  => true,
            Ability::PickUp       => true,
            Ability::WideVision   => true,
            Ability::Precognition => true,
            Ability::IronEars     => true,
            Ability::EagleEyes    => true,
            Ability::Pugilism     => true,
            Ability::Backstab     => true,
            Ability::Stealth      => true,
            Ability::Tough        => true,
            Ability::Embark       => false,
            Ability::Disembark    => false,
            Ability::Throw        => false,
            Ability::Juke         => false,
            Ability::IronBody     => false,
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
            Ability::IronEars =>
                "You are able to distinguish individual sounds even in a very noisy environment, greatly increasing your noise tolerance.",
            Ability::EagleEyes =>
                "Your viewing range is increased by 50%.",
            Ability::Pugilism =>
                "Your melee strikes target the head instead of the torso.",
            Ability::Backstab =>
                "Melee attacks against humans who cannot see you deal 400% increased damage.",
            Ability::Stealth =>
                "Your movement makes no sound.",
            Ability::Tough =>
                "All of your body parts have 50% more hit points.",
            Ability::Embark =>
                "Allows entering a vehicle. Move into a drivable vehicle to take the controls.",
            Ability::Disembark =>
                "Allows exiting a vehicle you are currently driving.",
            Ability::Throw =>
                "Allows throwing items such as grenades.",
            Ability::Juke =>
                "Move instantly out of turn order. Costs 25 energy. Useful for dodging incoming fire.",
            Ability::IronBody =>
                "Activate to harden your body for 3 turns, increasing physical resistance by 50%.",
        }
    }

    /// The body part index (into human_body parts) where this ability is stored when gained.
    pub fn default_body_part(&self) -> usize {
        match self {
            Ability::WideVision   => 0, // Head
            Ability::Precognition => 0, // Head
            Ability::IronEars     => 0, // Head
            Ability::EagleEyes    => 0, // Head
            Ability::Pugilism     => 2, // R. Arm
            Ability::Backstab     => 2, // R. Arm
            Ability::VehicleMove  => 1, // Torso
            Ability::Disembark    => 1, // Torso
            Ability::PickUp       => 2, // R. Arm
            Ability::Embark       => 2, // R. Arm
            Ability::Throw        => 2, // R. Arm
            Ability::HumanMove    => 4, // Legs
            Ability::Juke         => 4, // Legs
            Ability::Stealth      => 4, // Legs
            Ability::Tough        => 1, // Torso
            Ability::IronBody     => 1, // Torso
        }
    }
}
