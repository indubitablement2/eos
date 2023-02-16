extends Object

static func start(on) -> void:
	on.start()

static func step(on) -> void:
	on.step()

static func serialize(on) -> Variant:
	return on.serialize()

static func deserialize(on, data) -> void:
	on.deserialize(data)

static func callback(arg_array, callable: Callable) -> void:
	callable.callv(arg_array)

static func callback_empty(callable: Callable) -> void:
	callable.call()
