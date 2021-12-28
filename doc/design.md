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
* Agent: Trenchcoat-wearing government operative. Specializes in ranged combat.
 - aim, scout, controlled burst, fast reload, dodge
* Pugilist: Unstoppable strongman. Specializes in crowd control and unarmed combat.
 - disarm, stun, throw, dash, grab and hold, block
* Burglar: Silent protagonist. Goes unseen before attacking. Relies on stealth.
 - predict movement, misdirection, lockpicking, backstab, sneak, tumble
* Extropicist: Curious magician. Aims to uncover unearthly secrets. Relies on magic.
 - circles of magic, sense magic
* Engineer: Practical intellectual. Relies on versatile equipment.
 - build consumables, handle traps, enhance equipment, assemble, disassemble
* Pilot: 

Or perhaps no classes at all? Just perks?

### Perk ideas

#### Automatic Weapons
* Controlled burst (burst at body part)
* Long burst (double/triple burst amount)
* Suppressive fire
* Recoil handling (better accuracy of subsequent shots in burst)
* Dual wielding submachine guns

#### Chemistry
* Improved healing effect from medkits
* Create poison canister
* Create acid canister
* Create smoke canister
* Create KO gas canister
* Inject berzerkium

#### Conditioning
* Double stamina
* Refuel stamina by spending HP
* Instant inventory management

#### Engineering
* Build trap
* Disarm trap
* Fit modifications
* Destructive modification (massively overpower weapon, destroy after x uses)
* Rocket-powered dash

#### Explosives
* Accurate throw
* Bigger booms
* Booby trap
* Create grenade
* Create firebomb

#### Infiltration
* Pick lock
* Peek through keyhole
* Block door
* Thorough body search
* Predict action
* Sneak
* Hide
* Make noise (misdirect)
* Disguise
* Precision throwing

#### Launchers
* Safe shot (only fire if explosion does not harm user)
* Shaped charge (explode in area shaped away from user)
* Rapid reload

#### Melee

#### Mobility
* Dodge
* Dash
* Slide trip (Bison low roundhouse)
* Free movement (take X steps without spending turn)

#### Pugilism
* Triple punch
* Dashing punch
* Throw opponent
* Knock out
* Disarm
* Trip
* Strike weakspot (eyes if face, kidneys if torso, groin if legs etc)
* Break bodypart (neck, arm, foot)
* Free strike

#### Piloting

#### Pistols
* Free shot
* Double wielding
* Melee barrage
* Double tap
* Human shield

#### Resilience
* Double HP
* Innate armor
* Tougher bodyparts
* Disregard debuff
* Revive
* Gun-fu (fire at all visible opponents)

#### Rifles
* Aim
* Penetrating shot (shoot through opponents)
* Instant reload
* Crippling shot
* Shoot weakpoint

#### Scouting
* Eagle eyes (long distance sight)
* Listening
* Spot hidden

#### Shotguns
* Shoot and reload
* Double shot
* Deafen
* Shower power (fire several shots in area, hit several targets)

### Resources:
* HP: Local for each body part, plus overall HP. No regeneration.
* Fatigue: Used for abilities, including magic. No regeneration.
* Should there be a mana resource?

## Combat ideas:
* NO resource regeneration!
* The above leads to NO resting! No need for a hunger clock.
* Focus on active abilities and crowd control.
* Few enemies in a single battle. Enemies should spread out to search if alerted, allowing takedowns.
* Abilities and equipment should follow the half or double rule.
* Armour should be powerful. Use damage = (power - absorbtion) * resistance.
* Simple damage calculations! No complicated formulas - make it intuitive.
* Localized damage for body parts.

## Stealth:
* All actors have a viewing direction. Turning takes an action. 180 degree FOV for most.
* Listening can give approximate location of enemies.
* Backstabbing unaware opponent is guaranteed success. The risk is getting to that point.

## Magic:
* Glyph-based. Finding a glyph lets player name it - they don't know what it actually does.
* Spells are glyph combinations. Since glyphs are named by the player, spells need to be discovered.
* Known spells can be shortcutted, like any other ability.
* Extropy-based

## Tinkering:
* Add/remove attributes on items.
* Build completely new items, competitive with found items.
* Construct consumables, but never infinites.

## UI:
* Need comprehensive shortcut system.
* Need way to see character facing and localized damage.
* When targeting: pick target first, then pick ability from list of applicable options

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