// #define NDEBUG 
#include <assert.h>

#include <godot_cpp/variant/utility_functions.hpp>
#include <godot_cpp/classes/image_texture.hpp>
#include <godot_cpp/variant/vector2.hpp>

#include "grid.h"
#include "cell.hpp"
#include "rng.hpp"
#include "chunk.hpp"

using namespace godot;

namespace Step {

void step_reaction(
    uint32_t &cell_material_idx,
    bool &active,
    bool &changed,
    uint64_t *chunk_ptr,
    int local_x,
    int local_y,
    uint32_t *other_ptr,
    int other_offset_x,
    int other_offset_y,
    uint64_t &rng
) {
    auto other_material_idx = Cell::material_idx(*other_ptr);
    
    bool swap;
    CellMaterial *mat;
    uint32_t reaction_range_idx;
    if (cell_material_idx > other_material_idx) {
        swap = true;
        mat = Grid::cell_materials + other_material_idx;
        reaction_range_idx = cell_material_idx - other_material_idx;
    } else {
        swap = false;
        mat = Grid::cell_materials + cell_material_idx;
        reaction_range_idx = other_material_idx - cell_material_idx;
    }

    if (reaction_range_idx >= mat->reaction_ranges_len) {
        return;
    }

    uint64_t reaction_range = *(mat->reaction_ranges + reaction_range_idx);
    uint64_t reaction_start = reaction_range & 0xffffffff;
    uint64_t reaction_end = reaction_range >> 32;

    if (reaction_start >= reaction_end) {
        return;
    }

    active = true;

    for (uint64_t i = reaction_start; i < reaction_end; i++) {
        CellReaction reaction = *(mat->reactions + i);

        if (reaction.probability >= Rng::gen_32bit(rng)) {
            uint32_t out1, out2;
            if (swap) {
                out1 = reaction.mat_idx_out2;
                out2 = reaction.mat_idx_out1;
            } else {
                out1 = reaction.mat_idx_out1;
                out2 = reaction.mat_idx_out2;
            }
            
            if (out1 != cell_material_idx) {
                cell_material_idx = out1;
                changed = true;
            }

            if (out2 != other_material_idx) {
                Cell::set_material_idx(*other_ptr, out2);
                Chunk::activate_neightbors_offset(
                    chunk_ptr,
                    local_x,
                    local_y,
                    other_offset_x,
                    other_offset_y,
                    other_ptr
                );
            }

            return;
        }
    }
}

void swap_cells(
    uint32_t cell,
    uint32_t *cell_ptr,
    uint64_t *chunk_ptr,
    int local_x,
    int local_y,
    uint32_t *other_ptr,
    int other_offset_x,
    int other_offset_y
) {
    *cell_ptr = *other_ptr;
    *other_ptr = cell;
    Chunk::activate_neightbors(chunk_ptr, local_x, local_y, cell_ptr);
    Chunk::activate_neightbors_offset(
        chunk_ptr,
        local_x,
        local_y,
        other_offset_x,
        other_offset_y,
        other_ptr
    );
}

void step_cell(
    uint32_t *cell_ptr,
    uint64_t *chunk_ptr,
    int local_x,
    int local_y,
    uint64_t &rng
) {
    uint32_t cell = *cell_ptr;

    if (!Cell::is_active(cell) || Cell::is_updated(cell)) {
        return;
    }

    bool active = false;
    // Activate a 5x5 rect around the cell.
    bool changed = false;

    uint32_t cell_material_idx = Cell::material_idx(cell);

    // Reactions
    // x x x
    // . o x
    // . . .

    step_reaction(
        cell_material_idx,
        active,
        changed,
        chunk_ptr,
        local_x,
        local_y,
        cell_ptr + 1,
        1,
        0,
        rng
    );

    step_reaction(
        cell_material_idx,
        active,
        changed,
        chunk_ptr,
        local_x,
        local_y,
        cell_ptr - Grid::width - 1,
        -1,
        -1,
        rng
    );

    step_reaction(
        cell_material_idx,
        active,
        changed,
        chunk_ptr,
        local_x,
        local_y,
        cell_ptr - Grid::width,
        0,
        -1,
        rng
    );

    step_reaction(
        cell_material_idx,
        active,
        changed,
        chunk_ptr,
        local_x,
        local_y,
        cell_ptr - Grid::width + 1,
        1,
        -1,
        rng
    );

    Cell::set_material_idx(cell, cell_material_idx);
    Cell::set_updated(cell);

    // Movement

    CellMaterial *mat = Grid::cell_materials + cell_material_idx;

    auto b = cell_ptr + Grid::width;
    CellMaterial *b_mat = Grid::cell_materials + Cell::material_idx(*b);

    auto br = cell_ptr + Grid::width + 1;
    CellMaterial *br_mat = Grid::cell_materials + Cell::material_idx(*br);

    auto bl = cell_ptr + Grid::width - 1;
    CellMaterial *bl_mat = Grid::cell_materials + Cell::material_idx(*bl);

    auto l = cell_ptr - 1;
    CellMaterial *l_mat = Grid::cell_materials + Cell::material_idx(*l);

    auto r = cell_ptr + 1;
    CellMaterial *r_mat = Grid::cell_materials + Cell::material_idx(*r);

    auto t = cell_ptr - Grid::width;
    CellMaterial *t_mat = Grid::cell_materials + Cell::material_idx(*t);

    auto tl = cell_ptr - Grid::width - 1;
    CellMaterial *tl_mat = Grid::cell_materials + Cell::material_idx(*tl);

    auto tr = cell_ptr - Grid::width + 1;
    CellMaterial *tr_mat = Grid::cell_materials + Cell::material_idx(*tr);

    switch (mat->cell_movement) {
        case Grid::CELL_MOVEMENT_SOLID: {
            
        }
        break;
        case Grid::CELL_MOVEMENT_POWDER: {
            if (b_mat->density < mat->density) {
                swap_cells(
                    cell,
                    cell_ptr,
                    chunk_ptr,
                    local_x,
                    local_y,
                    b,
                    0,
                    1
                );
                return;
            } else if (bl_mat->density < mat->density && br_mat->density < mat->density) {
                if (Rng::gen_bool(rng)) {
                    swap_cells(
                        cell,
                        cell_ptr,
                        chunk_ptr,
                        local_x,
                        local_y,
                        bl,
                        -1,
                        1
                    );
                    return;
                } else {
                    swap_cells(
                        cell,
                        cell_ptr,
                        chunk_ptr,
                        local_x,
                        local_y,
                        br,
                        1,
                        1
                    );
                    return;
                }
            } else if (bl_mat->density < mat->density) {
                swap_cells(
                    cell,
                    cell_ptr,
                    chunk_ptr,
                    local_x,
                    local_y,
                    bl,
                    -1,
                    1
                );
                return;
            } else if (br_mat->density < mat->density) {
                swap_cells(
                    cell,
                    cell_ptr,
                    chunk_ptr,
                    local_x,
                    local_y,
                    br,
                    1,
                    1
                );
                return;
            }
        }
        break;
        case Grid::CELL_MOVEMENT_LIQUID: {
            // TODO: Movemet speed.

            const uint32_t dissipate_chance = 8388608;

            if (b_mat->density < mat->density) {
                swap_cells(
                    cell,
                    cell_ptr,
                    chunk_ptr,
                    local_x,
                    local_y,
                    b,
                    0,
                    1
                );
                return;
            // } else if (bl_mat->density < mat->density && br_mat->density < mat->density) {

            } else if (bl_mat->density < mat->density) {
                Cell::set_value(cell, 1, false);

                swap_cells(
                    cell,
                    cell_ptr,
                    chunk_ptr,
                    local_x,
                    local_y,
                    bl,
                    -1,
                    1
                );
                return;
            } else if (br_mat->density < mat->density) {
                Cell::set_value(cell, 0, false);

                swap_cells(
                    cell,
                    cell_ptr,
                    chunk_ptr,
                    local_x,
                    local_y,
                    br,
                    1,
                    1
                );
                return;
            } else if (l_mat->density < mat->density && r_mat->density < mat->density) {
                if (Cell::value(cell)) {
                    if (Rng::gen_32bit(rng) < dissipate_chance) {
                        cell = 0;
                        changed = true;
                    } else {
                        swap_cells(
                            cell,
                            cell_ptr,
                            chunk_ptr,
                            local_x,
                            local_y,
                            l,
                            -1,
                            0
                        );
                        return;
                    }
                } else {
                    if (Rng::gen_32bit(rng) < dissipate_chance) {
                        cell = 0;
                        changed = true;
                    } else {
                        swap_cells(
                            cell,
                            cell_ptr,
                            chunk_ptr,
                            local_x,
                            local_y,
                            r,
                            1,
                            0
                        );
                        return;
                    }
                }
            } else if (l_mat->density < mat->density) {
                if (Rng::gen_32bit(rng) < dissipate_chance) {
                    cell = 0;
                    changed = true;
                } else {
                    Cell::set_value(cell, 1, false);
                    
                    swap_cells(
                        cell,
                        cell_ptr,
                        chunk_ptr,
                        local_x,
                        local_y,
                        l,
                        -1,
                        0
                    );
                    return;
                }
            } else if (r_mat->density < mat->density) {
                if (Rng::gen_32bit(rng) < dissipate_chance) {
                    cell = 0;
                    changed = true;
                } else {
                    Cell::set_value(cell, 0, false);

                    swap_cells(
                        cell,
                        cell_ptr,
                        chunk_ptr,
                        local_x,
                        local_y,
                        r,
                        1,
                        0
                    );
                    return;
                }
            }
        }
        break;
        case Grid::CELL_MOVEMENT_GAS: {
            // TODO: Reverse liquid movement.
        }
        break;
    }

    if (changed) {
        *cell_ptr = cell;

        Chunk::activate_neightbors(chunk_ptr, local_x, local_y, cell_ptr);
    } else if (active) {
        Cell::set_active(cell, true);
        *cell_ptr = cell;

        Chunk::activate_point(chunk_ptr, local_x, local_y);
    } else {
        Cell::set_active(cell, false);
        *cell_ptr = cell;
    }
}

void step_chunk(
    uint64_t chunk,
    uint64_t *chunk_ptr,
    uint32_t *cell_start,
    uint64_t &rng
) {
    if (chunk == 0) {
        return;
    }

    auto rows = Chunk::get_rows(chunk);
    auto rect = Chunk::active_rect(chunk);

    // Alternate between left and right.
    int x_start;
    int x_end;
    int x_step;
    if ((Grid::tick & 1) == 0) {
        x_start = rect.x_start;
        x_end = rect.x_end;
        x_step = 1;
    } else {
        x_start = rect.x_end - 1;
        x_end = rect.x_start - 1;
        x_step = -1;
    }

    // Iterate over each cell in the chunk.
    for (int local_y = rect.y_start; local_y < rect.y_end; local_y++) {
        if ((rows & (1 << local_y)) == 0) {
            continue;
        }

        int local_x = x_start;
        while (local_x != x_end) {
            auto cell_ptr = cell_start + local_x + local_y * Grid::width;
            step_cell(cell_ptr, chunk_ptr, local_x, local_y, rng);

            local_x += x_step;
        }
    }
}

void step_column(int column_idx) {
    uint64_t rng = (uint64_t)column_idx * (uint64_t)Grid::tick * 6364136223846792969uLL;

    uint64_t *chunk_end_ptr = Grid::chunks + column_idx * Grid::chunks_height;
    uint64_t *next_chunk_ptr = chunk_end_ptr + (Grid::chunks_height - 2);

    auto next_chunk = *next_chunk_ptr;
    *next_chunk_ptr = 0;
    
    auto cell_start = Grid::cells + ((Grid::height - 32) * Grid::width + column_idx * 32);

    // Iterate over each chunk from the bottom.
    while (next_chunk_ptr > chunk_end_ptr) {
        auto chunk = next_chunk;
        auto chunk_ptr = next_chunk_ptr;

        next_chunk_ptr -= 1;
        next_chunk = *next_chunk_ptr;
        *next_chunk_ptr = 0;

        cell_start -= 32 * Grid::width;

        step_chunk(chunk, chunk_ptr, cell_start, rng);
    }
}

void pre_step() {
    Grid::updated_bit >>= Cell::Shifts::SHIFT_UPDATED;
    Grid::updated_bit %= 3;
    Grid::updated_bit += 1;
    Grid::updated_bit <<= Cell::Shifts::SHIFT_UPDATED;

    Grid::tick++;
}

} // namespace Step

