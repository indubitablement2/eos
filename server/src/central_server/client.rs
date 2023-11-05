use super::*;

// TODO: Store current battlesscape
// TODO: Notify instance when client disconnects
pub struct Client {
    ships: AHashSet<()>,
    password: Option<String>,
}
impl Client {
    // TODO: Handle new client (add fleet).
    pub fn new_password(password: String) -> Self {
        Self {
            ships: Default::default(),
            password: Some(password),
        }
    }

    pub fn verify_password(&self, other: &str) -> bool {
        self.password.as_deref() == Some(other)
    }
}
