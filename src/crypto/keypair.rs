use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    pub public_key: PublicKey,
    secret_key: SecretKey,
}

impl KeyPair {
    
    pub fn generate() -> Self {
        let mut csprng = OsRng {};
        let keypair = Keypair::generate(&mut csprng);
        
        Self {
            public_key: keypair.public,
            secret_key: keypair.secret,
        }
    }

    
    pub fn from_bytes(public_bytes: &[u8], secret_bytes: &[u8]) -> crate::utils::Result<Self> {
        let public_key = PublicKey::from_bytes(public_bytes)
            .map_err(|e| crate::utils::Error::CryptoError(e.to_string()))?;
        let secret_key = SecretKey::from_bytes(secret_bytes)
            .map_err(|e| crate::utils::Error::CryptoError(e.to_string()))?;

        Ok(Self {
            public_key,
            secret_key,
        })
    }

    
    pub fn sign(&self, message: &[u8]) -> Signature {
        let keypair = Keypair {
            public: self.public_key,
            secret: self.secret_key.clone(),
        };
        keypair.sign(message)
    }

    
    pub fn to_hex_strings(&self) -> (String, String) {
        let public_hex = hex::encode(self.public_key.as_bytes());
        let secret_hex = hex::encode(self.secret_key.as_bytes());
        (public_hex, secret_hex)
    }

    
    pub fn from_hex_strings(public_hex: &str, secret_hex: &str) -> crate::utils::Result<Self> {
        let public_bytes = hex::decode(public_hex)
            .map_err(|e| crate::utils::Error::SerializationError(e.to_string()))?;
        let secret_bytes = hex::decode(secret_hex)
            .map_err(|e| crate::utils::Error::SerializationError(e.to_string()))?;
        
        Self::from_bytes(&public_bytes, &secret_bytes)
    }
}

impl fmt::Display for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeyPair(public: {})", hex::encode(self.public_key.as_bytes()))
    }
}