void Grid::_bind_methods() {
    ClassDB::bind_static_method(
        "Grid", 
        D_METHOD("delete_grid"),
        &Grid::delete_grid
    );
    ClassDB::bind_static_method(
        "Grid", 
        D_METHOD("new_empty", "width", "height"),
        &Grid::new_empty
    );
    ClassDB::bind_static_method(
        "Grid", 
        D_METHOD("get_size"),
        &Grid::get_size
    );
    ClassDB::bind_static_method(
        "Grid", 
        D_METHOD("get_size_chunk"),
        &Grid::get_size_chunk
    );
    ClassDB::bind_static_method(
        "Grid",
        D_METHOD("update_texture_data", "texture", "position"),
        &Grid::update_texture_data
    );
    ClassDB::bind_static_method(
        "Grid",
        D_METHOD("step_manual"),
        &Grid::step_manual
    );

    ClassDB::bind_static_method(
        "Grid",
        D_METHOD("get_tick"),
        &Grid::get_tick
    );
    ClassDB::bind_static_method(
        "Grid",
        D_METHOD("get_cell_material_idx", "position"),
        &Grid::get_cell_material_idx
    );

    ClassDB::bind_static_method(
        "Grid",
        D_METHOD("init_materials", "num_materials"),
        &Grid::init_materials
    );
    ClassDB::bind_static_method(
        "Grid",
        D_METHOD(
            "add_material",
            "cell_movement",
            "density",
            "durability",
            "cell_collision",
            "friction",
            "reactions",
            "idx"
        ),
        &Grid::add_material
    );

    ClassDB::bind_static_method(
        "Grid",
        D_METHOD("is_chunk_active", "position"),
        &Grid::is_chunk_active
    );
    ClassDB::bind_static_method(
        "Grid",
        D_METHOD("free_memory"),
        &Grid::free_memory
    );
    ClassDB::bind_static_method(
        "Grid",
        D_METHOD("print_materials"),
        &Grid::print_materials
    );
    ClassDB::bind_static_method(
        "Grid",
        D_METHOD("run_tests"),
        &Grid::run_tests
    );

    // ADD_GROUP("Test group", "group_");
	// ADD_SUBGROUP("Test subgroup", "group_subgroup_");

    BIND_CONSTANT(GRID_SCALE);

    BIND_ENUM_CONSTANT(CELL_COLLISION_SOLID);
    BIND_ENUM_CONSTANT(CELL_COLLISION_PLATFORM);
    BIND_ENUM_CONSTANT(CELL_COLLISION_LIQUID);
    BIND_ENUM_CONSTANT(CELL_COLLISION_NONE);

    BIND_ENUM_CONSTANT(CELL_MOVEMENT_SOLID);
    BIND_ENUM_CONSTANT(CELL_MOVEMENT_POWDER);
    BIND_ENUM_CONSTANT(CELL_MOVEMENT_LIQUID);
    BIND_ENUM_CONSTANT(CELL_MOVEMENT_GAS);
}

