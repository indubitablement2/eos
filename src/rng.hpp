#ifndef RNG_HPP
#define RNG_HPP

// #define NDEBUG 
#include <assert.h>

#include "godot_cpp/godot.hpp"

using namespace godot;

namespace Rng {

inline void next(uint64_t &rng) {
    rng = rng * 2862933555777941757uLL + 3037000493uLL;
}

inline uint32_t gen_32bit(uint64_t &rng) {
    next(rng);
    return rng >> 32;
}

inline uint32_t gen_range_32bit(uint64_t &rng, uint32_t min, uint32_t max) {
    assert(min < max);
    assert(max > 0);

    return (gen_32bit(rng) % (max - min)) + min;
}

inline bool gen_bool(uint64_t &rng) {
    return gen_32bit(rng) & 1;
}

} // namespace Rng

#endif