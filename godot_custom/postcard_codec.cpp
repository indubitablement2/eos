#include "postcard_codec.h"
#include "core/error/error_list.h"
#include "core/error/error_macros.h"
#include "core/math/vector2.h"
#include "core/string/ustring.h"
#include "core/variant/variant.h"
#include "preludes.h"
#include <cstring>

void PostcardCodec::_bind_methods() {
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("put_u8", "value"), &PostcardCodec::put_u8);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("put_u64", "value"), &PostcardCodec::put_u64);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("put_i64", "value"), &PostcardCodec::put_i64);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("put_bytes", "value"), &PostcardCodec::_put_bytes);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("put_string", "value"), &PostcardCodec::put_string);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("put_f32", "value"), &PostcardCodec::put_f32);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("put_f64", "value"), &PostcardCodec::put_f64);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("put_vector2", "value"), &PostcardCodec::put_vector2);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("finish_encode"), &PostcardCodec::_finish_encode);

	ClassDB::bind_static_method("PostcardCodec", D_METHOD("start_decode", "packet"), &PostcardCodec::_start_decode);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("get_u8"), &PostcardCodec::get_u8);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("get_u64"), &PostcardCodec::get_u64);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("get_i64"), &PostcardCodec::get_i64);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("get_bytes", "max_len"), &PostcardCodec::_get_bytes);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("get_string", "max_len"), &PostcardCodec::get_string);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("get_f32"), &PostcardCodec::get_f32);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("get_f64"), &PostcardCodec::get_f64);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("get_vector2"), &PostcardCodec::get_vector2);
	ClassDB::bind_static_method("PostcardCodec", D_METHOD("finish_decode"), &PostcardCodec::finish_decode);
}

bool PostcardCodec::decode_would_oob(u64 n) {
	return read_cursor + n > read_end;
}

void PostcardCodec::put_u8(u8 value) {
	*write_cursor = value;
	write_cursor += 1;
}

void PostcardCodec::put_u64(u64 value) {
	while (true) {
		u8 byte = u8(value & 0x7F);
		value >>= 7;
		if (value == 0) {
			put_u8(byte);
			break;
		}
		put_u8(byte | 0x80);
	}
}

void PostcardCodec::put_i64(i64 value) {
	put_u64((value << 1) ^ (value >> 63));
}

void PostcardCodec::put_bytes(const u8 *data, u64 len) {
	put_u64(len);
	memcpy(write_cursor, data, len);
	write_cursor += len;
}

void PostcardCodec::_put_bytes(PackedByteArray value) {
	put_bytes(value.ptr(), value.size());
}

void PostcardCodec::put_string(String value) {
	auto utf8 = value.utf8();
	put_bytes((const u8 *)utf8.get_data(), utf8.size());
}

void PostcardCodec::put_f32(f32 value) {
	memcpy(write_cursor, &value, 4);
	write_cursor += 4;
}

void PostcardCodec::put_f64(f64 value) {
	memcpy(write_cursor, &value, 8);
	write_cursor += 8;
}

void PostcardCodec::put_vector2(Vector2 value) {
	put_f32(value.x);
	put_f32(value.y);
}

PostcardCodec::ByteSlice PostcardCodec::finish_encode() {
	ByteSlice ret{
		write_buffer,
		u64(write_cursor - write_buffer)
	};
	write_cursor = write_buffer;
	return ret;
}

PackedByteArray PostcardCodec::_finish_encode() {
	ByteSlice slice = finish_encode();
	PackedByteArray arr = PackedByteArray();
	arr.resize(slice.len);
	memcpy(arr.ptrw(), slice.data, slice.len);
	return arr;
}

void PostcardCodec::start_decode(const u8 *packet, u64 len) {
	read_cursor = packet;
	read_end = packet + len;
}

void PostcardCodec::_start_decode(PackedByteArray packet) {
	read_array = packet;
	start_decode(read_array.ptr(), read_array.size());
}

u8 PostcardCodec::get_u8() {
	u8 value;
	if (decode_would_oob(1)) {
		value = 0;
		read_cursor = read_end + 1;
	} else {
		value = *read_cursor;
	}
	read_cursor += 1;
	return value;
}

u64 PostcardCodec::get_u64() {
	u64 value = 0;
	i32 shift = 0;
	while (true) {
		auto byte = get_u8();
		value |= (byte & 0x7F) << shift;
		if ((byte & 0x80) == 0) {
			break;
		}
		shift += 7;
	}
	return value;
}

i64 PostcardCodec::get_i64() {
	u64 value = get_u64();
	return i64(value >> 1) ^ -(i64(value & 1));
}

PostcardCodec::ByteSlice PostcardCodec::get_bytes(u64 max_len) {
	u64 len = get_u64();
	if (len > max_len || decode_would_oob(len)) {
		read_cursor = read_end + 1;

		return ByteSlice{
			nullptr,
			0
		};
	}

	ByteSlice value{
		.data = read_cursor,
		.len = len
	};
	read_cursor += len;
	return value;
}

PackedByteArray PostcardCodec::_get_bytes(u64 max_len) {
	ByteSlice bytes = get_bytes(max_len);
	PackedByteArray arr = PackedByteArray();
	arr.resize(bytes.len);
	memcpy(arr.ptrw(), bytes.data, bytes.len);
	return arr;
}

String PostcardCodec::get_string(u64 max_len) {
	ByteSlice bytes = get_bytes(max_len);

	return String::utf8((const char *)bytes.data, bytes.len);
}

f32 PostcardCodec::get_f32() {
	if (decode_would_oob(4)) {
		read_cursor = read_end + 1;
		return 0.0;
	}

	f32 value;
	memcpy(&value, read_cursor, 4);
	read_cursor += 4;
	return value;
}

f64 PostcardCodec::get_f64() {
	if (decode_would_oob(8)) {
		read_cursor = read_end + 1;
		return 0.0;
	}

	f64 value;
	memcpy(&value, read_cursor, 8);
	read_cursor += 8;
	return value;
}

Vector2 PostcardCodec::get_vector2() {
	return Vector2(get_f32(), get_f32());
}

Error PostcardCodec::finish_decode() {
	Error err = Error::OK;

	if (read_cursor != read_end) {
		err = Error::ERR_INVALID_DATA;
	}

	read_array = {};
	read_cursor = nullptr;
	read_end = nullptr;

	return err;
}
