#include "codec.h"
#include "core/error/error_macros.h"
#include "core/object/class_db.h"

void ClientCodec::_bind_methods() {
	ClassDB::bind_method(D_METHOD("c123ancel_login"), &ClientCodec::cancel_login);
	ClassDB::bind_method(D_METHOD("login_username_password", "url", "battlescape_id", "username", "password"), &ClientCodec::login_username_password);
	ClassDB::bind_method(D_METHOD("register_username_password", "url", "battlescape_id", "username", "password"), &ClientCodec::register_username_password);

	GDVIRTUAL_BIND(_entered_battlescape, "client_id", "battlescape_id");

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
			GDVIRTUAL_CALL(_entered_battlescape, get_varint(), get_varint());
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

void ClientCodec::login_username_password(String url, i64 battlescape_id, String username, String password) {
	peer = Ref<WebSocketPeer>(WebSocketPeer::create());
	peer->connect_to_url(url);
	connecting = true;

	start_write();

	put_varint(battlescape_id);
	put_varint(0);
	put_string(username);
	put_string(password);
}

void ClientCodec::register_username_password(String url, i64 battlescape_id, String username, String password) {
	peer = Ref<WebSocketPeer>(WebSocketPeer::create());
	peer->connect_to_url(url);
	connecting = true;

	start_write();

	put_varint(battlescape_id);
	put_varint(1);
	put_string(username);
	put_string(password);
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

void ClientCodec::finish_write() {
	peer->put_packet(write_buffer, write_cursor - write_buffer);
}

u8 ClientCodec::get_u8() {
	u8 value = *read_cursor;
	read_cursor += 1;
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