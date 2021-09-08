use crate::ecs_resources::*;
use crate::ecs_components::*;

use bevy_ecs::prelude::*;

pub fn time_system(mut time: ResMut<Time>) {
    time.tick += 1;
}

// pub fn apply_velocity(game_param: Res<GameParameter>, query: Query<(&mut Velocity, &mut Location)>) {
//     query.for_each_mut(|(mut velocity, mut location)| {
//         // Apply velocity by changing location.
//         location.location_on_tile += velocity.vel;

//         // Apply tile change if needed.
        
        
//         // Decrease velocity.
//         velocity.vel *= game_param.drag;
//     });
// }