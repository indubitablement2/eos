#![feature(test)]

pub mod collision;
pub mod generation;

use collision::*;
use crossbeam_channel::*;
use indexmap::IndexMap;
use rapier2d::na::{vector, Vector2};
use rapier2d::prelude::*;

pub struct PlayerID {
    pub id: u64,
}

pub struct System {}
impl System {
    const COLLISION_MEMBERSHIP: u32 = 0b0000000000000010;
    /// Radius of a small system.
    const SMALL: f32 = 32.0;
    const MEDIUM: f32 = System::SMALL * 1.5;
    const LARGE: f32 = System::SMALL * 2.0;
}

pub struct Player {}
impl Player {
    const COLLISION_MEMBERSHIP: u32 = 0b0000000000000001;
    const MIN_REALITY_BUBBLE_SIZE: f32 = 512.0;
}

pub struct Strategyscape {
    pub tick: u64,
    pub bound: AABB,

    pub collision_pipeline_bundle: CollisionPipelineBundle,
    pub query_pipeline_bundle: QueryPipelineBundle,
    pub body_set_bundle: BodySetBundle,
    pub collision_events_bundle: CollisionEventsBundle,

    pub systems: IndexMap<ColliderHandle, System>,
    pub players: IndexMap<ColliderHandle, Player>,
}
impl Strategyscape {
    /// Create a new Strategyscape with default parameters.
    pub fn new() -> Self {
        let (collision_events_bundle, channel_event_collector) = CollisionEventsBundle::new();

        Self {
            tick: 0,
            bound: AABB::from_half_extents(point![0.0, 0.0], vector![512.0, 512.0]),
            collision_pipeline_bundle: CollisionPipelineBundle::new(channel_event_collector),
            query_pipeline_bundle: QueryPipelineBundle::new(),
            body_set_bundle: BodySetBundle::new(),
            collision_events_bundle,
            systems: IndexMap::new(),
            players: IndexMap::new(),
        }
    }

    /// Get the position and radius of every system.
    pub fn get_systems(&self) -> Vec<(Vector2<f32>, f32)> {
        let mut systems = Vec::with_capacity(self.systems.len());

        for collider_handle in self.systems.keys() {
            if let Some(collider) = self.body_set_bundle.collider_set.get(*collider_handle) {
                if let Some(ball) = collider.shape().as_ball() {
                    systems.push((collider.translation().to_owned(), ball.radius));
                }
            }
        }

        systems
    }
}

pub struct StrategyscapeHandle {
    pub request_sender: Sender<Strategyscape>,
    pub result_receiver: Receiver<Strategyscape>,
}
impl StrategyscapeHandle {
    /// Create a Strategyscape with default parameters.
    pub fn new() -> Self {
        let (request_sender, request_receiver) = bounded(0);
        let (result_sender, result_receiver) = bounded(0);

        let runner = StrategyscapeRunner::new(request_receiver, result_sender);
        runner.spawn_loop();

        Self {
            request_sender,
            result_receiver,
        }
    }

    // pub fn add_system(&mut self, translation: Vector2<f32>) {
    //     let coll = ColliderBuilder::ball(64.0)
    //         .translation(translation)
    //         .collision_groups(InteractionGroups {
    //             memberships: System::COLLISION_MEMBERSHIP,
    //             filter: Player::COLLISION_MEMBERSHIP,
    //         })
    //         .active_events(ActiveEvents::INTERSECTION_EVENTS)
    //         .sensor(true)
    //         .build();

    //     let collider_handle = self.colliders.insert(coll);
    //     let system = System {};
    //     self.system.insert(collider_handle, system);
    // }

    // pub fn add_player(&mut self, translation: Vector2<f32>) {
    //     let coll = ColliderBuilder::ball(Player::MIN_REALITY_BUBBLE_SIZE)
    //         .translation(translation)
    //         .collision_groups(InteractionGroups {
    //             memberships: Player::COLLISION_MEMBERSHIP,
    //             filter: Player::COLLISION_MEMBERSHIP | System::COLLISION_MEMBERSHIP,
    //         })
    //         .active_events(ActiveEvents::INTERSECTION_EVENTS)
    //         .sensor(true)
    //         .build();

    //     let collider_handle = self.colliders.insert(coll);
    //     let player = Player {};
    //     self.player.insert(collider_handle, player);
    // }
}

pub struct StrategyscapeRunner {
    request_receiver: Receiver<Strategyscape>,
    result_sender: Sender<Strategyscape>,
}
impl StrategyscapeRunner {
    /// Make a new runner.
    fn new(request_receiver: Receiver<Strategyscape>, result_sender: Sender<Strategyscape>) -> Self {
        Self {
            request_receiver,
            result_sender,
        }
    }

    /// Start the runner thread.
    fn spawn_loop(self) {
        std::thread::spawn(move || {
            while let Ok(strategyscape) = self.request_receiver.recv() {
                if self.result_sender.send(main_loop(strategyscape)).is_err() {
                    break;
                }
            }
        });
    }
}

fn main_loop(mut strategyscape: Strategyscape) -> Strategyscape {
    strategyscape.tick += 1;

    // Update collisions.
    strategyscape
        .collision_pipeline_bundle
        .step(&mut strategyscape.body_set_bundle);
    strategyscape.query_pipeline_bundle.update(&mut strategyscape.body_set_bundle);

    strategyscape
}