#[derive(Debug, Clone, Copy)]
pub enum FleetAi {
    /// Fleet is controlled by a client.
    /// Actions will be taken from the client.
    // TODO: Fallback to what when client is not connected?
    ClientControl,
    Idle,
}
