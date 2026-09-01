use crate::{
    business::{
        RequestContext,
        error::{UnexpectedError, use_case_error},
        usecases::outputs::user::UserDetailedOutput,
    },
    infrastructure::guards::AuthenticatedUserGuard,
};
use jsonrpc_usecase::{UseCase, current_context};

#[derive(Default)]
pub struct WhoAmI;

use_case_error! {
    pub enum WhoAmIError {
        Unexpected(UnexpectedError),
    }
}

#[UseCase(guards = [AuthenticatedUserGuard])]
impl WhoAmI {
    async fn execute(&self, _input: ()) -> Result<UserDetailedOutput, WhoAmIError> {
        let context = current_context::<RequestContext>().ok_or(UnexpectedError)?;
        let user = context.user.as_ref().ok_or(UnexpectedError)?;

        Ok(user.into())
    }
}

#[cfg(test)]
mod tests {
    use super::{WhoAmI, WhoAmIError};
    use jsonrpc_usecase::UseCaseExecutionError;

    #[tokio::test]
    async fn returns_unexpected_error_when_request_context_is_missing() {
        let result = WhoAmI.execute(()).await;

        assert!(matches!(
            result,
            Err(UseCaseExecutionError::Execution(WhoAmIError::Unexpected(_)))
        ));
    }
}
