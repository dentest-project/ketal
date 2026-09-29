#![cfg(test)]

use crate::business::entities::user::{
    User,
    user_gateway::{GatewayFuture, UserGateway},
};
use crate::business::error::GatewayError;
use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryUserGateway {
    users: Mutex<HashMap<Uuid, User>>,
}

impl UserGateway for InMemoryUserGateway {
    fn find_one_by_id(&self, id: Uuid) -> GatewayFuture<'_, Option<User>> {
        Box::pin(async move {
            Ok(self
                .users
                .lock()
                .map_err(|error| GatewayError::new(error.to_string()))?
                .get(&id)
                .cloned())
        })
    }

    fn save<'a>(&'a self, user: &'a User) -> GatewayFuture<'a, ()> {
        let user = user.clone();

        Box::pin(async move {
            self.users
                .lock()
                .map_err(|error| GatewayError::new(error.to_string()))?
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
                .map_err(|error| GatewayError::new(error.to_string()))?
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
                .map_err(|error| GatewayError::new(error.to_string()))?
                .values()
                .find(|user| {
                    user.email.to_lowercase() == email || user.username.to_lowercase() == username
                })
                .cloned();

            Ok(user)
        })
    }

    fn find_one_by_email_or_username_excluding_user<'a>(
        &'a self,
        email: &'a str,
        username: &'a str,
        excluded_user: &'a User,
    ) -> GatewayFuture<'a, Option<User>> {
        let email = email.to_lowercase();
        let username = username.to_lowercase();
        let excluded_user_id = excluded_user.id;

        Box::pin(async move {
            let user = self
                .users
                .lock()
                .map_err(|error| GatewayError::new(error.to_string()))?
                .values()
                .find(|user| {
                    user.id != excluded_user_id
                        && (user.email.to_lowercase() == email
                            || user.username.to_lowercase() == username)
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
                .map_err(|error| GatewayError::new(error.to_string()))?
                .values()
                .find(|user| user.reset_password_code.as_deref() == Some(reset_password_code))
                .cloned();

            Ok(user)
        })
    }
}
