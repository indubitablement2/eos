# EOS (working title)


###### Table of contents

[Income](https://github.com/indubitablement2/eos#Income)

[User Experience](https://github.com/indubitablement2/eos#user-experience)

[Fleet](https://github.com/indubitablement2/eos#Fleet)

[World](https://github.com/indubitablement2/eos#World)

[Inspiration](https://github.com/indubitablement2/eos#Inspiration)

## Income Free to play

* Sell custom ship ~$1000:
  * People can buy the possibility to create a custom ship.
  * This ship will be found in-game, but much rarer than the default variant only found as quest reward, abandonned ship or in ai fleet (which they can defeat and salvage).
  * They chose a base ship and customize some of it's property:
  * Name: My Super Custom Ship, Custom Harrier class.
  * Description.
  * Built-in weapon and module.
  * Weapon mount.
  * Skin.
  * (no guarantee this ship will even work)
* Sell fleet slot ~$8
  * Can only control one fleet by default. Additional slot can be bought.
* Sell cargo space $2 - $8
  * Limited number of items and ships can be stashed. Special cargo that have better visibility and qol.
* Sell ship skin $4 - $20
  * Special ship skin or skin transfer for a particular ship class.
* Weapon effects ~$2
  * Alter the effect of a weapon without making it outlandish.
* Character portrait pack ~$4
  * Free character portrait looks like normal civilian. Paid ones can be more noticeable.

### Server cost
Estimate US$360/months, US$4320/years.

* (US$170/months, US$2040/years) c6g.4xlarge, 16 arm cpus, 32gb ram, up to 10gbs network.
* (US$50/months, US$600/years) 300gb ssd with backup.
* (US$140/months, US$1680/years) 1.5tb internet per months.


## User Experience

### Core loop

Explore - Fight - Customize (gradual fleet upgrade)

### Decay and entropy
Ship need to be rebuilt and maintained.

This keep the world fresh. New player are not overwhelmingly behind when joining later.

This push player to utilise their wealt.

#### Resource storage
* Fleet cargo: 
  * Need ship with cargo capacity. 
  * Ends up being the most expensive. 
  * Immediately available.
* Hangar: 
  * Limited. Cost real money to increase. 
  * Items stay at the station they were stashed forever.
* Cargo pod:
  * Free.
  * Lootable by anyone
  * Will stay there indefinitely until looted by someone.
  * High value cargo attracts pirate. Low value cargo gets combined together.

### Minimal industry flexibility
When a planet is invested into (for example) ore refining, it is impractical to switch industry in response to market demand.

This lead to surplus or scarsity increasing player activity:
* Building new needed industry. 
* Hauling resource from suplus area to scarsity area. 
* Fight over scarse resource. 
* Fight for monopoly.

### No standard currency
This make it harder to stockpile resource in conjonction with entropy.

Players get into a loop of always needing more of x resource while having too much of y.
This lead to player engagement through trading.

When shopping at a colony, the cost of item is in generic value.
The player can buy by offering at least as much value as what he is buying.

Resources value is determined by its local availability.

### Acquiring ship/weapon/module
Low tier ship and civilian ship are available everywhere.

Getting higher tier gear is hard:
* Ranking inside a faction (still need resources to buy, only give access to this faction's ethic gear).
* Post battle salvage (needs to be repaired).
* Random wreck and cargo in space.
* Quests.

## Fleet & ships

### Loot randomness

Ship:
* Static/based on ship class: 
  * Base stats (hull, armor, energy capacity)
  * Number, position and size of weapon slots.
* Semi-random/weighted by ship class: 
  * Weapon and module slot manufacturer requirement.
  * Built in modules.
* Random:
  * Number of weapon module slot.
  * Number of ship module slot.

Weapon:
* Rate of fire
* Salvo size
* Energy cost
* Projectile speed
* Projectile damage
* Number of projectile

Captain:
* Skills (passive skill tree from poe)

### Hull size
* Fighter: 
  * 1-2 light weapons.
  * The size of 1 heavy weapon.
  * No ship collision.
  * Depend on carrier to refill ammo and repair.
  * Infinitely produced by carrier for free.
  * Moves like a plane with poor turn rate and lightning speed.

* Frigate: 
  * ~4 light or 1 medium weapons.
  * The size of 4 fighters.
  * High turn rate and good speed.

* Destroyer:
  * 4 medium weapons.
  * The size of 3 frigates.

* Cruiser: 
  * 4 medium and 2 heavy weapons.
  * The size of 3 destroyers.
  * Slow turn rate.

* Capital: 
  * 4 heavy and many medium/light weapons.
  * Often has unique built-in weapon.
  * The size of 3 cruisers or more.
  * Extremely expensive to buy and maintain.
  * Very slow.

### Manufacturer
Weapon and ship have manufacturer. Some ship and weapon mount may have an afinity toward a manufacturer. 

Module and weapon slots generaly only accept a pacticular manufacturer. This is semi-random and hard to modify.

### Weapons
* Light:
* Medium:
* Heavy:

### Modules
They can be built-in.

Player installed mods gives bonus (better shield, more energy storage, faster movement, increased range, ...) at the cost of reduced energy production.

Ships have a limited number of free module slot. Higher quality ship have more.

Damaged ship have negative built-in mods that are expensive to remove.

## World

### Ethic archetype
* UEF: Brute force. More frontal firepower and defence, but slower. Vulnerable when flanked or isolated. 
  * Theme: bulky, see Imperium of Man from 40k, human elitist.
  * Music: 
  * Specialist: Armor, balistic

* Cybran: Unconventional tactic (stealth, suicide ship), prioritise offence. Worse quality, cheaper.
  * Theme: spiky, punk, synthetic.
  * Music: darksynth, https://soundcloud.com/beasuce/bereavement
  * Specialist: Speed, missile

* Aeon: Ship specialize in one thing. Better at their primary task and worst at everything else. Slightly better quality and more expensive. 
  * Theme: round, fragile noble, religious elitist.
  * Music: synth
  * Specialist: Range, fighter

* Seraphim: High quality multi-purpose ship, less numerous. One ship is equivalent to 1.5 ships of the same size from other faction.
  * Theme: round asymmetric
  * Music: 
  * Specialist: Shield, energy

## Inspiration

* Starsector:
  * Battle
* Path of Exile:
  * Customization
* Stellaris:
  * Terminology
* Borderlands:
  * Manufacturer
* Dawn of war 2:
  * Effects
* XPirateZ
  * World
* SupCom:
  * Faction
* Highfleet
  * Art style
