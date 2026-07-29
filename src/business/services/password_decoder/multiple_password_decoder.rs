use super::{PasswordDecoder, PasswordDecoderResult};
use std::sync::Arc;

pub struct MultiplePasswordDecoder {
    decoders: Vec<Arc<dyn PasswordDecoder>>,
}

impl MultiplePasswordDecoder {
    pub fn new(decoders: Vec<Arc<dyn PasswordDecoder>>) -> Self {
        Self { decoders }
    }
}

impl PasswordDecoder for MultiplePasswordDecoder {
    fn matches(&self, password: &str, encoded_password: &str) -> PasswordDecoderResult<bool> {
        for decoder in &self.decoders {
            if decoder.matches(password, encoded_password)? {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::MultiplePasswordDecoder;
    use crate::business::services::{PasswordDecoder, password_decoder::PasswordDecoderDouble};
    use std::sync::Arc;

    #[test]
    fn tries_decoders_until_one_matches() {
        let first_decoder = Arc::new(PasswordDecoderDouble::matching(false));
        let second_decoder = Arc::new(PasswordDecoderDouble::matching(true));
        let decoder =
            MultiplePasswordDecoder::new(vec![first_decoder.clone(), second_decoder.clone()]);

        assert!(
            decoder
                .matches("secret123", "hashed-password")
                .expect("password should be decoded")
        );
        assert_eq!(
            first_decoder.received_matches(),
            vec![("secret123".to_owned(), "hashed-password".to_owned())]
        );
        assert_eq!(
            second_decoder.received_matches(),
            vec![("secret123".to_owned(), "hashed-password".to_owned())]
        );
    }

    #[test]
    fn stops_after_matching_decoder() {
        let first_decoder = Arc::new(PasswordDecoderDouble::matching(true));
        let second_decoder = Arc::new(PasswordDecoderDouble::matching(true));
        let decoder =
            MultiplePasswordDecoder::new(vec![first_decoder.clone(), second_decoder.clone()]);

        assert!(
            decoder
                .matches("secret123", "hashed-password")
                .expect("password should be decoded")
        );
        assert_eq!(
            first_decoder.received_matches(),
            vec![("secret123".to_owned(), "hashed-password".to_owned())]
        );
        assert!(second_decoder.received_matches().is_empty());
    }

    #[test]
    fn returns_false_when_no_decoder_matches() {
        let first_decoder = Arc::new(PasswordDecoderDouble::matching(false));
        let second_decoder = Arc::new(PasswordDecoderDouble::matching(false));
        let decoder = MultiplePasswordDecoder::new(vec![first_decoder, second_decoder]);

        assert!(
            !decoder
                .matches("secret123", "hashed-password")
                .expect("password should be decoded")
        );
    }
}
