use base64::prelude::*;
use ed25519_dalek::ed25519::signature::SignerMut;
use ed25519_dalek::{Signature, SigningKey, Verifier, VerifyingKey, pkcs8::DecodePrivateKey as _, pkcs8::DecodePublicKey as _};
use std::fmt::Debug;
use std::fs;
use std::marker::PhantomData;
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedPayload<T>
where
    T: Serialize + DeserializeOwned + Debug,
{
    payload: String,
    author: String,
    signature: String,
    #[serde(skip)]
    _marker: PhantomData<T>,
}

#[derive(Debug, thiserror::Error)]
pub enum SigningError {
    #[error("Couldn't find key at {0}")]
    PrivateKeyNotFound(PathBuf),
    #[error("Couldn't parse private key at {0} : {1}")]
    ParsingPrivateKeyFailed(PathBuf, String),
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Couldn't parse public key : {0}")]
    ParsingPublicKeyFailed(String),
}

pub fn sign<T>(
    payload: T,
    private_key: PathBuf,
    author: String,
) -> Result<SignedPayload<T>, SigningError>
where
    T: Serialize + DeserializeOwned + Debug,
{
    let serialized_payload = serde_json::json!(payload).to_string();
    let digest = &Sha256::digest(&serialized_payload)[..];

    let mut signing_key = read_pkcs8_der_public_key(private_key)?;
    let signature = BASE64_STANDARD.encode(signing_key.sign(digest).to_bytes());

    Ok(SignedPayload {
        payload: serialized_payload,
        author,
        signature,
        _marker: PhantomData,
    })
}

pub fn verify(payload: String, signature: String, public_key: &[u8]) -> Result<(), SigningError> {
    let verifying_key = VerifyingKey::from_public_key_der(public_key)
        .map_err(|e| SigningError::ParsingPublicKeyFailed(e.to_string()))?;

    let signature_bytes = BASE64_STANDARD
        .decode(&signature)
        .map_err(|_| SigningError::InvalidSignature)?;

    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| SigningError::InvalidSignature)?;

    let digest = Sha256::digest(&payload);

    verifying_key
        .verify(&digest, &signature)
        .map_err(|_| SigningError::InvalidSignature)
}

pub fn read_pkcs8_der_public_key(path: PathBuf) -> Result<SigningKey, SigningError> {
    let content =
        fs::read(path.clone()).map_err(|_| SigningError::PrivateKeyNotFound(path.clone()))?;

    SigningKey::from_pkcs8_der(&content)
        .map_err(|e| SigningError::ParsingPrivateKeyFailed(path, e.to_string()))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

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

        let private_key = PathBuf::from("./tests/private_key.der");

        let res = sign(payload, private_key, "John".to_string()).unwrap();

        let signature = res.signature;

        assert_eq!(signature, "1MS8Tr67yUxQK5pzFvn2a275HeFEodV36dVyckwgR8urr98AM73V90wdqCPJQ4TbABRYr7XZ3PLhu/Q3ol0XAQ==".to_string());
    }

    #[test]
    fn test_verify() {
        let payload = Foo {
            bar: "baz".to_string(),
        };

        let private_key = PathBuf::from("./tests/private_key.der");
        let public_key_path = PathBuf::from("./tests/public_key.der");
        let public_key = fs::read(public_key_path).unwrap();

        let res = sign(payload, private_key, "John".to_string()).unwrap();

        let serialized_payload = serde_json::json!(Foo {
            bar: "baz".to_string(),
        }).to_string();

        let res = verify(serialized_payload, res.signature, &public_key);
        assert!(res.is_ok());
    }


    #[test]
    fn test_verify_fails() {
        let payload = Foo {
            bar: "baz".to_string(),
        };

        let private_key = PathBuf::from("./tests/private_key.der");
        let public_key_path = PathBuf::from("./tests/public_key.der");
        let public_key = fs::read(public_key_path).unwrap();

        let res = sign(payload, private_key, "John".to_string()).unwrap();

        let serialized_payload = serde_json::json!(Foo {
            bar: "boz".to_string(),
        }).to_string();

        let res = verify(serialized_payload, res.signature, &public_key);
        assert!(res.is_err());
    }
}
