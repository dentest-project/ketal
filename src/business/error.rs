mod gateway_error;
mod invalid_credentials_error;
mod organization_already_exists_error;
mod password_decoder_error;
mod password_encoder_error;
mod reset_password_request_too_early_error;
mod token_generator_error;
mod unexpected_error;
mod user_already_exists_error;
mod user_not_found_error;

pub(crate) use gateway_error::GatewayError;
pub(crate) use invalid_credentials_error::InvalidCredentialsError;
pub(crate) use organization_already_exists_error::OrganizationAlreadyExistsError;
pub(crate) use password_decoder_error::PasswordDecoderError;
pub(crate) use password_encoder_error::PasswordEncoderError;
pub(crate) use reset_password_request_too_early_error::ResetPasswordRequestTooEarlyError;
pub(crate) use token_generator_error::TokenGeneratorError;
pub(crate) use unexpected_error::UnexpectedError;
pub(crate) use user_already_exists_error::UserAlreadyExistsError;
pub(crate) use user_not_found_error::UserNotFoundError;

macro_rules! use_case_error {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $variant:ident($inner:ty)
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, thiserror::Error, serde::Serialize)]
        #[serde(untagged)]
        $vis enum $name {
            $(
                #[error(transparent)]
                $variant(#[from] $inner),
            )+
        }

        impl jsonrpc_usecase::Error for $name {
            fn code(&self) -> i64 {
                match self {
                    $(Self::$variant(error) => <$inner as jsonrpc_usecase::Error>::code(error),)+
                }
            }

            fn message(&self) -> std::borrow::Cow<'static, str> {
                match self {
                    $(Self::$variant(error) => <$inner as jsonrpc_usecase::Error>::message(error),)+
                }
            }

            fn data(&self) -> serde_json::Value {
                match self {
                    $(Self::$variant(error) => <$inner as jsonrpc_usecase::Error>::data(error),)+
                }
            }
        }
    };
}

pub(crate) use use_case_error;
