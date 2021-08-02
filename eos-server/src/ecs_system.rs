use crate::ecs_component::*;
use crate::ecs_resoure::*;
use bevy_ecs::prelude::*;
use eos_common::const_var::*;
use eos_common::math::*;
use glam::Vec2;
use std::time::Instant;

/// Update time.
pub fn update_sector_time(mut sector_time: ResMut<SectorTimeRes>) {
    sector_time.tick += 1;
    sector_time.delta = sector_time.last_instant.elapsed().as_secs_f32();
    sector_time.last_instant = Instant::now();
}

/// New fleets enter this sector.
pub fn get_new_fleet(sec_com: Res<SectorCommunicationRes>, sector_id: Res<SectorIdRes>) {
    let mut new_fleets = Vec::with_capacity(32);
    sec_com.receive_entity.try_iter().for_each(|new_fleet| {
        new_fleets.push(new_fleet.fleet_id);
        // TODO: Add new entity.
    });
    // Send update to GlobalList
    let _ = sec_com.fleet_current_sector_insert_send.send((new_fleets, sector_id.0));
}

// Receive packets and check for disconnect.
pub fn read_packet(query: Query<& ClientIdComp, Without<SystemComp>>, global_list: Res<GlobalListRes>) {
    // Lock global_list.
    let global_list_read = global_list.0.read();

    query.for_each_mut(|client_id| {
        if let Some(connection) = global_list_read.connected_client.get(&client_id.0) {
            connection.local_packets.read().iter().for_each(|packet| {
                match packet {
                    eos_common::packet_mod::ClientLocalPacket::Invalid => todo!(),
                    eos_common::packet_mod::ClientLocalPacket::ClientFleetWishLocation { fleet_id, location } => todo!(),
                    eos_common::packet_mod::ClientLocalPacket::Broadcast { message } => todo!(),
                }
            });
        }
    });
}

/// Update velocity and movement state.
pub fn fleet_movement(query: Query<(&mut VelocityComp, &mut MovementStateComp)>, sec_time: Res<SectorTimeRes>) {
    query.for_each_mut(|(mut vel, mut move_state)| {
        match move_state.0 {
            MovementState::Orbiting(_) => {
                // TODO
                if vel.is_changed() {
                    move_state.0 = MovementState::Breaking(1.0);
                }
            }
            MovementState::Breaking(break_mult) => {
                let force = steering_break(sec_time.delta, vel.0, 5.0 * break_mult);
                vel.0 += force;

                // Check if we are done breaking.
                if vel.0.length_squared() <= 0.1 {
                    vel.0 = Vec2::ZERO;
                    move_state.0 = MovementState::Still;
                }
            }
            MovementState::Seeking(mut target_distance) => {
                target_distance -= vel.0;
                let speed_square = vel.0.length_squared();
                let speed = speed_square.sqrt();
                // TODO: Make sure vel and target_distance is non zero.
                let heading = vel.0.angle_between(target_distance);

                // Reached target system.
                if target_distance.length_squared() <= time_to_zero_vel(speed_square, 5.0 * 5.0) && heading < 0.2 {
                    // Set vel to point toward target.
                    vel.0 = target_distance.clamp_length(speed, speed);
                    move_state.0 = MovementState::Breaking(1.0);
                }
                // Cruise if we are going at max speed and heading to target.
                else if speed >= 10.0 * 0.99 && heading < 0.1 {
                    // Set vel to max speed (or current speed if it is greater) toward target.
                    vel.0 = target_distance.clamp_length(speed, 10.0f32.max(speed));
                    move_state.0 = MovementState::Cruising(target_distance);
                }
                // Continue toward target.
                else {
                    // Use steering_local_seek().
                    let force = steering_local_seek(sec_time.delta, vel.0, speed, 5.0, target_distance);
                    // TODO: Hard cap velocity to max speed is boring.
                    vel.0 += force;
                    vel.0 = vel.0.clamp_length_max(10.0);
                }
            }
            MovementState::Cruising(mut target_distance) => {
                target_distance -= vel.0;

                // Reached target system.
                if target_distance.length_squared() <= time_to_zero_vel(vel.0.length_squared(), 5.0 * 5.0) {
                    move_state.0 = MovementState::Breaking(1.0);
                }
                // Something applied a force.
                else if vel.is_changed() {
                    move_state.0 = MovementState::Seeking(target_distance);
                }
            }
            MovementState::Still => {
                // Something applied a force.
                if vel.is_changed() {
                    move_state.0 = MovementState::Breaking(1.0);
                }
            }
        }
    });
}

pub fn apply_velocity(query: Query<(&VelocityComp, &mut LocationComp)>) {
    query.for_each_mut(|(vel, mut loc)| {
        loc.0.local_position += vel.0;
    });
}

// TODO: Send to sector and remove from here.
// TODO: Send remove notification to global list.
pub fn change_sector(
    query: Query<&mut LocationComp, Without<SystemComp>>,
    sec_com: Res<SectorCommunicationRes>,
    sector_id: Res<SectorIdRes>,
) {
    query.for_each_mut(|mut loc| {
        // Go one sector down.
        if loc.0.local_position.y > SECTOR_HALF_SIZE {
            loc.0.local_position.y -= SECTOR_SIZE;
        }
        // Go one sector up.
        else if loc.0.local_position.y < SECTOR_HALF_SIZE {
            loc.0.local_position.y += SECTOR_SIZE;
        }
        // Go one sector right.
        else if loc.0.local_position.x > SECTOR_HALF_SIZE {
            loc.0.local_position.x -= SECTOR_SIZE;
        }
        // Go one sector left.
        else if loc.0.local_position.x < SECTOR_HALF_SIZE {
            loc.0.local_position.x += SECTOR_SIZE;
        }
    });
}

// Event disconnect client.
