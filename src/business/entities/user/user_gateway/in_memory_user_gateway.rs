#![cfg(test)]

use crate::business::entities::user::{
    User,
    user_gateway::{GatewayFuture, UserGateway},
};
use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryUserGateway {
    users: Mutex<HashMap<Uuid, User>>,
}

impl UserGateway for InMemoryUserGateway {
    fn save<'a>(&'a self, user: &'a User) -> GatewayFuture<'a, ()> {
        let user = user.clone();

        Box::pin(async move {
            self.users
                .lock()
                .expect("user gateway mutex should not be poisoned")
                .insert(user.id, user);

            Ok(())
        })
    }

    fn find_one_by_username<'a>(&'a self, username: &'a str) -> GatewayFuture<'a, Option<User>> {
        let username = username.to_lowercase();

        Box::pin(async move {
            let user = self
                .users
                .lock()
                .expect("user gateway mutex should not be poisoned")
                .values()
                .find(|user| user.username.to_lowercase() == username)
                .cloned();

            Ok(user)
        })
    }

    fn find_one_by_email_or_username<'a>(
        &'a self,
        email: &'a str,
        username: &'a str,
    ) -> GatewayFuture<'a, Option<User>> {
        let email = email.to_lowercase();
        let username = username.to_lowercase();

        Box::pin(async move {
            let user = self
                .users
                .lock()
                .expect("user gateway mutex should not be poisoned")
                .values()
                .find(|user| {
                    user.email.to_lowercase() == email || user.username.to_lowercase() == username
                })
                .cloned();

            Ok(user)
        })
    }

    fn find_one_by_reset_password_code<'a>(
        &'a self,
        reset_password_code: &'a str,
    ) -> GatewayFuture<'a, Option<User>> {
        Box::pin(async move {
            let user = self
                .users
                .lock()
                .expect("user gateway mutex should not be poisoned")
                .values()
                .find(|user| user.reset_password_code.as_deref() == Some(reset_password_code))
                .cloned();

            Ok(user)
        })
    }
}
