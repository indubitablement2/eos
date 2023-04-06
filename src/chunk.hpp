#ifndef CHUNK_HPP
#define CHUNK_HPP

// #define NDEBUG 
#include <assert.h>

#include <bit>

#include "godot_cpp/godot.hpp"
#include "grid.h"
#include "cell.hpp"

using namespace godot;

namespace Chunk {

struct ChunkActiveRect {
    int x_start;
    int x_end;
    int y_start;
    int y_end;
};

inline uint32_t get_rows(uint64_t chunk) {
    return (uint32_t)chunk;
}

inline uint32_t get_columns(uint64_t chunk) {
    return (uint32_t)(chunk >> 32);
}

inline ChunkActiveRect active_rect(uint64_t chunk) {
    ChunkActiveRect rect;

    if (chunk == 0) {
        rect.x_start = 0;
        rect.x_end = 0;
        rect.y_start = 0;
        rect.y_end = 0;
        return rect;
    }

    uint32_t rows = get_rows(chunk);
    uint32_t columns = get_columns(chunk);

    assert(rows > 0);
    assert(columns > 0);

    rect.x_start = std::countr_zero(columns);
    rect.x_end = 32 - std::countl_zero(columns);
    rect.y_start = std::countr_zero(rows);
    rect.y_end = 32 - std::countl_zero(rows);

    return rect;
}

// Rect needs to be within a single chunk.
inline void activate_rect(uint64_t *chunk_ptr, int x_offset, int y_offset, uint64_t width, uint64_t height) {
    assert(x_offset >= 0);
    assert(y_offset >= 0);
    assert(x_offset < 32);
    assert(y_offset < 32);
    assert(x_offset + width <= 32);
    assert(y_offset + height <= 32);
    assert(width > 0);
    assert(height > 0);

    *chunk_ptr |= ((1uLL << height) - 1uLL) << y_offset; // Set rows
    *chunk_ptr |= ((1uLL << width) - 1uLL) << (x_offset + 32); // Set columns
    
    assert(get_rows(*chunk_ptr) > 0);
    assert(get_columns(*chunk_ptr) > 0);
}

inline void activate_point(uint64_t *chunk_ptr, int local_x, int local_y) {
    activate_rect(chunk_ptr, local_x, local_y, 1, 1);
}

// Unlike other functions, this one also activate the cells.
inline void activate_neightbors(uint64_t *chunk_ptr, int local_x, int local_y, uint32_t *cell) {
    if (local_x <= 0 && local_y <= 0) {
        // Top left corner
        activate_rect(chunk_ptr, 0, 0, 2, 2);
        activate_rect(chunk_ptr - Grid::chunks_height, 31, 0, 1, 2);
        activate_rect(chunk_ptr - Grid::chunks_height - 1, 31, 31, 1, 1);
        activate_rect(chunk_ptr - 1, 0, 31, 2, 1);
    } else if (local_x <= 0 && local_y >= 31) {
        // Bottom left corner
        activate_rect(chunk_ptr, 0, 30, 2, 2);
        activate_rect(chunk_ptr - Grid::chunks_height, 31, 30, 1, 2);
        activate_rect(chunk_ptr - Grid::chunks_height + 1, 31, 0, 1, 1);
        activate_rect(chunk_ptr + 1, 0, 0, 2, 1);
    } else if (local_x >= 31 && local_y <= 0) {
        // Top right corner
        activate_rect(chunk_ptr, 30, 0, 2, 2);
        activate_rect(chunk_ptr + Grid::chunks_height, 0, 0, 1, 2);
        activate_rect(chunk_ptr + Grid::chunks_height - 1, 0, 31, 1, 1);
        activate_rect(chunk_ptr - 1, 30, 31, 2, 1);
    } else if (local_x >= 31 && local_y >= 31) {
        // Bottom right corner
        activate_rect(chunk_ptr, 30, 30, 2, 2);
        activate_rect(chunk_ptr + Grid::chunks_height, 0, 30, 1, 2);
        activate_rect(chunk_ptr + Grid::chunks_height + 1, 0, 0, 1, 1);
        activate_rect(chunk_ptr + 1, 30, 0, 2, 1);
    } else if (local_x <= 0) {
        // Left edge
        activate_rect(chunk_ptr, 0, local_y - 1, 2, 3);
        activate_rect(chunk_ptr - Grid::chunks_height, 31, local_y - 1, 1, 3);
    } else if (local_y <= 0) {
        // Top edge
        activate_rect(chunk_ptr, local_x - 1, 0, 3, 2);
        activate_rect(chunk_ptr - 1, local_x - 1, 31, 3, 1);
    } else if (local_x >= 31) {
        // Right edge
        activate_rect(chunk_ptr, 30, local_y - 1, 2, 3);
        activate_rect(chunk_ptr + Grid::chunks_height, 0, local_y - 1, 1, 3);
    } else if (local_y >= 31) {
        // Bottom edge
        activate_rect(chunk_ptr, local_x - 1, 30, 3, 2);
        activate_rect(chunk_ptr + 1, local_x - 1, 0, 3, 1);
    } else {
        // Middle
        activate_rect(chunk_ptr, local_x - 1, local_y - 1, 3, 3);
    }

    Cell::set_active(*(cell - 1), true);
    Cell::set_active(*cell, true);
    Cell::set_active(*(cell + 1), true);
    Cell::set_active(*(cell - Grid::width - 1), true);
    Cell::set_active(*(cell - Grid::width), true);
    Cell::set_active(*(cell - Grid::width + 1), true);
    Cell::set_active(*(cell + Grid::width - 1), true);
    Cell::set_active(*(cell + Grid::width), true);
    Cell::set_active(*(cell + Grid::width + 1), true);
}

// Only works for offset up to 32.
inline void offset_chunk_local_position(
    uint64_t *&chunk_ptr,
    int &local_x,
    int &local_y,
    int offset_x,
    int offset_y
) {
    assert(offset_x <= 32);
    assert(offset_y <= 32);
    assert(offset_x >= -32);
    assert(offset_y >= -32);
    
    local_x += offset_x;
    local_y += offset_y;

    if (local_x < 0) {
        chunk_ptr -= Grid::chunks_height;
        local_x += 32;
    } else if (local_x >= 32) {
        chunk_ptr += Grid::chunks_height;
        local_x -= 32;
    }

    if (local_y < 0) {
        chunk_ptr -= 1;
        local_y += 32;
    } else if (local_y >= 32) {
        chunk_ptr += 1;
        local_y -= 32;
    }

    assert(local_x >= 0);
    assert(local_y >= 0);
    assert(local_x < 32);
    assert(local_y < 32);
}

inline void activate_neightbors_offset(
    uint64_t *chunk_ptr,
    int local_x,
    int local_y,
    int offset_x,
    int offset_y,
    uint32_t *other_ptr
) {
    offset_chunk_local_position(chunk_ptr, local_x, local_y, offset_x, offset_y);
    activate_neightbors(chunk_ptr, local_x, local_y, other_ptr);
}

} // namespace Chunk

#endif