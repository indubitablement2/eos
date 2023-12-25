#ifndef RNG_HPP
#define RNG_HPP

#include "preludes.h"

namespace Rng {

inline void next(u64 &rng) {
	rng = rng * 2862933555777941757uLL + 3037000493uLL;
}

inline u32 gen_u32(u64 &rng) {
	next(rng);
	return rng >> 32;
}

inline u32 gen_range_u32(u64 &rng, u32 min, u32 max) {
	TEST_ASSERT(min < max, "min must be less than max");
	TEST_ASSERT(max > 0, "max must be greater than 0");

	return (gen_u32(rng) % (max - min)) + min;
}

inline bool gen_bool(u64 &rng) {
	return gen_u32(rng) & 1;
}

inline f32 gen_f32(u64 &rng) {
	return (f32)gen_u32(rng) / (f32)MAX_U32;
}

// probability > 1 always return true.
// probability < 0 always return false.
inline bool gen_probability(u64 &rng, f32 probability) {
	return (f32)gen_u32(rng) < (probability * (f32)MAX_U32);
}

} // namespace Rng

#endif