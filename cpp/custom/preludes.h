#ifndef PRELUDES_HPP
#define PRELUDES_HPP

#include "core/error/error_macros.h"
#include "editor/connections_dialog.h"
#include "godot/core/typedefs.h"
#include <memory>
#include <unordered_map>
#include <vector>

using u8 = uint8_t;
using u16 = uint16_t;
using u32 = uint32_t;
using u64 = uint64_t;

using i8 = int8_t;
using i16 = int16_t;
using i32 = int32_t;
using i64 = int64_t;

using f32 = float;
using f64 = double;

const u8 MAX_U8 = UINT8_MAX;
const u16 MAX_U16 = UINT16_MAX;
const u32 MAX_U32 = UINT32_MAX;
const u64 MAX_U64 = UINT64_MAX;

const i8 MIN_I8 = INT8_MIN;
const i8 MAX_I8 = INT8_MAX;
const i16 MIN_I16 = INT16_MIN;
const i16 MAX_I16 = INT16_MAX;
const i32 MIN_I32 = INT32_MIN;
const i32 MAX_I32 = INT32_MAX;
const i64 MIN_I64 = INT64_MIN;
const i64 MAX_I64 = INT64_MAX;

const f32 TAU = 6.28318530718f;
const f32 PI = 3.14159265359f;
const f32 HALF_PI = 1.57079632679f;

#define ADD_SETGET_NO_INIT(type, name) \
	type name;                         \
	void set_##name(type value);       \
	type get_##name() const;

#define ADD_SETGET(type, name, init) \
	type name = init;                \
	void set_##name(type value);     \
	type get_##name() const;

#define ADD_SETGET_IMPL(class, type, name)                \
	void class ::set_##name(type value) { name = value; } \
	type class ::get_##name() const { return name; }

#define ADD_SETGET_PROPERTY(class, variant, name)                               \
	ClassDB::bind_method(D_METHOD("set_" #name, "value"), &class ::set_##name); \
	ClassDB::bind_method(D_METHOD("get_" #name), &class ::get_##name);          \
	ADD_PROPERTY(PropertyInfo(variant, #name), "set_" #name, "get_" #name);

#define ADD_SETGET_MODIFIERS(type, name, init) \
	ADD_SETGET(type, name##_base, init)        \
	ADD_SETGET(type, name##_add, 0.0f)         \
	ADD_SETGET(type, name##_mul, 1.0f)         \
	type get_##name() const;

#define ADD_SETGET_MODIFIERS_IMPL(class, type, name) \
	ADD_SETGET_IMPL(class, type, name##_base)        \
	ADD_SETGET_IMPL(class, type, name##_add)         \
	ADD_SETGET_IMPL(class, type, name##_mul)         \
	type class ::get_##name() const { return (name##_base + name##_add) * name##_mul; }

#define ADD_SETGET_MODIFIERS_PROPERTY(class, name)                     \
	ClassDB::bind_method(D_METHOD("get_" #name), &class ::get_##name); \
	ADD_GROUP(#name, #name "_");                                       \
	ADD_SETGET_PROPERTY(class, Variant::FLOAT, name##_base)            \
	ADD_SETGET_PROPERTY(class, Variant::FLOAT, name##_add)             \
	ADD_SETGET_PROPERTY(class, Variant::FLOAT, name##_mul)

#define TEST_ASSERT_ENABLED
#ifdef TEST_ASSERT_ENABLED
#define TEST_ASSERT(m_cond, m_msg) CRASH_COND_MSG(!(m_cond), m_msg)
#else
#define TEST_ASSERT(m_cond, m_msg) ((void)0)
#endif

template <typename T>
inline T swap_remove(std::vector<T> &vec, const u32 i) {
	TEST_ASSERT(i < vec.size(), "Index out of bounds");

	auto removed = vec[i];
	auto swap = vec.pop_back();
	vec[i] = swap;
	return removed;
}

#endif // PRELUDES_HPP