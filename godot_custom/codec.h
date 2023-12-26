#ifndef CLIENT_CODEC_H
#define CLIENT_CODEC_H

#include "core/math/vector2.h"
#include "core/object/ref_counted.h"
#include "core/string/string_name.h"
#include "modules/websocket/websocket_peer.h"
#include "preludes.h"
#include "scene/main/node.h"

class ClientCodec : public Node {
	GDCLASS(ClientCodec, Node);

private:
	u8 *write_buffer;
	u8 *write_cursor;

	const u8 *read_cursor;

	Ref<WebSocketPeer> peer;

	bool connecting;

	void start_write();
	void put_u8(u8 value);
	void put_varint(i64 value);
	void put_string(String value);
	void finish_write();

	void decode();
	u8 get_u8();
	i64 get_varint();
	String get_string();

protected:
	static void _bind_methods();
	void _notification(int p_what);

public:
	ClientCodec() {
		write_buffer = new u8[262144];
		peer = Ref<WebSocketPeer>(WebSocketPeer::create());
		connecting = false;
	}

	~ClientCodec() {
		delete[] write_buffer;
	}

	inline static StringName connection_closed = StringName("connection_closed");

	inline static StringName _entered_battlescape = StringName("_entered_battlescape");
	GDVIRTUAL2(_entered_battlescape, i64, i64);

	void cancel_login();
	void login_username_password(String url, i64 battlescape_id, String username, String password);
	void register_username_password(String url, i64 battlescape_id, String username, String password);
};

#endif