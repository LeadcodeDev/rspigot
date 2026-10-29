use rsa::pkcs8::EncodePublicKey;
use rsa::rand_core::OsRng;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use sha1::{Digest, Sha1};

const RSA_BITS: usize = 1024;

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("rsa: {0}")]
    Rsa(#[from] rsa::Error),
    #[error("public key encoding: {0}")]
    PublicKeyEncoding(#[from] rsa::pkcs8::spki::Error),
}

pub struct ServerKey {
    private: RsaPrivateKey,
    public_der: Vec<u8>,
}

impl ServerKey {
    pub fn generate() -> Result<Self, CryptoError> {
        let private = RsaPrivateKey::new(&mut OsRng, RSA_BITS)?;
        let public_der = private.to_public_key().to_public_key_der()?.into_vec();
        Ok(Self {
            private,
            public_der,
        })
    }

    pub fn public_der(&self) -> &[u8] {
        &self.public_der
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        Ok(self.private.decrypt(Pkcs1v15Encrypt, data)?)
    }
}

pub fn server_hash(server_id: &str, shared_secret: &[u8], public_key_der: &[u8]) -> String {
    let mut hasher = Sha1::new();
    let latin1: Vec<u8> = server_id
        .chars()
        .map(|c| u8::try_from(u32::from(c)).unwrap_or(b'?'))
        .collect();
    hasher.update(&latin1);
    hasher.update(shared_secret);
    hasher.update(public_key_der);
    let mut digest: [u8; 20] = hasher.finalize().into();
    let negative = digest[0] & 0x80 != 0;
    if negative {
        negate_twos_complement(&mut digest);
    }
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    let magnitude = hex.trim_start_matches('0');
    match (negative, magnitude.is_empty()) {
        (_, true) => String::from("0"),
        (true, false) => format!("-{magnitude}"),
        (false, false) => magnitude.to_owned(),
    }
}

fn negate_twos_complement(bytes: &mut [u8]) {
    let mut carry = true;
    for byte in bytes.iter_mut().rev() {
        let (value, overflow) = (!*byte).overflowing_add(u8::from(carry));
        *byte = value;
        carry = overflow;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::pkcs8::DecodePublicKey;
    use rsa::rand_core::OsRng;
    use rsa::{Pkcs1v15Encrypt, RsaPublicKey};

    #[test]
    fn server_hash_matches_known_vectors() {
        assert_eq!(
            server_hash("Notch", &[], &[]),
            "4ed1f46bbe04bc756bcb17c0c7ce3e4632f06a48"
        );
        assert_eq!(
            server_hash("jeb_", &[], &[]),
            "-7c9d5b0044c130109a5d7b5fb5c317c02b4e28c1"
        );
        assert_eq!(
            server_hash("simon", &[], &[]),
            "88e16a1019277b15d58faf0541e11910eb756f6"
        );
    }

    #[test]
    fn server_hash_digests_id_secret_and_key_in_order() {
        assert_eq!(
            server_hash("No", b"tc", b"h"),
            server_hash("Notch", &[], &[])
        );
    }

    #[test]
    fn server_hash_encodes_server_id_as_latin1() {
        assert_eq!(server_hash("é", &[], &[]), server_hash("", &[0xE9], &[]));
        assert_eq!(server_hash("€", &[], &[]), server_hash("?", &[], &[]));
    }

    #[test]
    fn public_key_is_x509_rsa_1024() {
        let key = ServerKey::generate().unwrap();
        let der = key.public_der();
        assert_eq!(der.len(), 162);
        assert_eq!(
            &der[..22],
            &[
                0x30, 0x81, 0x9F, 0x30, 0x0D, 0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01,
                0x01, 0x01, 0x05, 0x00, 0x03, 0x81, 0x8D, 0x00
            ]
        );
    }

    #[test]
    fn decrypts_what_the_public_key_encrypts() {
        let key = ServerKey::generate().unwrap();
        let public = RsaPublicKey::from_public_key_der(key.public_der()).unwrap();
        let secret = [7u8; 16];
        let encrypted = public
            .encrypt(&mut OsRng, Pkcs1v15Encrypt, &secret)
            .unwrap();
        assert_eq!(encrypted.len(), 128);
        assert_eq!(key.decrypt(&encrypted).unwrap(), secret.to_vec());
    }

    #[test]
    fn rejects_garbage_ciphertext() {
        let key = ServerKey::generate().unwrap();
        assert!(key.decrypt(&[0u8; 128]).is_err());
        assert!(key.decrypt(&[1, 2, 3]).is_err());
    }
}
