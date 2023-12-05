# DieselRogue design document

## Name ideas:
DieselRogue
Deco-x (decodence, decoventure, decogue)
Heavy Fuel
Rogue Noir

## Elevator pitch:
A roguelike set in a dieselpunk world where armed conflict is a constant threat.
Combat is deliberate and deadly in this game, and the effect of equipment, skills
and attributes is dramatic. No fiddling with single digit percentages; the choices
you make have a large impact on how you play.
Also, you get to blow things up with retrofuturistic tanks.

## Potential major direction:
No randomness beyond level generation and enemy actions. Everything you do has
predictable results. If you shoot, you know that you will hit and how much damage
you will do. The challenge comes from analyzing the situation and applying your
tools for maximum effect. You will never be screwed by RNG - except potentially
in level layout...

## Character:
### Stats:
No stats! It's just busywork to keep track of.

### Classes (pick any two?) and perk ideas:
No classes!

### Perk ideas

#### Automatic Weapons
* Burst
* Long burst (double/triple burst amount)
* Suppressive burst
* Overwhelm (burst at every body part of target)

#### Pistols
* Free shot
* Double tap
* Instant reload
* Disarming shot
* Mercy kill (instantly kill burning, blind or stunned enemy)

#### Rifles
* Penetrating shot (shoot through opponents)
* Target weakpoint (ignore armor)
* Disarming shot
* Execution (instantly kill unaware enemy)

#### Shotguns
* Shoot and reload
* Double tap
* Fan fire
* Blinding shot

#### Launchers
* Shaped charge (explode in area shaped away from user)
* Target structural weakpoint (destroy walls)
* Bigger booms
* Focused blast (bigger damage on only one bodypart)

#### Conditioning
* Double stamina
* Refuel stamina by spending HP
* Instant inventory management
* Wide vision

#### Mobility
* Dodge
* Dash
* Free movement (take X steps without spending turn)

#### Infiltration
* Pick lock
* Peek through keyhole
* Block door
* Sneak
* Hide
* Make noise (misdirect)
* Disguise

#### Scouting
* Eagle eyes (long distance sight)
* Wide vision
* Listening
* Spot hidden
* Predict action

#### Piloting
* Pilot vehicle
* Disable vehicle

#### Resilience
* Double HP
* Innate armor
* Tougher bodyparts
* Disregard debuff

#### Engineering
* Build trap
* Disarm trap
* Fit modifications
* Destructive modification (massively overpower weapon, destroy after x uses)
* Modify ammo
* Create grenade
* Create firebomb
* Combine two weapons to give one an ability or stat of the other? (seems waaaay OP. Does that matter?)

#### Chemistry
* Improved healing effect from medkits
* Create poison canister
* Create acid canister
* Create smoke canister
* Create KO gas canister
* Inject berzerkium

### Resources:
* HP: Local for each body part, plus overall HP. No regeneration.
* Fatigue: Used for abilities. No regeneration.

### Status effects:
* Blind: reduce vision range to 1
* Burning: Take fire damage on all body parts every turn
* Stuck: Cannot move
* Stunned: Cannot act except wait
* Aiming at target: Can fire on target next round if in line of sight and range
* Aiming at area: Can fire on any target in area next round
* Deaf: Cannot hear noise
* Dazed: Cannot spend fatigue on abilities
* Suppressed: Cannot fire or move

## Combat ideas:
* NO resource regeneration!
* The above leads to NO resting! No need for a hunger clock.
* Focus on active abilities and crowd control.
* Few enemies in a single battle. Enemies should spread out to search if alerted, allowing takedowns.
* Abilities and equipment should follow the half or double rule.
* Armour should be powerful. Use damage = (power - absorbtion) * resistance.
* Simple damage calculations! No complicated formulas - make it intuitive.
* Localized damage for body parts.
* Heavy recoil weapons removes aiming status. Low recoil weapons keeps it.

## Stealth:
* All actors have a viewing direction. Turning takes an action. 180 degree FOV for most.
* Listening can give approximate location of enemies.
* Backstabbing unaware opponent is guaranteed kill. The risk is getting to that point.

## Tinkering:
* Add/remove attributes on items.
* Build completely new items, competitive with found items.
* Construct consumables, but never infinites.

## Rounds:
All actions are one of the following categories:

