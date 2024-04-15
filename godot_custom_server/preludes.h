#ifndef PRELUDES_HPP
#define PRELUDES_HPP

#include "core/error/error_macros.h"
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

const f32 f32_TAU = 6.28318530718;
const f32 f32_PI = 3.14159265359;

#define TEST_ASSERT(m_cond, m_msg) CRASH_COND_MSG(!(m_cond), m_msg)
// #define TEST_ASSERT(m_cond, m_msg) ((void)0)

template <typename T>
inline T swap_remove(std::vector<T> &vec, const u32 i) {
	TEST_ASSERT(i < vec.size(), "Index out of bounds");

	auto v = vec[i];
	vec[i] = vec[vec.size() - 1];
	vec.pop_back();
	return v;
}

#endif // PRELUDES_HPP