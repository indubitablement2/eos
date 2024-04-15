#include "codec.h"
#include "core/error/error_macros.h"
#include "core/math/vector2.h"
#include "core/object/class_db.h"
#include "core/object/object.h"
#include "preludes.h"

ClientCodec::ClientCodec() {
	write_buffer = new u8[262144];
	peer = Ref<WebSocketPeer>(WebSocketPeer::create());
	connecting = false;

	entity_state_entity_id = StringName("entity_id");
	entity_state_translation = StringName("translation");
	entity_state_rotation = StringName("rotation");

	connection_closed = StringName("connection_closed");

	_entered_simulation = StringName("_entered_simulation");
	_state = StringName("_state");
	_add_entity = StringName("_add_entity");
	_remove_entity = StringName("_remove_entity");
	_remove_seen_entity = StringName("_remove_seen_entity");
	_add_seen_entity = StringName("_add_seen_entity");
}

ClientCodec::~ClientCodec() {
	delete[] write_buffer;
}

void ClientCodec::_bind_methods() {
	ClassDB::bind_method(D_METHOD("cancel_login"), &ClientCodec::cancel_login);
	ClassDB::bind_method(D_METHOD("login_username_password", "url", "simulation_id", "username", "password"), &ClientCodec::login_username_password);
	ClassDB::bind_method(D_METHOD("register_username_password", "url", "simulation_id", "username", "password"), &ClientCodec::register_username_password);

	GDVIRTUAL_BIND(_entered_simulation, "client_id", "simulation_id");
	GDVIRTUAL_BIND(_state, "time", "entity_states");
	GDVIRTUAL_BIND(_add_entity, "entity_id", "entity_data_id");
	GDVIRTUAL_BIND(_remove_entity, "entity_id");
	GDVIRTUAL_BIND(_remove_seen_entity, "entity_id");
	GDVIRTUAL_BIND(_add_seen_entity, "entity_id");

	ADD_SIGNAL(MethodInfo("connection_closed", PropertyInfo(Variant::STRING, "message")));
}

void ClientCodec::_notification(int p_what) {
	if (p_what != NOTIFICATION_PROCESS || Engine::get_singleton()->is_editor_hint()) {
		return;
	}

	// TODO: Add test to see if all methods are overridden?
	// GDVIRTUAL_IS_OVERRIDDEN(_process);

	peer->poll();

	switch (peer->get_ready_state()) {
		case WebSocketPeer::State::STATE_CLOSED: {
			if (!connecting) {
				connecting = true;
				emit_signal(connection_closed, peer->get_close_reason());
			}
		}
		case WebSocketPeer::State::STATE_CLOSING: {
			break;
		}
		case WebSocketPeer::State::STATE_CONNECTING: {
			break;
		}
		case WebSocketPeer::State::STATE_OPEN: {
			if (connecting) {
				connecting = false;
				finish_write();
			}

			break;
		}
	}

	while (peer->get_available_packet_count() > 0) {
		i32 len;
		Error err = peer->get_packet(&read_cursor, len);
		ERR_CONTINUE_MSG(err != OK, "Error reading packet: " + itos(err));
		decode();
	}
}

void ClientCodec::decode() {
	switch (get_varint()) {
		case 0: {
			i64 client_id = get_varint();
			i64 simulation_id = get_varint();

			entity_ids = std::unordered_map<u32, i64>();

			GDVIRTUAL_CALL(_entered_simulation, client_id, simulation_id);
			break;
		}
		case 1: {
			f64 time = get_f64();
			Vector2 origin = get_vector2();
			i64 state_len = get_varint();

			TypedArray<Dictionary> entity_states = TypedArray<Dictionary>();
			entity_states.resize(state_len);
			for (i64 i = 0; i < state_len; i++) {
				u32 network_id = get_varint();
				Vector2 relative_translation = get_vector2();
				f32 rotation = get_u16_packed_f32(-f32_PI, f32_PI);

				Dictionary state = Dictionary();

				state[entity_state_entity_id] = entity_ids[network_id];
				state[entity_state_translation] = origin + relative_translation;
				state[entity_state_rotation] = rotation;

				entity_states[i] = state;
			}

			GDVIRTUAL_CALL(_state, time, entity_states);
			break;
		}
		case 2: {
			i64 entity_id = get_varint();
			u32 network_id = get_varint();
			i64 entity_data_id = get_varint();

			entity_ids[network_id] = entity_id;

			GDVIRTUAL_CALL(_add_entity, entity_id, entity_data_id);
			break;
		}
		case 3: {
			u32 network_id = get_varint();

			i64 entity_id = entity_ids[network_id];
			entity_ids.erase(network_id);

			GDVIRTUAL_CALL(_remove_entity, entity_id);
			break;
		}
		case 4: {
			u32 network_id = get_varint();

			i64 entity_id = entity_ids[network_id];

			GDVIRTUAL_CALL(_remove_seen_entity, entity_id);
			break;
		}
		case 5: {
			u32 network_id = get_varint();

			i64 entity_id = entity_ids[network_id];

			GDVIRTUAL_CALL(_add_seen_entity, entity_id);
			break;
		}
	}
}

