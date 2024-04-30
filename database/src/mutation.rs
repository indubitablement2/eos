use super::*;

pub enum Mutation {
    Auth(auth_request::Mutation),
}
impl From<auth_request::Mutation> for Mutation {
    fn from(mutation: auth_request::Mutation) -> Self {
        Self::Auth(mutation)
    }
}

impl Database {
    pub fn push_mutation(&self, mutation: impl Into<mutation::Mutation>) {
        self.mutations
            .get_or_default()
            .borrow_mut()
            .push(mutation.into());
    }

    pub fn apply_mutation(&mut self, mutation: Mutation) {
        match mutation {
            Mutation::Auth(mutation) => {
                self.apply_auth_mutation(mutation);
            }
        }
    }
}
