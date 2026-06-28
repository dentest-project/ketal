mod gateway_error;
mod password_encoder_error;
mod user_already_exists_error;

pub(crate) use gateway_error::GatewayError;
pub(crate) use password_encoder_error::PasswordEncoderError;
pub(crate) use user_already_exists_error::UserAlreadyExistsError;

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
