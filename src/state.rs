use super::invoice::InvoiceCommands;
use async_std::sync::{Arc, Mutex};
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;

#[derive(Clone)]
pub struct State<T: InvoiceCommands + std::clone::Clone> {
    pub db: Arc<Mutex<T>>,
    pub hmac: String,
}

impl<T: InvoiceCommands + std::clone::Clone> State<T> {
    pub fn verify_hmac(&self, data: String, sig: String) -> bool {
        let mut mac = HmacSha25::new_varkey(self.hmac.as_bytes()).expect("HMAC key error");
        mac.update(data.as_bytes());

        let decoded_message: Vec<u8> = match hex::decode(sig) {
            Ok(msg) => msg,
            Err(e) => {
                log::warn!("{}", e);
                return false;
            }
        };

        log::trace!("{:?}", decoded_message);

        log::trace!("{}", hex::encode(mac.clone().finalize().into_bytes()));

        match mac.verify(&decoded_message.as_slice()) {
            Ok(()) => true,
            Err(e) => {
                log::warn!("{}", e);
                false
            }
        }
    }
}

pub type HmacSha25 = Hmac<Sha256>;
