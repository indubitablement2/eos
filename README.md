# EOS (working title)
![Project Logo](client/godot/assets/icon/godot-ferris-64x64.png)

###### Table of contents

[Finance](https://github.com/indubitablement2/eos#Finance)

[User Experience](https://github.com/indubitablement2/eos#user-experience)

[Fleet](https://github.com/indubitablement2/eos#Fleet)

[World](https://github.com/indubitablement2/eos#World)

[Market](https://github.com/indubitablement2/eos#Market)

[Inspiration](https://github.com/indubitablement2/eos#Inspiration)

## Finance 
### Server cost
Estimate US$360/months, US$4320/years.

* (US$170/months, US$2040/years) c6g.4xlarge, 16 arm cpus, 32gb ram, up to 10gbs network.
* (US$50/months, US$600/years) 300gb ssd with backup.
* (US$140/months, US$1680/years) 1.5tb internet per months.

### Income
* Base cost $20
* Ship/weapon skin trade cut 10%
  * Ship skin are gained by playing.
  * Community can create their skin.
* Character portrait trade cut 10%
  * Character portrait are gained by playing. Rarer ones can be more noticeable.

## User Experience

### Core loop

Explore - Fight - Customize (gradual fleet upgrade)

### Decay & entropy
Ship need to be rebuilt and maintained.
Storage cost resource. 

This keep the world fresh. New player are not overwhelmingly behind when joining later.

This push player to utilise their wealt.

#### Resource storage
* Fleet cargo: 
  * Need ship with cargo capacity. 
  * Ends up being the most expensive. 
  * Immediately available.
* Hangar: 
  * Cost a fee. 
  * If host station is destroyed, hangar is lootable. 
  * Items are only available at the station they were stashed in by default. Transport is not safe and cost a fee. 
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

### Currency
Standard currency is an item (credit chip). 
This make it harder to stockpile wealth in conjonction with entropy.

### Acquiring ship/weapon/module
Low tier ship and civilian ship are available everywhere.

Getting higher tier gear is harder:
* Ranking inside a faction (still need to buy, only give access to this faction's gear).
* Post battle salvage (needs to be repaired).
* Random wreck and cargo in space.
* Quests.

## Fleet & ships

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
  * High turn rate and speed.

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
  * Extremely expensive to build and maintain.
  * Very slow.

* Experimental
  * 8 heavy and many medium weapons.
  * Has fortress like capability, but is mobile. 
  * 1.5+ capital
  * Ludicrously expensive to maintain.
  * Can not be build (no blueprint hence the name). 

### Manufacturer
Weapon and ship have manufacturer.

Some ship and weapon mount may have an afinity toward a manufacturer. 
Module and weapon slots generaly only accept a pacticular manufacturer.

### Weapons size
* Light:
* Medium:
* Heavy:

### Modules
They can be built-in.

Ships have a limited number of free module slot. Higher quality ship have more.

Damaged ship have negative built-in mods that are expensive to remove.

## World

### Manufacturers archetype
* UEF: Brute force. More frontal firepower and defence, but slower. Vulnerable when flanked or isolated. 
  * Theme: bulky, ww1
  * Specialist: Armor, balistic
* Cybran: Unconventional tactic (stealth, suicide ship), prioritise offence. Worse quality, cheaper.
  * Theme: spiky, punk,
  * Specialist: Speed, missile
* Aeon: Ship specialize in one thing. Better at their primary task and worst at everything else. Slightly better quality and more expensive. 
  * Theme: round
  * Specialist: Range, fighter
* Seraphim: High quality multi-purpose ship. One ship is equivalent to 1.5 ships of the same size from other manufacturers.
  * Theme: round asymmetric
  * Specialist: Shield, energy
* Pirate: Cheap, very poor defence and mediocre offence. 
  * Theme: Repurposed civilian ship
  * Specialist: None
* Civilian: Cheap utility ship. Poor combat ship. 
  * Theme: Dull, mining equipment 
  * Specialist: Utility

### Joining vs creating faction
Joining faction:
* Pros:
  * Free insurance
  * Fleet protection in controlled systems
  * Access to faction market
  * Vassal colony are supported by parent faction
* Cons:
  * Vassal colony pay taxes to parent faction
* Neutrals:
  * Inherit faction relation

Creating faction:
* Pros:
  * Potentially high passive income from vassal taxe, market taxe and colony production
* Cons:
  * Needs to provide costly incentives to vassals (insurance, lower market price)
  * Needs to sustain a military

### Reputation & relation
Relation determine a fleet-faction or faction-faction standing. 

If a fleet-faction reputation is not present, it use the faction's default. Pacifist may be higher. Xenophobe may be lower. 

Fleet in faction inherit their faction's relation with other factions. 

Fleet have reputation which act as a multiplier(0..1.0) to relation with factions and a relation with other factionless fleet (rep_self.min(rep_other). 

Some faction (pirate) do not care about reputation much.

### Faction archetype 
Pirate
xeno
pacifist
wide
tall
capitalist 

## Market

todo

## Technical
* smoke
  * particles
* light & shadow
  * godot cpu lightmapper
* particles collision
  * custom particle shader
* trail metascape
  * godot lide2d
* ui
  * godot control & signal

## Inspiration

* Starsector:
  * Battle
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
