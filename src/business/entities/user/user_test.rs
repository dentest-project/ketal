use uuid::Uuid;

impl super::User {
    pub(crate) fn id(&self) -> Uuid {
        self.id
    }
}

#[test]
fn defines_reset_password_code_and_updates_last_request_date() {
    use crate::business::{EntityBuilder, entities::user::UserBuilder};

    let mut user = UserBuilder::init().build();
    let before = std::time::SystemTime::now();

    user.define_reset_password_code("reset-code".to_owned());

    let last_request = user
        .last_reset_password_request
        .expect("last reset password request should be defined");
    let after = std::time::SystemTime::now();

    assert_eq!(user.reset_password_code(), Some("reset-code"));
    assert!(last_request >= before);
    assert!(last_request <= after);
}

#[test]
fn considers_reset_password_request_cooldown_expired_when_no_request_was_made() {
    use crate::business::{EntityBuilder, entities::user::UserBuilder};

    let user = UserBuilder::init().build();

    assert!(user.is_reset_password_request_cooldown_expired());
    assert_eq!(
        user.reset_password_request_remaining_cooldown_minutes(),
        None
    );
}

#[test]
fn considers_reset_password_request_cooldown_expired_after_two_hours() {
    use crate::business::{EntityBuilder, entities::user::UserBuilder};

    let mut user = UserBuilder::init().build();
    user.last_reset_password_request =
        Some(std::time::SystemTime::now() - std::time::Duration::from_secs(121 * 60));

    assert!(user.is_reset_password_request_cooldown_expired());
    assert_eq!(
        user.reset_password_request_remaining_cooldown_minutes(),
        None
    );
}

#[test]
fn exposes_remaining_reset_password_request_cooldown_minutes_when_not_expired() {
    use crate::business::{EntityBuilder, entities::user::UserBuilder};

    let mut user = UserBuilder::init().build();
    user.last_reset_password_request =
        Some(std::time::SystemTime::now() - std::time::Duration::from_secs(30 * 60));

    assert!(!user.is_reset_password_request_cooldown_expired());
    assert_eq!(
        user.reset_password_request_remaining_cooldown_minutes(),
        Some(90)
    );
}

#[test]
fn resets_password_and_consumes_reset_password_code() {
    use crate::business::{EntityBuilder, entities::user::UserBuilder};

    let mut user = UserBuilder::init()
        .with_password("old-password".to_owned())
        .build();
    user.define_reset_password_code("reset-password-code".to_owned());

    user.reset_password("new-password".to_owned());

    assert_eq!(user.password(), "new-password");
    assert_eq!(user.reset_password_code(), None);
}

#[test]
fn updates_personal_information_without_changing_an_omitted_password() {
    use crate::business::{EntityBuilder, entities::user::UserBuilder};

    let mut user = UserBuilder::init()
        .with_username("old-username".to_owned())
        .with_email("old-email@example.com".to_owned())
        .with_password("old-password".to_owned())
        .build();

    user.update_personal_information(
        "new-username".to_owned(),
        "new-email@example.com".to_owned(),
        None,
    );

    assert_eq!(user.username(), "new-username");
    assert_eq!(user.email(), "new-email@example.com");
    assert_eq!(user.password(), "old-password");
}

#[test]
fn updates_personal_information_with_a_new_password() {
    use crate::business::{EntityBuilder, entities::user::UserBuilder};

    let mut user = UserBuilder::init()
        .with_password("old-password".to_owned())
        .build();

    user.update_personal_information(
        "new-username".to_owned(),
        "new-email@example.com".to_owned(),
        Some("new-password".to_owned()),
    );

    assert_eq!(user.password(), "new-password");
}
