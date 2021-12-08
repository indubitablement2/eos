use std::cmp::Ordering;

use crate::generation::GenerationParameters;
use crate::intersection::{Collider, ColliderId, SAPRow, SystemIntersectionPipeline};
use crate::res_parameters::ParametersRes;
use glam::Vec2;
use indexmap::IndexMap;
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemId(u32);

pub struct SystemsRes {
    pub systems: IndexMap<SystemId, System>,
}
impl SystemsRes {
    const SYSTEM_SAFE_DISTANCE: f32 = 32.0;

    pub fn generate(
        generation_parameters: &GenerationParameters,
        parameters_res: &ParametersRes,
    ) -> (Self, SystemIntersectionPipeline) {
        let mut next_system_id = 0u32;
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(generation_parameters.seed);

        let mut systems = IndexMap::new();
        let mut system_colliders = Vec::new();

        let bound = parameters_res.bound;

        // How many systems we will try to place randomly.
        let num_attempt = (bound.powi(2) / System::RADIUS_MAX.powi(2)) as usize;
        debug!("Num system generation attempt: {}.", num_attempt);

        for attempt_number in 0..num_attempt {
            let completion = attempt_number as f32 / num_attempt as f32;
            let uv: Vec2 = rng.gen::<Vec2>();

            // Check if we are within metascape bound.
            let position: Vec2 = uv * 2.0 * bound - bound;
            if position.length_squared() > bound.powi(2) {
                continue;
            }

            // Check density.
            if completion > generation_parameters.system_density.sample(uv) {
                continue;
            }

            // Generate a random system.
            let system_size_multiplier = generation_parameters.system_size.sample(uv);
            let system = System::generate_system(system_size_multiplier, &mut rng);

            // Check if system bound is within metascape bound.
            let system_bound_squared = position.length_squared() + system.radius.powi(2);
            if system_bound_squared > bound.powi(2) {
                continue;
            }

            // Test if it overlap with any existing system.
            let test_collider = Collider {
                radius: system.radius + Self::SYSTEM_SAFE_DISTANCE,
                position,
            };
            for other_collider in system_colliders.iter() {
                if test_collider.intersection_test(*other_collider) {
                    continue;
                }
            }

            // Create system Collider.
            let collider = Collider {
                radius: system.radius,
                position,
            };

            // Add system.
            system_colliders.push(collider);
            systems.insert(SystemId(next_system_id), system);
            next_system_id += 1
        }

        debug!("Num system generated: {}.", system_colliders.len());

        // Create SystemIntersectionPipeline.
        let system_intersection_pipeline = create_system_intersection_pipeline(&systems, system_colliders);

        (Self { systems }, system_intersection_pipeline)
    }
}

pub enum CelestialBodyType {
    /// Used as a center body for system with multiple stars.
    Nothing,
    Star,
    Planet,
}

pub struct CelestialBody {
    pub celestial_body_type: CelestialBodyType,
    pub radius: f32,
    pub orbit_radius: f32,
    pub orbit_time: u64,
    pub moons: Vec<CelestialBody>,
}

pub struct System {
    /// Edge of the outtermost body.
    pub radius: f32,
    pub center_body: CelestialBody,
}
impl System {
    pub const RADIUS_MIN: f32 = 32.0;
    pub const RADIUS_MAX: f32 = 128.0;
    const ORBIT_TIME_MIN_PER_RADIUS: u64 = 600;

