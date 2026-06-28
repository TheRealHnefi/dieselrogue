# DieselRogue

A roguelike set in a 1930's world, where technology is fuelled by oil and high voltage electricity.
The game has a single, large level which the player must escape to win, utilizing what equipment
and vehicles they find along the way.

There is no randomness in the game, except for map generation and enemy actions. Turns occur
simultaneously and actions are entirely deterministic.

The interface consists of traditional, ASCII-like tiles using a modified CP437 font.

The game is implemented in Rust, using RLTK.

## Screenshots

![Gameplay](doc/screenshot1.png)
*The main gameplay screen, with the map viewport and the status, inventory, equipment, and log panels.*

![Equipment](doc/screenshot2.png)
*The equipment menu, showing equipped items.*

![Level up](doc/screenshot3.png)
*Choosing an ability on level up, with the ability description panel.*

## Building

DieselRogue is a standard Rust project, so install Rust and execute *cargo run* from the project root.