#![cfg(test)]

use super::ResetPasswordCodeGenerator;
use std::sync::Mutex;

pub(crate) struct ResetPasswordCodeGeneratorDouble {
    code: String,
    calls: Mutex<usize>,
}

impl ResetPasswordCodeGeneratorDouble {
    pub(crate) fn generating(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            calls: Mutex::new(0),
        }
    }

    pub(crate) fn calls(&self) -> usize {
        *self
            .calls
            .lock()
            .expect("reset password code generator calls mutex should not be poisoned")
    }
}

impl ResetPasswordCodeGenerator for ResetPasswordCodeGeneratorDouble {
    fn generate(&self) -> String {
        *self
            .calls
            .lock()
            .expect("reset password code generator calls mutex should not be poisoned") += 1;

        self.code.clone()
    }
}
