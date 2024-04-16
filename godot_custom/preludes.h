#ifndef PRELUDES_H
#define PRELUDES_H

#include "core/error/error_macros.h"
#include <limits>

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
const f32 INF_F32 = std::numeric_limits<f32>::infinity();
const f32 NEG_INF_F32 = -std::numeric_limits<f32>::infinity();

/// Crash if condition is false.
#ifdef TOOLS_ENABLED
#define TEST_ASSERT(m_cond, m_msg) CRASH_COND_MSG(!(m_cond), m_msg)
#else
#define TEST_ASSERT(m_cond, m_msg) ((void)0)
#endif

#endif // PRELUDES_H