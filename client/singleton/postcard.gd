extends Object
class_name Postcard

const WRITE_SIZE := 65535
static var _write := StreamPeerBuffer.new()
static var _read := StreamPeerBuffer.new()

static func start_encode():
	if _write.get_size() != WRITE_SIZE:
		_write.resize(WRITE_SIZE)
		_write.big_endian = false
		print_debug("resized write buffer")
	_write.seek(0)

static func finish_encode() -> PackedByteArray:
	return _write.data_array.slice(0, _write.get_position())

static func put_u8(value: int):
	_write.put_8(value)

static func put_u64(value: int):
	while true:
		var byte := value & 0x7F
		value >>= 7
		value &= 0x1FFFFFFFFFFFFFF
		if value == 0:
			_write.put_8(byte)
			break
		_write.put_8(byte | 0x80)

static func put_i64(value: int):
	put_u64((value << 1) ^ (value >> 63));

static func put_bytes(value: PackedByteArray):
	put_u64(value.size())
	_write.put_data(value)

static func put_string(value: String):
	put_bytes(value.to_utf8_buffer())

static func put_f32(value: float):
	_write.put_float(value)

static func put_f64(value: float):
	_write.put_double(value)

static func put_vector2(value: Vector2):
	_write.put_float(value.x)
	_write.put_float(value.y)


static func start_decode(packet: PackedByteArray):
	_read.data_array = packet
	_read.big_endian = false

static func get_u8() -> int:
	return _read.get_u8()

static func get_u64() -> int:
	var value := 0
	var shift := 0
	while true: 
		var byte := _read.get_u8()
		value |= (byte & 0x7F) << shift;
		if (byte & 0x80) == 0:
			break
		shift += 7
	return value

static func get_i64() -> int:
	var value := get_u64()
	return ((value >> 1) & 0x7FFFFFFFFFFFFFFF) ^ -(value & 1)

static func get_bytes() -> PackedByteArray:
	return _read.get_data(get_u64())[1]

static func get_string() -> String:
	return get_bytes().get_string_from_utf8()

static func get_f32() -> float:
	return _read.get_float()

static func get_f64() -> float:
	return _read.get_double()

static func get_vector2() -> Vector2:
	return Vector2(_read.get_float(), _read.get_float())

static func _test():
	start_encode()
	put_u8(123)
	put_u64(65000)
	put_u64(-1)
	put_u64(1)
	put_i64(-1)
	put_i64(1)
	put_string("Hello World!")
	put_vector2(Vector2(123, 123))
	
	var encoded := finish_encode()
	assert(encoded == PackedByteArray([123, 232, 251, 3, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 1, 1, 2, 12, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 33, 0, 0, 246, 66, 0, 0, 246, 66]))
	start_decode(encoded)
	
	assert(get_u8() == 123)
	assert(get_u64() == 65000)
	assert(get_u64() == -1)
	assert(get_u64() == 1)
	assert(get_i64() == -1)
	assert(get_i64() == 1)
	assert(get_string() == "Hello World!")
	assert(get_vector2() == Vector2(123, 123))
