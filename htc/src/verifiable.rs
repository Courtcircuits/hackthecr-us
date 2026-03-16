use base64::prelude::*;
use ed25519_dalek::ed25519::signature::SignerMut;
use ed25519_dalek::{Signature, SigningKey, Verifier, VerifyingKey, pkcs8::DecodePrivateKey as _, pkcs8::spki::DecodePublicKey as _};
use utoipa::ToSchema;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct SignedPayload<T>
where
    T: Serialize + DeserializeOwned + Debug,
{
    pub payload: T,
    pub author: String,
    pub signature: String,
    #[serde(skip)]
    _marker: PhantomData<T>,
}

#[derive(Debug, thiserror::Error)]
pub enum SigningError {
    #[error("Couldn't find key at {0}")]
    PrivateKeyNotFound(PathBuf),
    #[error("Couldn't parse private key : {0}")]
    ParsingPrivateKeyFailed(String),
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Couldn't parse public key : {0}")]
    ParsingPublicKeyFailed(String),
    #[error("Invalid base64")]
    InvalidBASE64
}

impl <T> SignedPayload<T>
where
    T: Serialize + DeserializeOwned + Debug + Clone
{
    pub fn sign(payload: T, private_key: &str, author: &str) -> Result<SignedPayload<T>, SigningError> {
        crate::verifiable::sign(payload, private_key, author.to_string())
    }

    pub fn verify(&self, public_key: &str) -> Result<&T, SigningError> {
        let serialized_payload = serde_json::json!(self.payload).to_string();
        verify(serialized_payload, &self.signature, public_key)?;
        Ok(&self.payload)
    }
}

pub fn sign<T>(
    payload: T,
    private_key: &str,
    author: String,
) -> Result<SignedPayload<T>, SigningError>
where
    T: Serialize + DeserializeOwned + Debug,
{
    let serialized_payload = serde_json::json!(payload).to_string();
    let digest = &Sha256::digest(&serialized_payload)[..];

    let private_key = BASE64_STANDARD
        .decode(private_key)
        .map_err(|_| SigningError::InvalidBASE64)?;
    let private_key = str::from_utf8(&private_key)
        .map_err(|_| SigningError::InvalidBASE64)?;
    let mut signing_key = read_pkcs8_pem_private_key(private_key)?;
    let signature = BASE64_STANDARD.encode(signing_key.sign(digest).to_bytes());

    Ok(SignedPayload {
        payload,
        author,
        signature,
        _marker: PhantomData,
    })
}

pub fn verify(payload: String, signature: &str, public_key: &str) -> Result<(), SigningError> {
    let public_key = BASE64_STANDARD
        .decode(public_key)
        .map_err(|_| SigningError::InvalidBASE64)?;
    let public_key = str::from_utf8(&public_key)
        .map_err(|_| SigningError::InvalidBASE64)?;
    let verifying_key = VerifyingKey::from_public_key_pem(public_key)
        .map_err(|e| SigningError::ParsingPublicKeyFailed(e.to_string()))?;

    let signature_bytes = BASE64_STANDARD
        .decode(signature)
        .map_err(|_| SigningError::InvalidSignature)?;

    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| SigningError::InvalidSignature)?;

    let digest = Sha256::digest(&payload);

    verifying_key
        .verify(&digest, &signature)
        .map_err(|_| SigningError::InvalidSignature)
}

pub fn read_pkcs8_pem_private_key(content: &str) -> Result<SigningKey, SigningError> {
    SigningKey::from_pkcs8_pem(content)
        .map_err(|e| SigningError::ParsingPrivateKeyFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use std::{fs};

    use serde::{Deserialize, Serialize};

    use crate::verifiable::{sign, verify};

    #[derive(Serialize, Deserialize, Debug)]
    struct Foo {
        bar: String,
    }

    #[test]
    fn test_sign() {
        let payload = Foo {
            bar: "baz".to_string(),
        };

        let private_key = include_str!("../tests/private_key.pem");

        let res = sign(payload, private_key, "John".to_string());
        assert!(res.is_ok());
    }

    #[test]
    fn test_verify() {
        let payload = Foo {
            bar: "baz".to_string(),
        };

        let private_key = include_str!("../tests/private_key.pem");
        let public_key = fs::read_to_string("./tests/public_key.pem").unwrap();

        let res = sign(payload, private_key, "John".to_string()).unwrap();

        let serialized_payload = serde_json::json!(Foo {
            bar: "baz".to_string(),
        }).to_string();

        let res = verify(serialized_payload, &res.signature, &public_key);
        assert!(res.is_ok());
    }

    #[test]
    fn test_verify_fails() {
        let payload = Foo {
            bar: "baz".to_string(),
        };

        let private_key = include_str!("../tests/private_key.pem");
        let public_key = fs::read_to_string("./tests/public_key.pem").unwrap();

        let res = sign(payload, private_key, "John".to_string()).unwrap();

        let serialized_payload = serde_json::json!(Foo {
            bar: "boz".to_string(),
        }).to_string();

        let res = verify(serialized_payload, &res.signature, &public_key);
        assert!(res.is_err());
    }
}