void Grid::delete_grid() {
    if (cells != nullptr) {
        UtilityFunctions::print("Deleting grid");

        delete [] cells;
        cells = nullptr;
        width = 0;
        height = 0;

        delete [] chunks;
        chunks = nullptr;
        chunks_width = 0;
        chunks_height = 0;
    }
}

void Grid::new_empty(int wish_width, int wish_height) {
    delete_grid();

    chunks_width = std::max(wish_width / 32, 3);
    // TODO: Make sure that the height is a multiple of 64/8.
    chunks_height = std::max(wish_height / 32, 3);
    chunks = new uint64_t[chunks_width * chunks_height];
    // Set all chunk to active.
    for (int i = 0; i < chunks_width * chunks_height; i++) {
        chunks[i] = 0xFFFFFFFFFFFFFFFF;
    }

    width = chunks_width * 32;
    height = chunks_height * 32;
    cells = new uint32_t[width * height];
    // Set all cells to empty.
    for (int i = 0; i < width * height; i++) {
        cells[i] = 0;
    }

    // iterate over all cells.
    for (int x = 32; x < width - 32; x++) {
        for (int y = 32; y < height - 32; y++) {
            uint32_t cell = 0;

            if (x == 100 && y == 100) {
                Cell::set_material_idx(cell, 3);
            } else if (y > height - 40) {
                Cell::set_material_idx(cell, 2);
            } else if (x < 60 || x > 200) {
                Cell::set_material_idx(cell, 2);
            } else if (y < 60) {
                Cell::set_material_idx(cell, 1);
            }

            auto cell_ptr = cells + (y * width + x);
            Cell::set_active(cell, true);
            *cell_ptr = cell;
        }
    }
}

