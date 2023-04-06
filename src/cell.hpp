#ifndef CELL_HPP
#define CELL_HPP

#include "godot_cpp/godot.hpp"
#include "grid.h"

using namespace godot;

namespace Cell {

enum Shifts {
    SHIFT_UPDATED = 12,
    SHIFT_ACTIVE = 14,
    SHIFT_MOVING = 15,
    SHIFT_DIRECTION = 16,
    SHIFT_UNUSED = 17,
    SHIFT_MOVEMENT = 18,
    SHIFT_VALUE = 20,
    SHIFT_COLOR = 24,
};

enum Masks {
    MASK_MATERIAL = 0xFFF,
    // Alternate between 1, 2 and 3.
    // 0 used for inactive/new cell. eg. always update.
    MASK_UPDATED = 0b11 << Shifts::SHIFT_UPDATED,
    MASK_ACTIVE = 1 << Shifts::SHIFT_ACTIVE,
    MASK_MOVING = 1 << Shifts::SHIFT_MOVING,
    // Horizontal direction for moving cells.
    MASK_DIRECTION = 1 << Shifts::SHIFT_DIRECTION,
    // Free real estate!
    MASK_UNUSED = 1 << Shifts::SHIFT_UNUSED,
    // state: solid/powder/liquid/gas
    MASK_MOVEMENT = 0b11 << Shifts::SHIFT_MOVEMENT,
    MASK_VALUE = 0xF << Shifts::SHIFT_VALUE,
    MASK_COLOR = 0xFF << Shifts::SHIFT_COLOR,
};

inline uint32_t material_idx(const uint32_t &cell) {
    return cell & Masks::MASK_MATERIAL;
}

inline void set_material_idx(uint32_t &cell, const uint32_t material_idx) {
    cell = (cell & ~Masks::MASK_MATERIAL) | material_idx;
}

inline bool is_updated(const uint32_t &cell) {
    return (cell & Masks::MASK_UPDATED) == Grid::updated_bit;
}

inline void set_updated(uint32_t &cell) {
    cell = (cell & ~Masks::MASK_UPDATED) | Grid::updated_bit;
}

inline bool is_active(const uint32_t &cell) {
    return cell & Masks::MASK_ACTIVE;
}

// When inactive, set updated bit to 0 (never skip when it return to active).
inline void set_active(uint32_t &cell, const bool active) {
    if (active) {
        cell |= Masks::MASK_ACTIVE;
    } else {
        cell &= ~Masks::MASK_ACTIVE;
        cell &= ~Masks::MASK_UPDATED;
    }
}

inline int32_t value(const uint32_t &cell) {
    return (cell & Masks::MASK_VALUE) >> Shifts::SHIFT_VALUE;
}

inline void set_value(uint32_t &cell, int32_t value, bool saturate) {
    if (saturate) {
        if (value > 0xFF) {
            value = 0xFF;
        } else if (value < 0) {
            value = 0;
        }
    }

    cell = (cell & ~Masks::MASK_VALUE) | ((uint32_t)value << Shifts::SHIFT_VALUE);
}

} // namespace Cell

#endif