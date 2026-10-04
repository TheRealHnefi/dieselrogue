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

## Patrolling guard (profile = Patrol, combat_tactic = Pursue)

Walks a fixed route until something draws its attention, then commits hard: chases
the intruder while it can see them, and once it loses them raises the alarm and
sweeps an ever-widening area, shouting again as it goes. Unlike the sentinel it has
no anchor to hold and never mills in place — it either patrols, pursues, or searches.

Always:
    Primed grenade is carried:
        Throw grenade(enemy)!
    Active grenade nearby:
        Flee(grenade position)!

Unaware:
    Armed:
        Unequip weapon!
    Follow patrol route!

Suspicious:
    Unarmed or out of ammo:
        Equip or reload weapon!
    Suspicion faded:
        Decay to Unaware!
    Investigate area(cause of concern)!

Alert:
    Unarmed or out of ammo:
        Equip or reload weapon!
    Time to raise the alarm again:
        Shout!
    Search area(last known position)!

Combat:
    Unarmed or out of ammo:
        Equip or reload weapon!
    Enemy is within throw range and a grenade is carried:
        Prime grenade!
    Can see enemy:
        Attack(enemy)!
    Not yet at last known position:
        GoTo(last known position)!
    Decay to Alert, Shout!

## Pilot (profile = Pilot, bound to one vehicle)

A crewman posted beside a parked tank. It lounges by the vehicle until it learns of a
confirmed threat, then runs for the tank and fights from inside it. The tank is its
whole job: it never strays far from it on foot, and once aboard it never gets out.
Killing the pilot before it reaches the tank leaves the tank empty, so a player with
Embark can steal it. A tank that is already crewed can only be destroyed.

The pilot knows Embark innately. Its tree has two halves: on foot (the pilot's own
body) and driving (the tank's body, with the tank's narrow 90° view and slow turning).

It boards only on a confirmed threat, so mere suspicion can be used to lure it away
from the tank — but never far: "near tank" is a short leash, and past it the pilot
gives up and returns. It never shouts. Instead, the engine of a manned tank is heard
by other guards as a suspicious noise, so an alerted pilot that reaches its tank
draws the curious toward it.

### On foot

Always:
    Primed grenade is carried:
        Throw grenade(enemy)!
    Active grenade nearby:
        Flee(grenade position)!

Unaware:
    Armed:
        Unequip weapon!
    Near tank:
        95%: Idle!
        5%: Rotate(random)!
    GoTo(tank)!

Suspicious:
    Unarmed or out of ammo:
        Equip or reload weapon!
    Near tank:
        Investigate area(cause of concern)!
    Decay to Unaware, GoTo(tank)!

Alert and Combat:
    Tank is gone (destroyed or stolen):
        Act as a patrolling guard (Alert/Combat branches above)!
    Enemy is adjacent:
        Attack(enemy)!
    Board tank!

### Driving

A driving pilot is always Alert or Combat: it only boards on a confirmed threat, and
it never forgets one.

Alert:
    Search area(last known position)!

Combat:
    Can see enemy:
        Attack(enemy)!
    Can turn to see last known enemy position:
        Rotate(towards enemy position)!
    Not yet at last known position:
        GoTo(last known position)!
    Decay to Alert!

## Perception (shared by all profiles)

What moves a guard between alert states. A confirmed threat is never forgotten:
nothing decays from Alert or Combat to Unaware. Only unconfirmed suspicion fades.

Seeing the player:
    Fills a detection meter each turn in view (faster when closer, instant within
    3 tiles, doubled when Alert, instant in Combat). Drains one step per turn out of view.
    Meter full:
        Combat(player)!
    Meter partly full:
        Suspicious(player position)! (a glimpse — unconfirmed)
Hearing:
    Player footsteps:
        Suspicious(sound position)! (footsteps of other guards are ignored)
    Vehicle engine (any driver, including other guards):
        Suspicious(sound position)!
    Gunshot, burst, explosion or shout:
        Alert(sound position)!
Seeing:
    A body:
        Alert(body position)!
    A door the player left open:
        Suspicious(door position)!

Priority when several happen at once: confirmed threats beat unconfirmed ones, then
seen beats heard. An Alert guard only reacts to seeing the player (anything else would
restart its search).

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

### Follow patrol route
At current waypoint:
    Advance to next waypoint!
GoTo(current waypoint)!

### Search area(origin)
Far from origin:
    GoTo(origin)!
At the current search point (or none picked yet):
    Pick a new search point around origin, wider than the last!
GoTo(search point)!

### Prime grenade
Prime the carried grenade! (thrown next turn by the Always block)

### Shout
Raise the alarm! (a loud shout heard by nearby guards, putting them on alert)

### Board tank
Adjacent to tank:
    Walk into tank! (embarks; the pilot now drives)
GoTo(tank)!

### While driving: Rotate and GoTo
A tank turns only 45° per turn, so Rotate(direction) turns one step toward direction
and GoTo turns before each move just like on foot. A tank cannot open doors or squeeze
through narrow gaps; if the path is blocked it waits (Idle!) rather than bumping.

### Open door
Pathfinding leads through door:
    Facing door:
        Open door!
    Face door!
