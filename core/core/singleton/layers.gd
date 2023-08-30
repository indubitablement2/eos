extends Object
class_name Layers

const TEAM_OFFSET := 7

const HULL_SHIP := 1
const HULL_MISSILE := 1 << 1
const HULL_FIGHTER := 1 << 2
const PROJECTILE := 1 << 3
const DETECTOR_LARGE := 1 << 4
const DETECTOR_SMALL := 1 << 5
const UNUSED_0 := 1 << 6

const ALL_TEAM := 0b0111_1111

const ALL_HULL_TEAM := HULL_SHIP | HULL_MISSILE | HULL_FIGHTER
const ALL_HULL := (
		ALL_HULL_TEAM
	| ALL_HULL_TEAM << TEAM_OFFSET
	| ALL_HULL_TEAM << TEAM_OFFSET * 2
	| ALL_HULL_TEAM << TEAM_OFFSET * 3)


const ALL_TEAMED := (
	ALL_TEAM
	| ALL_TEAM << TEAM_OFFSET
	| ALL_TEAM << TEAM_OFFSET * 2
	| ALL_TEAM << TEAM_OFFSET * 3)


const DEBRIS := 1 << 28
const UNUSED_1 := 1 << 29
const UNUSED_2 := 1 << 30
const UNUSED_3 := 1 << 31

const ALL_NOT_TEAMED := DEBRIS | UNUSED_1 | UNUSED_2 | UNUSED_3




