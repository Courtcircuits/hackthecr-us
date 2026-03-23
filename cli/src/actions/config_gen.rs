use base64::prelude::*;
use rcgen::{KeyPair, PKCS_ED25519};

pub struct Ed25519Pem {
    pub certificate: String,
    pub private_key: String,
}

pub fn generate_ed25519_pem() -> Result<Ed25519Pem, rcgen::Error> {
    let key_pair = KeyPair::generate_for(&PKCS_ED25519)?;

    let certificate = BASE64_STANDARD.encode(key_pair.public_key_pem());
    let private_key = BASE64_STANDARD.encode(key_pair.serialize_pem());

    Ok(Ed25519Pem {
        certificate,
        private_key,
    })
}
