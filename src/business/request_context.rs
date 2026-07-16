use crate::business::entities::user::User;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RequestContext {
    pub user: Option<User>,
}
