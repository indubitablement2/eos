# EOS (working title)
![Project Logo](client/godot/assets/icon/godot-ferris-64x64.png)

###### Table of contents

[User Experience](https://github.com/indubitablement2/eos#user-experience)

[Fleet](https://github.com/indubitablement2/eos#Fleet)

[World](https://github.com/indubitablement2/eos#World)

[Market](https://github.com/indubitablement2/eos#Market)

[Inspiration](https://github.com/indubitablement2/eos#Inspiration)

### Dependencies

client-rust -> battlescape, data, net
server -> metascape, battlescape, data, net

battlescape -> data
metascape -> data, net, acc

data
net
acc

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

### Acquiring ship/weapon/module
Low tier ship and civilian ship are available everywhere.

Getting higher tier gear is harder:
* Ranking inside a faction (still need to buy, only give access to this faction's gear).
* Post battle salvage (needs to be repaired).
* Random wreck and cargo in space.
* Quests.

### Disconnected fleet
When a played disconnect, his fleet does not disappear.
It does not consume resources however. 

It can be assigned to do a number of things:
* idle near allied planet 
* idle near last location and flee from enemy 
* stay still

## Gear & ships

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

### Design 
Weapon and ship have designer.

Some ship and weapon mount may have an afinity for a design. 
Module and weapon mount may only accept a pacticular design.

Design archetype
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

### Weapons size
* Light:
* Medium:
* Heavy:

### Modules
They can be built-in.

Ships have a limited number of free module slot. 
Higher quality ship have more.

Damaged ship have negative built-in mods that are expensive to remove.

### Captain
Captain can be hired and assigned to a ship to provide bonuses. 

They cost a salary based on their rank. 

They gain experience by taking part in fight. 

## Battlescape 
Battle between fleets happens on a separate world. 

### Battle point & lag prevention
To limit lag, each team can only field a limited number of ship at a time. 
Each ship has a set bp cost based on strenght.
The bigger fleet has more bp available. 

A battle can only happen between enemies. 
A neutral can not interfere. 
Multiple player can join the same side if they are allied. 

### Ship control
A player can control any of his ship and change at will. 
Otherwise he can chose to let the ai control his ship. 

### Fleet control
Player can give general direction to his fleet. 
Final action is left to the ai. 

Fleet control is deliberately limited as to not overshadow directly controlling a ship. 

## World

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

### Faction research path

Player can hire (expensive) researcher to upgrade their faction. 
This is a currency sink. 
Upgrade are lost when a faction is dismantled. 

* Piratry P
  * Allow pirating fleet
  * Default reputation is 0
  * Enable futher research in pirating 
* Minting P
  * Allow building mint
* Pirate ship 1..5 P
  * Allow building some very cheap pirate ship
* Trade 1..5
  * Can start trade with other ally faction
  * Better trade price
* Insurance

### Reputation
* faction-faction
  * a.min(b).reputation(a.max(b))
  * a.default_reputation.min(b.default_reputation)
* faction-neutral
  * a.reputation(b)
  * a.default_reputation.min(b.reputation)
* neutral-neutral
  * a.reputation.min(b.reputation)

Reputation 0..100 default 70
* 80+ ally
  * help encouraged
* 60+ neutral
  * war discouraged
* 40+ hated
  * war allowed
* 0+ enemy
  * war encouraged
  * no trade

There is a delay before a faction reputation change take effect. 
Going up require both faction to agree. 

Reputation trend toward neutral. 
* It goes down by 
  * initiating a fight with a neutral or ally 
  * joining a pirate faction
* It goes up by
  * initiating a fight with an enemy
  * defending an ally

## Market

Player can see the whole market. 

Item bought stay where they are and player need to get them. 

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
