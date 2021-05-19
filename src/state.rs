use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;
use std::str;

pub type HmacSha25 = Hmac<Sha256>;

#[derive(Clone)]
pub struct State {
    redis_url: String,
    hmac: String,
}

impl State {
    pub fn new_connection(&self) -> Result<redis::Client, redis::RedisError> {
        redis::Client::open(self.redis_url.clone())
    }

    pub fn new(redis_pass: String, redis_host: String, hmac: String) -> State {
        State {
            redis_url: format!("redis://:{}@{}", redis_pass, redis_host),
            hmac,
        }
    }

    pub fn verify_hmac(&self, message: String, signature: &[u8]) -> bool {
        let mut mac = HmacSha25::new_varkey(self.hmac.as_bytes()).expect("HMAC key error");
        mac.update(message.as_bytes());

        let decoded_message: Vec<u8> = match hex::decode(signature) {
            Ok(msg) => msg,
            Err(e) => {
                log::warn!("{}", e);
                return false;
            }
        };

        match mac.verify(&decoded_message[..]) {
            Ok(()) => true,
            Err(e) => {
                log::warn!("{}", e);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_hmac() {
        let state = State::new("oof".to_owned(), "oof".to_owned(), "0f0".to_owned());

        let mut mac = HmacSha25::new_varkey(b"0f0").expect("HMAC key error");
        mac.update(b"oof");
        let result = mac.finalize();
        let bytes = result.into_bytes();

        assert!(state.verify_hmac("oof".to_string(), &bytes));
        assert!(!state.verify_hmac("of".to_string(), &bytes));
    }
}