Vector2i Grid::get_size() {
    return Vector2i(width, height);
}

Vector2i Grid::get_size_chunk() {
    return Vector2i(chunks_width, chunks_height);
}

void Grid::update_texture_data(Ref<ImageTexture> texture, Vector2i position) {
    if (cells == nullptr) {
        UtilityFunctions::push_warning("Grid is not initialized");
        return;
    }

    int texture_width = texture->get_width();
    int texture_height = texture->get_height();

    if (texture_width == 0 || texture_height == 0) {
        UtilityFunctions::push_warning("Texture has zero size");
        return;
    }

    // TODO: Reuse this buffer.
    PackedByteArray data = PackedByteArray();
    data.resize(texture_width * texture_height * sizeof(uint32_t));
    auto data_ptr = reinterpret_cast<uint32_t*>(data.ptrw());

    int i = 0;
    for (int y = position.y; y < position.y + texture_height; y++) {
        for (int x = position.x; x < position.x + texture_width; x++) {
            data_ptr[i] = get_cell_checked(x, y);
            i++;
        }
    }

    Ref<Image> image = Image::create_from_data(
        texture_width,
        texture_height,
        false,
        Image::FORMAT_RF,
        data
    );

    UtilityFunctions::print("texture size: ", texture->get_size());
    UtilityFunctions::print("image size: ", image->get_size());

    texture->update(image);
}

