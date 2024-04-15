#ifndef CLIENT_CODEC_H
#define CLIENT_CODEC_H

#include "core/math/vector2.h"
#include "core/object/ref_counted.h"
#include "core/string/string_name.h"
#include "core/variant/array.h"
#include "core/variant/dictionary.h"
#include "core/variant/typed_array.h"
#include "modules/websocket/websocket_peer.h"
#include "preludes.h"
#include "scene/main/node.h"
#include <unordered_map>

class ClientCodec : public Node {
	GDCLASS(ClientCodec, Node);

private:
	Ref<WebSocketPeer> peer;
	/// Network id to entity id
	std::unordered_map<u32, i64> entity_ids;

	u8 *write_buffer;
	u8 *write_cursor;

	const u8 *read_cursor;

	bool connecting;

	void start_write();
	void put_u8(u8 value);
	void put_varint(i64 value);
	void put_string(String value);
	void put_f32(f32 value);
	void finish_write();

	void decode();
	u8 get_u8();
	u16 get_u16();
	i64 get_varint();
	String get_string();
	f32 get_f32();
	f64 get_f64();
	f32 get_u16_packed_f32(f32 min, f32 max);
	Vector2 get_vector2();

protected:
	static void _bind_methods();
	void _notification(int p_what);

public:
	ClientCodec();
	~ClientCodec();

	StringName entity_state_entity_id;
	StringName entity_state_translation;
	StringName entity_state_rotation;

	StringName connection_closed;

	StringName _entered_simulation;
	GDVIRTUAL2(_entered_simulation, i64, i64);
	StringName _state;
	GDVIRTUAL2(_state, f64, TypedArray<Dictionary>);
	StringName _add_entity;
	GDVIRTUAL2(_add_entity, i64, i64);
	StringName _remove_entity;
	GDVIRTUAL1(_remove_entity, i64);
	StringName _remove_seen_entity;
	GDVIRTUAL1(_remove_seen_entity, i64);
	StringName _add_seen_entity;
	GDVIRTUAL1(_add_seen_entity, i64);

	void cancel_login();
	void login_username_password(String url, i64 simulation_id, String username, String password);
	void register_username_password(String url, i64 simulation_id, String username, String password);

	void create_first_ship();
};

#endif