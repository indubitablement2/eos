# EOS (working title)
![Project Logo](client/godot/assets/icon/godot-ferris-64x64.png)

###### Table of contents

[User Experience](https://github.com/indubitablement2/eos#user-experience)

[Fleet](https://github.com/indubitablement2/eos#Fleet)

[World](https://github.com/indubitablement2/eos#World)

[Inspiration](https://github.com/indubitablement2/eos#Inspiration)

## User Experience

### Core loop

Explore - Fight - Customize (gradual fleet upgrade)

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

## Battlescape
Battle between fleets happens on a separate world. 

### Battle point & lag prevention
To limit lag, each team can only field a limited number of ship at a time. 
Each ship has a set bp cost based on strenght.
The bigger fleet has more bp available.

### Ship control
A player can control any of his ship and change at will. 
Otherwise he can chose to let the ai control his ship. 

### Fleet control
Player can give general direction to his fleet. 
Final action is left to the ai. 

Fleet control is deliberately limited as to not overshadow directly controlling a ship. 
This is not an RTS.

## World

### Factions
TODO

### Events
TODO

## Market

Player can see the whole market. 

Item bought stay where they are and player need to get them. 

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