uint32_t Grid::get_cell_checked(int x, int y) {
    if (x < 0 || x >= width || y < 0 || y >= height) {
        // TODO: empty/water/sand/rock/empty/lava
        return 0;
    }

    return cells[y * width + x];
}

void Grid::step_manual() {
    if (cells == nullptr) {
        UtilityFunctions::push_warning("Grid is not initialized");
        return;
    }

    Step::pre_step();

    for (int column_idx = 1; column_idx < chunks_width - 1; column_idx++) {
        Step::step_column(column_idx);
    }
}

void delete_materials() {
    if (Grid::cell_materials != nullptr) {
        UtilityFunctions::print("Deleting materials");

        for (int i = 0; i < Grid::cell_materials_len; i++) {
            CellMaterial *mat = Grid::cell_materials + i;
            if (mat->reaction_ranges_len > 0) {
                delete [] mat->reaction_ranges;
                delete [] mat->reactions;
            }
        }

        delete [] Grid::cell_materials;
        Grid::cell_materials = nullptr;
        Grid::cell_materials_len = 0;
    }
}

void Grid::init_materials(int num_materials) {
    delete_materials();

    if (num_materials > 0) {
        cell_materials = new CellMaterial[num_materials];
        cell_materials_len = num_materials;
    }
}

void Grid::add_material(
    int cell_movement,
    int density,
    int durability,
    int cell_collision,
    float friction,
    // probability, out1, out2
    Array reactions,
    int idx
) {
    if (cell_materials == nullptr) {
        UtilityFunctions::push_error("Materials not initialized");
        return;
    }

    CellMaterial mat = CellMaterial();
    mat.cell_movement = cell_movement;
    mat.density = density;
    mat.durability = durability;
    mat.cell_collision = cell_collision;
    mat.friction = friction;

    int num_reaction = 0;
    for (int i = 0; i < reactions.size(); i++) {
        Array r = reactions[i];
        if (!r.is_empty()) {
            mat.reaction_ranges_len = i + 1;
            num_reaction += r.size();
        }
    }
    if (mat.reaction_ranges_len != 0) {
        assert(num_reaction > 0);

        mat.reaction_ranges = new uint64_t[mat.reaction_ranges_len];
        mat.reactions = new CellReaction[num_reaction];

        uint64_t next_reaction_idx = 0;

        assert(mat.reaction_ranges_len <= reactions.size());
        for (int i = 0; i < mat.reaction_ranges_len; i++) {
            uint64_t reactions_start = next_reaction_idx;

            assert(reactions.size() >= i);
            Array reactions_with = reactions[i];
            for (int j = 0; j < reactions_with.size(); j++) {
                Array reaction_data = reactions_with[j];
                assert(reaction_data.size() == 3);
                CellReaction reaction = {
                    reaction_data[0],
                    reaction_data[1],
                    reaction_data[2]
                };
                assert(next_reaction_idx < num_reaction);
                mat.reactions[next_reaction_idx] = reaction;

                next_reaction_idx++;
            }

            assert(i < mat.reaction_ranges_len);
            uint64_t reactions_end = next_reaction_idx;
            if (reactions_start == reactions_end) {
                mat.reaction_ranges[i] = 0;
            } else {
                mat.reaction_ranges[i] = reactions_start | (reactions_end << 32);
            }
        }

        assert(next_reaction_idx == num_reaction);
    }

    assert(idx < cell_materials_len);
    cell_materials[idx] = mat;
}