* Static phases:
 * Update viewsheds
 * Monster decides next action
 * Take input and designate actions for player
 * Resolve out-of-order actions (instant abilities or time manipulation)
  * Potentially return to start - if movement occurs, need to reassess entire situation
 * Resolve inventory actions (get/drop/equip/use)
 * Resolve attacks (including throwing items)
 * Resolve movement
 * Resolve other actions (are there any?)
 * Resolve status effects
 * Cleanup?

## Misc:
* Multi-tile actors, like mechs and tanks - pilotable!
* Fluid mechanics. Possible to have fuel leak and ignite. Precursor to fortress mode?
* Items that give active abilities, but never overlap with perks abilities

## Projectile Weapons:
### Examples:
* Pistol
 * Dart gun
 * 9mm sidearm
 * Army .45
 * Hand cannon
* Submachine gun
 * Tommy gun
 * 'KSP'
 * Unique: Kummerlauf
* Shotgun
 * Pump action
 * Double barrel
 * Sawed off
 * Combat shotgun
 * Unique: Flamebelcher
* Machine gun
* Rifle
* Launcher
 * Grenade launcher
 * Rocket launcher
 * Minilauncher
* Flamethrower
 * Flamethrower
 * Napalm thrower
 * Acid thrower
* Tesla

### Modifiers:
* Drum magazine (triple ammo count)
* Long range optics (inverse range penalty)
* Good sights (+25% chance to hit)
* Silent
* Efficient construction (half weight)
* Long barrel (+25% damage)
* Recoil compensator (allow burst fire on bodypart)
* Heatseeker (+50% chance to hit)
* Bayonet (allows melee attacks)
* Gasoline injector (blinds opponents when firing, +25% damage, extremely loud)
* Bipod (+25% chance to hit when taking single shot)

## Scenario:
World is 1950's, parallel earth.

### Mission option 1:
Break into the military development base, filled with guards and other personnel. Objective is to
stop or steal the development of new weapons (airplanes? tanks? mechs? all of the above?).

The Castle consists of large battlements with many rooms and a central courtyard. In the middle,
the head scientist is waiting.

### Mission option 2:
Break out of the prison by first getting out of your cell. There are guards everywhere and you need
to explore to find the tools necessary to escape.

You start in the middle of the map, which is made up by many blocks of buildings. You escape by
opening one of locked gates at the edges of the compound.

### Map:
1000x1000 in size, made up up 100 100*100 blocks. Each block has its own layout logic - some blocks
are prison complexes, some are huge singular buildings such as hangars, some are open areas for
target practice, some are filled with many small buildings etc. Some blocks have open edges, some
are walled in.

Some blocks may form large roads through the map. This metalayer of organization may be added later,
but it's possible this one floor layout will work regardless.

# UI design

Key concepts:
* Important info at a glance
 * Stuff that changes often
 * Stuff that has immediate effects on play
* Clearly labelled information always
* Detailed info on separate screens
* Comparisons between pieces of equipment

Sum of all stuff to show:
* Event log
* Noise log/status
* Body parts:
 * Health
 * Armor
 * Equipment
* Fatigue
* Inventory
 * Item names
 * Item descriptions
 * Item stats
* Equipped items
 * Weapons
 * Armor
 * Item stats
 * Item names
 * Empty slots
 * Item descriptions
* Abilities
 * Ability names
 * Ability descriptions
 * Ability hotkeys
* Status effects
 * Status effect names
 * Status effect descriptions
 * Status effect durations
* Enemy information
 * Field of view
 * Intent
 * Description?
 * Health
 * Equipment
* Square information
 * Walls
 * Doors
 * Items
* Location information
 * Name of area
 * Coordinates

## At-a-glance data
* Event log
* Noise status
* Body parts:
 * Health
* Fatigue
* Status effects
* Location information

## Menu system

### Action examples
* Instant action: move, turn
* Instant item use: equip, pick up
* Instant equipped item use: unequip, use medkit
* Apply action on map: jump, teleport, look
* Apply item on map: aim, throw
* Apply item on detailed targeting: fire (limited by area aimed at)

### Action types
* Instant(optional action, item, equipped item)
 * Pick source
* Targeting (optional action, item, equipped item)
 * Pick source
 * Pick target tile
* Detailed targeting (optional action, item, equipped item)
 * Pick source
 * Pick target tile
 * Pick target bodypart