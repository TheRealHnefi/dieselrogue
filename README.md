# DieselRogue

A diesel-punk roguelike written in Rust with an ASCII terminal UI.

## Concept

DieselRogue combines traditional roguelike mechanics with vehicle combat in a diesel-punk setting. Players pilot vehicles, manage character abilities, and engage in turn-based tactical combat.

## Features

- **Vehicle combat**: Embark/disembark from vehicles (tanks, etc.) with distinct movement mechanics
- **Firearms**: Ranged combat with weapons (revolvers, pistols) featuring damage, range, and ammunition
- **Ability system**: Entities have capabilities (HumanMove, VehicleMove, etc.) that determine available actions
- **Inventory & equipment**: Multi-slot equipment system with items assignable to body parts
- **Status effects**: Dynamic effects applied during combat resolution
- **Animations**: Visual feedback framework for actions
- **Procedural maps**: Block-based map generation with varied terrain

## Architecture

Turn-based intent-resolution loop: `DeclareIntent → Resolve → RenderAnimations → ResolveStatusEffects`

- **World**: Central game state managing entities, map, and items
- **Entity**: Players, enemies, and vehicles with bodies, abilities, and inventory
- **Systems**: AI, animation, and action resolution
- **Input/Menu**: Player input handling and menu navigation

## Technology

- **Language**: Rust (v0.1.0)
- **Rendering**: RLTK (Rust LibTcod) with CP437 font
- **Serialization**: Serde
