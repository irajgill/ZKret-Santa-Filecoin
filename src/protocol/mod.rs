use crate::crypto::{KeyPair, ZKSystem};
use crate::filecoin::FilecoinStorage;
use crate::utils::{Result, Error};

pub struct SecretSantaProtocol {
    pub storage: FilecoinStorage,
    pub zk_system: ZKSystem,
}

impl SecretSantaProtocol {
    pub fn new() -> Self {
        SecretSantaProtocol {
            storage: FilecoinStorage::new(),
            zk_system: ZKSystem::new(),
        }
    }

    pub fn register(&self, keypair: &KeyPair) -> Result<()> {
        let public_key = keypair.public.to_bytes().to_vec();
        let signature = keypair.sign(&public_key);
        let proof = self.zk_system.generate_proof(&signature, &public_key);
        let data = serde_json::to_vec(&(public_key, proof))
            .map_err(|e| Error::Serialization(e.to_string()))?;
        self.storage.store_data(&data)?;
        Ok(())
    }

    pub fn verify_registration(&self, _cid: &str) -> Result<bool> {
        
        Ok(true)
    }
}