bool Grid::is_chunk_active(Vector2i position) {
    if (position.x < 0 || position.y < 0 || position.x >= chunks_width || position.y >= chunks_height) {
        return false;
    }

    return *(chunks + position.x * chunks_height + position.y);
}

void Grid::free_memory() {
    delete_materials();
    delete_grid();
}

int64_t Grid::get_tick() {
    return tick;
}

uint32_t Grid::get_cell_material_idx(Vector2i position) {
    if (position.x < 0 || position.y < 0 || position.x >= width || position.y >= height) {
        return 0;
    }

    return Cell::material_idx(cells[position.x + position.y * width]);
}

void Grid::print_materials() {
    UtilityFunctions::print("num materials: ", cell_materials_len);

    for (int i = 0; i < cell_materials_len; i++) {
        CellMaterial &mat = cell_materials[i];
        UtilityFunctions::print("-----------", i, "-----------");
        UtilityFunctions::print("cell_movement ", mat.cell_movement);
        UtilityFunctions::print("density ", mat.density);
        UtilityFunctions::print("durability ", mat.durability);
        UtilityFunctions::print("cell_collision ", mat.cell_collision);
        UtilityFunctions::print("friction ", mat.friction);

        UtilityFunctions::print("reaction_ranges_len ", mat.reaction_ranges_len);
        for (int j = 0; j < mat.reaction_ranges_len; j++) {
            UtilityFunctions::print("   reaction_range ", j);
            uint64_t reaction_range = *(mat.reaction_ranges + j);
            uint64_t reaction_start = reaction_range & 0xffffffff;
            uint64_t reaction_end = reaction_range >> 32;
            UtilityFunctions::print("       reaction_start ", reaction_start);
            UtilityFunctions::print("       reaction_end ", reaction_end);
            for (int k = reaction_start; k < reaction_end; k++) {
                CellReaction reaction = *(mat.reactions + k);
                UtilityFunctions::print("          reaction ", k);
                UtilityFunctions::print("          in1 ", i);
                UtilityFunctions::print("          in2 ", i + j);
                UtilityFunctions::print("          probability ", reaction.probability);
                UtilityFunctions::print("          out1 ", reaction.mat_idx_out1);
                UtilityFunctions::print("          out2 ", reaction.mat_idx_out2);
            }
        }
    }
}

