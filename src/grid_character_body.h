#include <godot_cpp/variant/vector2.hpp>
#include <godot_cpp/classes/node2d.hpp>

using namespace godot;

class GridCharacterBody : public Node2D {
    GDCLASS(GridCharacterBody, Node2D);

private:
    float step_offset = 0.0f;

protected:
    static void _bind_methods();

public:
    Vector2 size = Vector2(10.0f, 10.0f);
    void set_size(Vector2 value);
    Vector2 get_size() const;

    Vector2 velocity = Vector2(0.0f, 0.0f);
    void set_velocity(Vector2 value);
    Vector2 get_velocity() const;

    bool is_on_floor = false;
    bool hit_ceiling = false;
    bool hit_left_wall = false;
    bool hit_right_wall = false;

    int max_steps_height = 4;
    void set_max_steps_height(int value);
    int get_max_steps_height() const;

    bool stick_to_floor = true;

    bool collision = true;

    void move();

    virtual void _draw() override;
};
