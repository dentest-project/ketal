use crate::business::RequestContext;
use jsonrpc_usecase::{Guard, GuardContext};

#[derive(Default)]
pub struct AuthenticatedUserGuard;

impl Guard for AuthenticatedUserGuard {
    fn can_proceed(&self, context: &GuardContext) -> bool {
        context
            .get_context::<RequestContext>()
            .is_some_and(|context| context.user.is_some())
    }
}