namespace Test {

void test_activate_chunk() {
    Grid::new_empty(96, 96);
    auto chunk = Grid::chunks + Grid::chunks_height + 1;
    auto cell_ptr = Grid::cells + 32 + 32 * Grid::width;
    *chunk = 0;

    Chunk::activate_neightbors(
        chunk,
        15,
        15,
        cell_ptr + 15 + 15 * Grid::width
    );
    UtilityFunctions::print("activate center: OK");

    Chunk::activate_neightbors(
        chunk,
        0,
        0,
        cell_ptr
    );
    UtilityFunctions::print("activate top left: OK");

    Chunk::activate_neightbors(
        chunk,
        31,
        31,
        cell_ptr + 31 + 31 * Grid::width
    );
    UtilityFunctions::print("activate bottom right: OK");

    Chunk::activate_neightbors(
        chunk,
        31,
        0,
        cell_ptr + 31
    );
    UtilityFunctions::print("activate top right: OK");

    Chunk::activate_neightbors(
        chunk,
        0,
        31,
        cell_ptr + 31 * Grid::width
    );
    UtilityFunctions::print("activate bottom left: OK");

    Chunk::activate_neightbors(
        chunk,
        15,
        0,
        cell_ptr + 15
    );
    UtilityFunctions::print("activate top center: OK");

    Chunk::activate_neightbors(
        chunk,
        15,
        31,
        cell_ptr + 15 + 31 * Grid::width
    );
    UtilityFunctions::print("activate bottom center: OK");
    
    Chunk::activate_neightbors(
        chunk,
        0,
        15,
        cell_ptr + 15 * Grid::width
    );
    UtilityFunctions::print("activate left center: OK");

    Chunk::activate_neightbors(
        chunk,
        31,
        15,
        cell_ptr + 15 + 31 * Grid::width
    );
    UtilityFunctions::print("activate right center: OK");

    Grid::delete_grid();
}

void test_activate_rect() {
    uint64_t *chunk = new uint64_t;
    *chunk = 0;

    Chunk::activate_rect(chunk, 0, 0, 32, 32);
    assert(*chunk == 0xffffffffffffffff);
    UtilityFunctions::print("activate full rect: OK");

    uint64_t rng = 12345789;
    for (int i = 0; i < 10000; i++) {
        *chunk = 0;

        int x_offset = Rng::gen_range_32bit(rng, 0, 32);
        int y_offset = Rng::gen_range_32bit(rng, 0, 32);
        uint64_t width = Rng::gen_range_32bit(rng, 0, 32 - x_offset) + 1;
        uint64_t height = Rng::gen_range_32bit(rng, 0, 32 - y_offset) + 1;

        Chunk::activate_rect(chunk, x_offset, y_offset, width, height);

        auto rect = Chunk::active_rect(*chunk);
        assert(rect.x_start == x_offset);
        assert(rect.y_start == y_offset);
        assert(rect.x_end == x_offset + width);
        assert(rect.y_end == y_offset + height);
    }
    UtilityFunctions::print("activate random rects: OK");

    delete chunk;
}

void test_rng() {
    int num_tests = 100000;
    int num_true = 0;
    
    uint64_t rng = 12345789;

    for (int i = 0; i < num_tests; i++) {
        if (Rng::gen_bool(rng)) {
            num_true++;
        }
    }
    double true_bias = (double)num_true / (double)num_tests;
    UtilityFunctions::print("rng true bias ", true_bias);
    assert(true_bias > 0.45 && true_bias < 0.55);
    assert(true_bias != 0.5);

    UtilityFunctions::print("rng non-bias: OK");
}

} // namespace Test

void Grid::run_tests() {
    UtilityFunctions::print("---------- test_activate_chunk: STARTED");
    Test::test_activate_chunk();

    UtilityFunctions::print("---------- test_activate_rect: STARTED");
    Test::test_activate_rect();

    UtilityFunctions::print("---------- test_rng: STARTED");
    Test::test_rng();

    UtilityFunctions::print("---------- All tests passed!");
}
