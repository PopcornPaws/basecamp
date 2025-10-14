use hmac::digest::FixedOutput;
use hmac::{Hmac, Mac};

pub mod sha256 {
    use super::{FixedOutput, Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    /// Signs the provided message with the given HMAC key.
    ///
    /// # Panics
    ///
    /// In theory, this function never panics, as an HMAC `key` can have arbitrary size.
    pub fn sign<K: AsRef<[u8]>, M: AsRef<[u8]>>(key: K, message: M) -> [u8; 32] {
        let mut mac =
            HmacSha256::new_from_slice(key.as_ref()).expect("HMAC works with keys of all sizes");
        mac.update(message.as_ref());
        let mut output = [0u8; 32];
        mac.finalize_into((&mut output).into());
        output
    }

    /// Verifies the provided signature against the message and HMAC key.
    ///
    /// # Panics
    ///
    /// In theory, this function never panics, as an HMAC `key` can have arbitrary size.
    pub fn verify<K: AsRef<[u8]>, M: AsRef<[u8]>, S: AsRef<[u8]>>(
        key: K,
        message: M,
        signature: S,
    ) -> bool {
        let mut mac =
            HmacSha256::new_from_slice(key.as_ref()).expect("HMAC works with keys of all sizes");
        mac.update(message.as_ref());
        mac.verify_slice(signature.as_ref()).is_ok()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hmac_sha256() {
        let key = b"my very secret key";
        let message = b"hello hmac";

        let signature = sha256::sign(key, message);
        assert!(sha256::verify(key, message, signature));
    }
}
