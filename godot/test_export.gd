extends Object

var num : int
var num2: int

func inc(n: int):
	num += n
	num2 += n

func serde() -> PackedByteArray:
	return var_to_bytes([num, num2])

func deserde(bytes: PackedByteArray):
	var v = bytes_to_var(bytes)
	num = v[0]
	num2 = v[1]

func pr():
	print("export: ", num, " non export: ", num2)