    /// Return a randomly generated System with its radius.
    /// The ColliderId provided is invalid and needs to be replaced.
    fn generate_system(size_multiplier: f32, rng: &mut Xoshiro256PlusPlus) -> Self {
        // Get system radius.
        let system_radius = (rng.gen_range((Self::RADIUS_MIN / 0.8)..(Self::RADIUS_MAX * 0.8)) * size_multiplier)
            .clamp(System::RADIUS_MIN, System::RADIUS_MAX);

        // Create System center body.
        let mut center_body = CelestialBody {
            celestial_body_type: CelestialBodyType::Star,
            radius: 8.0,
            orbit_radius: 0.0,
            orbit_time: 0,
            moons: Vec::new(),
        };

        // Add bodies.
        let mut used_radius = center_body.radius;
        while used_radius < system_radius {
            let radius = 1.0;
            let orbit_radius = radius + used_radius + rng.gen_range(1.0..10.0);
            let orbit_time = Self::ORBIT_TIME_MIN_PER_RADIUS * orbit_radius as u64;

            let new_body = CelestialBody {
                celestial_body_type: CelestialBodyType::Planet,
                radius,
                orbit_radius,
                orbit_time,
                moons: Vec::new(),
            };

            center_body.moons.push(new_body);
            used_radius += orbit_radius + radius
        }

        Self {
            radius: used_radius,
            center_body,
        }
    }
}

fn create_system_intersection_pipeline(
    systems: &IndexMap<SystemId, System>,
    system_colliders: Vec<Collider>,
) -> SystemIntersectionPipeline {
    let min_collider_per_row = 8;
    let min_row_size = System::RADIUS_MAX * 3.0;
    let mut sip = SystemIntersectionPipeline::new();

    // Insert colliders.
    for (collider, system_id) in system_colliders.into_iter().zip(systems.keys()) {
        sip.snapshot.colliders.insert(ColliderId(system_id.0), collider);
        sip.snapshot
            .collider_custom_data
            .insert(ColliderId(system_id.0), system_id.0 as u64);
    }

    if sip.snapshot.colliders.is_empty() {
        return sip;
    }

    // Sort on y axis.
    sip.snapshot
        .colliders
        .sort_by(|_, v1, _, v2| v1.position.y.partial_cmp(&v2.position.y).unwrap_or(Ordering::Equal));

    // Prepare first row.
    let mut current_row = SAPRow::default();
    // First row's start should be very large negative number.
    current_row.start = -1.0e30f32;
    let mut num_in_current_row = 0usize;
    // Create rows.
    for collider in sip.snapshot.colliders.values() {
        num_in_current_row += 1;
        current_row.end = collider.position.y;
        if num_in_current_row >= min_collider_per_row {
            // We have the minimum number of collider to make a row.
            if current_row.end - current_row.start >= min_row_size {
                // We also have the minimun size.
                sip.snapshot.rows.push(current_row);

                // Prepare next row.
                current_row = SAPRow::default();
                current_row.start = collider.position.y;
                num_in_current_row = 0;
            }
        }
    }
    // Add non-full row.
    if num_in_current_row > 0 {
        sip.snapshot.rows.push(current_row);
    }
    // Last row's end should be very large.
    sip.snapshot.rows.last_mut().unwrap().end = 1.0e30f32;

    // Add colliders to overlapping rows.
    let mut i = 0u32;
    for collider in sip.snapshot.colliders.values() {
        let bottom = collider.position.y - collider.radius;
        let top = collider.position.y + collider.radius;
        let first_overlapping_row = sip.snapshot.rows.partition_point(|row| row.end < bottom);
        for row in &mut sip.snapshot.rows[first_overlapping_row..] {
            if row.start > top {
                break;
            }
            row.data.push(i);
        }

        i += 1;
    }

    // Sort each row on the x axis.
    for row in &mut sip.snapshot.rows {
        row.data.sort_unstable_by(|a, b| {
            sip.snapshot.colliders[*a as usize]
                .position
                .x
                .partial_cmp(&sip.snapshot.colliders[*b as usize].position.x)
                .unwrap_or(Ordering::Equal)
        });
    }

    // Find biggest radius for each row.
    for row in &mut sip.snapshot.rows {
        row.biggest_radius = row
            .data
            .iter()
            .fold(0.0, |acc, i| sip.snapshot.colliders[*i as usize].radius.max(acc));
    }

    sip
}
