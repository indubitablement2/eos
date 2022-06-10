// use crate::*;

// #[derive(Bundle)]
// struct FleetBundle {
//     fleet_id: FleetIdComp,
//     faction_id: FactionIdComp,
//     name: Name,
//     in_system: InSystem,
//     position: Position,
//     velocity: Velocity,
//     wish_position: WishPosition,
//     orbit: OrbitComp,
//     idle_counter: IdleCounter,
//     // fleet_inner: FleetInner,
//     // fleet_ai: FleetAi,
// }

// pub struct FleetBundleBuilder {
    
// }
// impl FleetBundleBuilder {
//     pub fn new() -> Self {
//         Self {

//         }
//     }
// }

// pub struct FleetBuilder {
//     pub faction_id: FactionId,
//     pub name: String,
//     pub position: Vec2,
//     pub velocity: Vec2,
//     pub wish_position: WishPosition,
// }
// impl FleetBuilder {
//     pub fn new(
//         faction_id: FactionId,
//         name: String,
//         position: Vec2,
//         fleet_ai: FleetAi,
//         fleet_composition: FleetComposition,
//     ) -> Self {
//         Self {
//             faction_id,
//             name,
//             in_system: None,
//             position,
//             velocity: Vec2::ZERO,
//             wish_position: Default::default(),
//             fleet_ai,
//             fleet_composition,
//         }
//     }

//     pub fn with_in_system(mut self, system_id: SystemId) -> Self {
//         self.in_system = Some(system_id);
//         self
//     }

//     pub fn with_velocity(mut self, velocity: Vec2) -> Self {
//         self.velocity = velocity;
//         self
//     }

//     pub fn with_wish_position(mut self, wish_position: WishPosition) -> Self {
//         self.wish_position = wish_position;
//         self
//     }

//     pub fn build_ai(self) -> FleetId {
//         let fleet_id = AI_FLEET_ID_DISPENSER.next();
//         FLEET_QUEUE.push((fleet_id, self));
//         fleet_id
//     }

//     pub fn build_client(self, client_id: ClientId) -> FleetId {
//         let fleet_id = client_id.to_fleet_id();
//         FLEET_QUEUE.push((fleet_id, self));
//         fleet_id
//     }
// }

// #[derive(Serialize, Deserialize)]
// pub struct FleetSave {
//     pub fleet_id: FleetId,
//     pub faction_id: FactionId,
//     pub name: String,
//     pub position: Vec2,
//     pub fleet_composition: FleetComposition,
//     pub fleet_ai: FleetAi,
// }
