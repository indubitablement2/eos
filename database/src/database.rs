use super::*;
use std::{num::NonZeroU64, sync::atomic::AtomicU64};

#[derive(Serialize, Deserialize)]
pub struct Database {
    next_client_id: AtomicU64,
}
impl Database {
    pub fn _load() {
        todo!()
    }

    pub fn client_login() -> Option<ClientId> {
        let client_id = ClientId(
            NonZeroU64::new(
                db().next_client_id
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            )
            .unwrap(),
        );

        Some(client_id)
    }

    /// Called when Client is dropped.
    pub fn _client_disconnected(client_id: ClientId) {}
}

static mut DATABASE: Option<Database> = None;
fn db() -> &'static Database {
    unsafe { DATABASE.as_ref().unwrap_unchecked() }
}
