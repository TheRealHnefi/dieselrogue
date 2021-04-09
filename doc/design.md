# DieselRogue design document

## Name ideas:
DieselRogue
Decodence

## Elevator pitch:
A roguelike set in a dieselpunk world where armed conflict is a constant threat.
Combat is deliberate and deadly in this game, and the effect of equipment, skills
and attributes is dramatic. No fiddling with single digit percentages; the choices
you make have a large impact on how you play.
Also, you get to blow things up with retrofuturistic tanks.


## Character:
### Stats:
3 base stats (but are stats even necessary?):
* Physical - shooting, punching, taking damage, sneaking
* Intellectual - tinkering, spotting, listening, trapping
* Spiritual - magic, willpower, bravery

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
* Static phases:
 * Take input
 * Resolve out-of-order actions (special abilities or time manipulation)
 * Resolve inventory actions (pick up/drop/equip)
 * Resolve melee attacks
 * Resolve ranged attacks
 * Resolve magic
 * Resolve movement
 * Resolve other actions
 * Monster decides next action

 ## Misc:
 * Multi-tile actors, like mechs and tanks - pilotable!
 * Fluid mechanics. Possible to have fuel leak and ignite. Precursor to fortress mode?

## Scenario:
World is 1950's, parallel earth.

Break into the military development base, filled with guards and other personnel. Objective is to
stop or steal the development of new weapons (airplanes? tanks? mechs? all of the above?).

The Castle consists of large battlements with many rooms and a central courtyard. Various towers
are spread out over the property and bridges cross over the courtyard high above. At the very top,
the head scientist is waiting.

Once killed, the catacombs open (get key from corpse?) and out pours various occult and/or mechanical monsters.
The protagonist has to fight through the monsters, find the laboratory in the old dungeons, and activate
the self-destruct (or plant the bomb, whatever). Get out before she blows up!