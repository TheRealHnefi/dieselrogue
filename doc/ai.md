# Decision tree for AI

Condition ends with :
Decision ends with !

Tree always ends in !. If condition does not match, go to next branch.

## Sentinel guard (profile = Guard, combat_tactic = Hold)

Always:
	Active grenade in inventory:
		Enemy visible:
			Throw grenade(enemy)!
		Throw grenade(safe spot)!
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