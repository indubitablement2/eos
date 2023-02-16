extends Node2D

#CHILD

var v = false : get = getter

func method() -> Vector2:
	print("int mut")
	return self.position

func getter() -> bool:
	print("interior mutability: ", position)
	return true
