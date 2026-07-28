# Decision tree for AI

Condition ends with :
Decision ends with !

Tree always ends in !. If condition does not match, go to next branch.

## Sentinel guard (profile = Guard, combat_tactic = Hold)

Always:
	Primed grenade is carried:
        Throw grenade(enemy)!
	Active grenade nearby:
		Flee(grenade position)!

Unaware:
	At anchor:
        Armed:
            Unequip weapon!
		95%: Idle!
		5%: Rotate(random)!
	GoTo(anchor)!

Suspicious:
	Unarmed or out of ammo:
		Equip or reload weapon!
	Near anchor:
		Investigate area(cause of concern)!
    Far from anchor:
		Decay to Unaware, GoTo(anchor)!

Alert:
    Unarmed or out of ammo:
        Equip or reload weapon!
    Near anchor:
        50%: Rotate(random)!
        25%: GoTo(1 step forward)!
        25%: Idle!
    Far from anchor:
        GoTo(anchor)!

Combat:
    Unarmed or out of ammo:
        Equip or reload weapon!
    Near anchor:
        Can see enemy:
            Attack(enemy)!
        Last known enemy position is in view:
            GoTo(halfway to last known position)!
        Can turn to see last known enemy position:
            Rotate(towards enemy position)!
        Can find nearby position to spot last known enemy position:
            GoTo(spot)!
    Far from anchor:
        Can see enemy:
            Attack(enemy)!
        Last known enemy position is in view:
            GoTo(halfway to last known position)!
        Can turn to see last known enemy position:
            Rotate(towards enemy position)!
        Decay to Alert, GoTo(anchor)!

## Decisions, detailed into actions

### Throw grenade(target)
Primed grenade is carried:
    Target is visible:
        Target is in range:
            Target is at a safe distance:
                Throw grenade at target!
            Throw grenade near target at safe distance!
        Throw grenade as close to target as possible!
    Safe spot exists within range:
        Throw grenade at safe spot!
    Throw grenade as far away as possible!
Target position is visible:
    Prime grenade!
GoTo(target)

### Flee(position)
Move away from position! (no pathfinding, just find adjacent tile that moves away from position if possible)

### Unequip weapon
Weapon in hand:
    Unequip weapon!
Idle!

### Equip or reload weapon
Weapon in hand:
    Weapon not full:
        Reload!
    Idle!
Weapon in inventory:
    Equip weapon!

### Investigate area(position)
Position is in view:
    Investigate area(near position)!
Can turn to see position:
    Turn(towards position)!
GoTo(position)!
    
### GoTo(position)
Pathfind to position!
### Rotate(direction)
Turn(dirction)!
### Idle
Do nothing!
### Attack(entity)
Entity in range:
    Aiming:
        Fire!
    Aim at entity!
GoTo(position close enough to entity to fire)!