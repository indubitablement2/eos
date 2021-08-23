# EOS (working title)


###### Table of contents

[Income](https://github.com/indubitablement2/eos-common#Income)

[User Experience](https://github.com/indubitablement2/eos-common#user-experience)

[Fleet](https://github.com/indubitablement2/eos-common#Fleet)

[World](https://github.com/indubitablement2/eos-common#World)

## Income

### Pay to play
Sell game for a low fixed price.
* Help prevent bots.
* Lower player compared to FTP. Cheaper server. Individual player are more important.

### Sell custom ship
People can buy (~$500) for the possibility to create a custom ship.

This ship will be found in-game, but much rarer than the default variant only found as quest reward, abandonned ship or in ai fleet (which they can defeat and salvage).

They chose a base ship and customize some of it's property.

* Name: My Super Custom Ship, Custom Harrier class.
* Description.
* Built-in weapon and module.
* Weapon mount.
* Skin.

(no guarantee this ship will even work)

### Free to play pirate
Can play the game for free as a pirate "faction". Need to buy the full game to get access to all ships.

### Sell fleet slot
Can only control one fleet by default. Additional slot can be bought.

### Server cost
Estimate US$360/months, US$4320/years.

* (US$170/months, US$2040/years) c6g.4xlarge, 16 arm cpus, 32gb ram, up to 10gbs network.
* (US$50/months, US$600/years) 300gb ssd with backup.
* (US$140/months, US$1680/years) 1.5tb internet per months.


## User Experience

### Core loop

Explore - Fight - Customize (gradual fleet upgrade)

### Decay and entropy
Ship need to be rebuilt and maintained, resouce need to be stocked in a ship or hangar. Hangar also cost resource to build and maintain.

This keep the world fresh. New player are not overwhelmingly behind when joining later.

This push player to utilise their wealt.

#### Resource storage
* Fleet cargo: 
  * Need ship with cargo capacity. 
  * Ends up being the most expensive. 
  * Immediately available.
* Hangar: 
  * Longer term storage. 
  * Only visible to the player who built it and anyone the player whish to share with. 
  * Drain resource inside of it to stay maintained or it will be lootable by everyone.
* Cargo pod:
  * Free.
  * Lootable by anyone
  * Will stay there indefinitely until looted by someone.
  * High value cargo attracts pirate. Low value cargo gets combined together.

### Minimal industry flexibility
When a planet is invested into (for example) ore refining, it is impractical to switch industry in response to market demand.

This can lead to surplus or scarsity increasing player activity:
* Building new needed industry. 
* Hauling resource from suplus area to scarsity area. 
* Fight over scarse resource. 
* Fight for monopoly.

### No standard currency
This make it harder to stockpile resource in conjonction with entropy.

Player can get into a loop of needing x resource while having too much of y. 
This lead to player engagement through trading.

When shopping at a colony, the cost of item is in generic value. 
The player can buy by offering at least as much value as what he is buying.

Resources value is determined by its local availability.

### Acquiring ship/weapon/module
Low tier ship and civilian ship are available everywhere.

Getting higher tier gear is hard. 
* Ranking inside a faction (need resources, only give access to this faction's ethic gear).
* Post battle salvage (needs to be repaired).
* Random wreck and cargo in space.
* Quests.

## Fleet

### Hull size
* Fighter: 
  * 1-2 light weapons.
  * The size of 1 heavy weapon.
  * No ship collision.
  * Depend on carrier to refill ammo and repair.
  * Infinitely produced by carrier for free.
  * Moves like a plane with slow turn rate and lightning speed.

* Frigate: 
  * ~4 light or 1 medium weapons.
  * The size of 4 fighters.
  * High turn rate and good speed.

* Destroyer:
  * 4 medium weapons.
  * The size of 4 frigates.

* Cruiser: 
  * 4 medium and 2 heavy weapons.
  * The size of 4 destroyers.
  * Slow turn rate.

* Capital: 
  * 4 heavy and many medium/light weapons.
  * Often has unique built-in weapon.
  * The size of 4 cruisers or more.
  * Extremely expensive to buy and maintain.
  * Very slow.

### Manufacturer
Weapon and ship have manufacturer. Some ship and weapon mount may have an afinity toward a manufacturer.

### Weapons
* Light:
* Medium:
* Heavy:

### Modules
They can be built-in.

Player installed mods gives bonus (better shield, more energy storage, faster movement, increased range, ...) at the cost of reduced energy productivity.

Damaged ship have negative built-in mods. 


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