void ClientCodec::cancel_login() {
	peer = Ref<WebSocketPeer>(WebSocketPeer::create());

	if (connecting) {
		connecting = false;
		emit_signal(connection_closed, "Login cancelled");
	}
}

void ClientCodec::login_username_password(String url, i64 simulation_id, String username, String password) {
	peer = Ref<WebSocketPeer>(WebSocketPeer::create());
	peer->connect_to_url(url);
	connecting = true;

	start_write();

	put_varint(simulation_id);
	put_varint(0);
	put_string(username);
	put_string(password);
}

void ClientCodec::register_username_password(String url, i64 simulation_id, String username, String password) {
	peer = Ref<WebSocketPeer>(WebSocketPeer::create());
	peer->connect_to_url(url);
	connecting = true;

	start_write();

	put_varint(simulation_id);
	put_varint(1);
	put_string(username);
	put_string(password);
}

void ClientCodec::create_first_ship() {
	start_write();

	put_varint(1);

	finish_write();
}

void ClientCodec::start_write() {
	write_cursor = write_buffer;
}

void ClientCodec::put_u8(u8 value) {
	*write_cursor = value;
	write_cursor += 1;
}

void ClientCodec::put_varint(i64 value) {
	while (true) {
		i64 byte = value & 0x7F;
		value >>= 7;
		if (value == 0) {
			put_u8(byte);
			break;
		}
		put_u8(byte | 0x80);
	}
}

void ClientCodec::put_string(String value) {
	i64 len = value.length();
	put_varint(len);
	memcpy(write_cursor, value.utf8().get_data(), len);
	write_cursor += len;
}

void ClientCodec::put_f32(f32 value) {
	memcpy(write_cursor, &value, 4);
	write_cursor += 4;
}

void ClientCodec::finish_write() {
	peer->put_packet(write_buffer, write_cursor - write_buffer);
}

u8 ClientCodec::get_u8() {
	u8 value = *read_cursor;
	read_cursor += 1;
	return value;
}

u16 ClientCodec::get_u16() {
	u16 value;
	memcpy(&value, read_cursor, 2);
	read_cursor += 2;
	return value;
}

i64 ClientCodec::get_varint() {
	i64 value = 0;
	i32 shift = 0;
	while (true) {
		i64 byte = get_u8();
		value |= (byte & 0x7F) << shift;
		if ((byte & 0x80) == 0) {
			break;
		}
		shift += 7;
	}
	return value;
}

String ClientCodec::get_string() {
	i64 len = get_varint();
	String value = String::utf8((const char *)read_cursor, len);
	read_cursor += len;
	return value;
}

f32 ClientCodec::get_f32() {
	f32 value;
	memcpy(&value, read_cursor, 4);
	read_cursor += 4;
	return value;
}

f64 ClientCodec::get_f64() {
	f64 value;
	memcpy(&value, read_cursor, 8);
	read_cursor += 8;
	return value;
}

f32 ClientCodec::get_u16_packed_f32(f32 min, f32 max) {
	return f32(get_u16()) / f32(MAX_U16) * (max - min) + min;
}

Vector2 ClientCodec::get_vector2() {
	return Vector2(get_f32(), get_f32());
}