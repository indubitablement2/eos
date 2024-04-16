#ifndef POSTCARD_CODEC_H
#define POSTCARD_CODEC_H

#include "core/math/vector2.h"
#include "core/object/class_db.h"
#include "core/object/object.h"
#include "core/string/ustring.h"
#include "core/variant/variant.h"
#include "preludes.h"

class PostcardCodec : Object {
	GDCLASS(PostcardCodec, Object);

protected:
	static void _bind_methods();

private:
	static inline u8 write_buffer[65535] = {};
	static inline u8 *write_cursor = &write_buffer[0];

	static inline PackedByteArray read_array = {};
	static inline const u8 *read_cursor = nullptr;
	// One past the last byte of the read buffer.
	static inline const u8 *read_end = nullptr;

	static bool decode_would_oob(u64 n);

public:
	struct ByteSlice {
		const u8 *data;
		u64 len;
	};

	static void put_u8(u8 value);
	static void put_u64(u64 value);
	static void put_i64(i64 value);
	static void put_bytes(const u8 *data, u64 len);
	static void _put_bytes(PackedByteArray value);
	static void put_string(String value);
	static void put_f32(f32 value);
	static void put_f64(f64 value);
	static void put_vector2(Vector2 value);
	static ByteSlice finish_encode();
	static PackedByteArray _finish_encode();

	static void start_decode(const u8 *packet, u64 len);
	static void _start_decode(PackedByteArray packet);
	static u8 get_u8();
	static u64 get_u64();
	static i64 get_i64();
	static ByteSlice get_bytes(u64 max_len);
	static PackedByteArray _get_bytes(u64 max_len);
	static String get_string(u64 max_len);
	static f32 get_f32();
	static f64 get_f64();
	static Vector2 get_vector2();
	static Error finish_decode();
};

#endif