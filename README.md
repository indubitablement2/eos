Status: **On hold.** Will come back to it eventually.

---

# EOS (working title)
![Project Logo](logo.jpg)

## Editor Build (C++)
See https://docs.godotengine.org/en/stable/contributing/development/compiling/index.html for requirements.
Commands to easily build and launch the editor as well as update compile_commands.json (for use with clangd) are in: `godot_custom/.vscode/tasks.json`. These commands are meant to run from `godot_custom/` folder.

## Server Build (Rust)
Require cargo and rust (only tested on nightly).
Run cargo from `server/` folder. By default database and instance are merged into a single executable which is mostly only useful for testing. Build for either by adding `--feature database` or `--feature instance`. See `launch.sh` which handle lauching servers.

## Design Document

TODO: 
* fleet capacity (limit system capacity by team)
* monetization (never p2w, can only buy extra maximum fleet capacity + maybe replay recording (both cost extra server resources))
* Fleet control is deliberately limited as to not overshadow directly controlling a ship.
* market
* currency mint and other piracy (pirate themed factions should have access to cheap/bad pirate ship)
* factions (all player controlled or allow npc?)
* events (similar to stellaris crisis, which the players can ally to fight back. perhaps do this organically by allowing a few players to find a VERY strong item which makes their faction over powered, but how to make them a global threat other players will want to ally against?)
* league: new system not connected to main systems which player can join, but have to start from nothing. Good for new players to start on an even playing field. Require that nothing can move between non-connected systems.
* Player have a bad (slow, no cargo) invincible ship by default, no free cam. can teleport to any system
* Use drones to move items between ships/station
* blueprint and ship manufacturing

##### Table of contents

[User Experience](https://github.com/indubitablement2/eos#user-experience)

[Items](https://github.com/indubitablement2/eos#Items)

[World](https://github.com/indubitablement2/eos#World)

## User Experience

### Core loop

Explore - Fight - Improve fleet

### Decay & entropy
Nothings stay forever without player input.
Ships and stations need to be maintained and can always be destroyed.
Storage cost resource.

Reasons:
Keep the world fresh. New player are not overwhelmingly behind when joining later.
Push player to utilise their wealt instead of hoarding.

#### Storage
* Ship cargo:
  * Need ship with cargo capacity.
  * Ends up being the most expensive.
* Station cargo:
  * Cost resource to be built and maintained on a station. Owner may rent hangar for a fee to other player.
  * Most secure. Host station need to be destroyed before hangar is lootable by other player.
  * Items are only available at the station they were stashed in. Transport is not safe and cost a fee.
* Cargo pod:
  * Free. Just drop items out of a ship's inventory.
  * Lootable by anyone.
  * Will stay there indefinitely or until looted by someone.

### Minimal industry flexibility
When a planet is investing into an industry (eg. by building an ore refinery), it is impractical to switch industry in response to market demand.

This lead to surplus or scarsity increasing player activity:
* Building new needed industry. 
* Hauling resource from suplus area to scarsity area. 
* Fight over scarse resource. 
* Fight for monopoly.

### Acquiring ship/weapon/module
Low tier and civilian ship should be readily available anywhere.

Getting higher tier item is harder:
* Ranking inside a faction (still need to buy, only give access to this faction's items).
* Post battle salvage (needs to be repaired).
* Random wreck and cargo in space.
* Missions.

### Ship ai autonomy
Can tell owned ships to perform broad tast similar to colony sim dwarf fortress. 
This is only for more monotonous task: Salvage and mine in selected system and defend trade route or station.
Ai handle the fine details eg. which path to take, which asteroid or salvage to mine, flee or fight when attacked. 
However, a player actively managing his ships should be more efficient. 

### Disconnected client
When a played disconnect, his fleets does not disappear. Instead, ai takes over and automatically perform assigned tasks.

## Items

### Ship size
* Fighter & drone: 
  * 1-2 light weapons.
  * The size of 1 heavy weapon.
  * No ship collision.
  * Depend on carrier to refill ammo and repair.
  * Infinitely produced by carrier for free.
  * Fighter have poor turn rate and high speed similar to plane.
  * Can not be player controlled.

* Frigate: 
  * 1 mediums or equivalent turrets.
  * The size of 4 fighters.
  * High turn rate and speed.

* Destroyer:
  * 4 mediums or equivalent turrets.
  * The size of 2 frigates.

* Cruiser: 
  * 3 heavy or equivalent weapons.
  * The size of 2 destroyers.
  * Slow turn rate.

* Experimental
  * 6 heavy or equivalent weapons.
  * Often has unique built-in weapon.
  * Has fortress like capability. 
  * The size of 2+ cruisers.
  * Very slow.
  * Ludicrously expensive to maintain.
  * Can not be build (no blueprint hence the name). 

### Design (lore) 
Weapon and ship have designer.

Design archetype (names taken from supreme commander)
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

### Ai
Ai can be installed on a ship to provide powerful bonuses. They can be crafted similar to end-game crafting common in arpg.

Player does not lose them when its ship is destroyed, providing a source of permanent power for the player.

They can not be traded between players.
