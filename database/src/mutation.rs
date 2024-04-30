use super::*;

pub enum Mutation {
    Auth(auth_request::Mutation),
}

impl Database {
    pub fn apply_mutation(&mut self, mutation: Mutation) {
        match mutation {
            Mutation::Auth(mutation) => {
                self.apply_auth_mutation(mutation);
            }
        }
    }
}
