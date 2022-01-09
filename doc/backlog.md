* Most items
* Refactor item abilities and innate abilities
 * Make abilities simple enums
* Add a few non-item abilities
* UI redesign
 * Minimap
 * Better inventory screen
 * Better ability menus
 * Show outcome of actions
 * Show more information about enemies and world
 * Optimize map drawing (RLTK batched rendering does not seem to help much. Other approaches?)
 * Use different fonts for different resolutions to fix scaling issues
* Noise (remember, only the noisiest noise is heard by most characters)
* Map building
 * Prisons
 * Barracks
 * Hangars
 * Armories
 * Firing ranges
 * Roads
 * Undestroyable walls
 * Single-building maze blocks
 * Gate blocks
 * Metalayer - paths, locked access, etc
* Vehicles
 * Cannon for tank
 * Mech with machineguns
 * Disembark via menu
* Doors
 * Unlockable
 * Openable
* Animations
 * Screen culling
* Aiming
 * Playtest - should I really use this aiming system?
 * Make entity able to aim at other entities (not just positions)
 * Make weapons only aimable at visible targets
 * Break aim when moving/unequipping/losing visibility/etc
 * Make weapons fireable only on targets aimed at
 * Make aiming at cone/area possible
* Weapons
 * Limited range
 * Fire only in FoV
* Limit inventory size to 20
* Refactor player functions
* Refactor menu system (oh my god it sucks!)
* Refactor action resolution
* Death effects (drop items, corpses etc, game over screen)
* Fluids (maybe)
* Most abilities
* Create AI's
 * Create new pathfinding system
 * Put AI's on worker thread
* Settings
* Savegames
* New game screen
 * Character creation
* Character progression (leveling)
* Optimizations!
 * State transition short-circuit when rerender is not needed
 * Bulk rendering
* Web!
* GFX!
* Sounds!
* A pony!