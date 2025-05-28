use crate::crypto::{KeyPair, ZKProof, ZKProofSystem};
use crate::filecoin::{FilecoinStorage, RecordType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Phase {
    Setup,
    Enter,
    Choice,
    Reveal,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterTransaction {
    pub public_key: Vec<u8>,
    pub zk_proof: ZKProof,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceTransaction {
    pub chosen_public_key: Vec<u8>,
    pub chooser_dh_public_key: Vec<u8>,
    pub zk_proof: ZKProof,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevealTransaction {
    pub public_key: Vec<u8>,
    pub encrypted_identity: Vec<u8>,
    pub dh_public_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub timestamp: u64,
}

pub struct SecretSantaProtocol {
    storage: FilecoinStorage,
    zk_system: ZKProofSystem,
    current_phase: Phase,
    participants: HashMap<Vec<u8>, ParticipantState>,
}

#[derive(Debug, Clone)]
struct ParticipantState {
    public_key: Vec<u8>,
    has_entered: bool,
    has_chosen: bool,
    chosen_by: Option<Vec<u8>>,
    has_revealed: bool,
}

impl SecretSantaProtocol {
    /// Initialize a new Secret Santa protocol instance
    pub async fn new(storage: FilecoinStorage) -> crate::utils::Result<Self> {
        let zk_system = ZKProofSystem::new()?;
        
        Ok(Self {
            storage,
            zk_system,
            current_phase: Phase::Setup,
            participants: HashMap::new(),
        })
    }

    /// Execute ENTER phase - participant registers their public key
    pub async fn enter_phase(&mut self, keypair: &KeyPair) -> crate::utils::Result<()> {
        if !matches!(self.current_phase, Phase::Setup | Phase::Enter) {
            return Err(crate::utils::Error::ProtocolError(
                "ENTER phase not available in current state".to_string()
            ));
        }

        // Generate zero-knowledge proof for ENTER phase
        let zk_proof = self.zk_system.prove_enter_phase(
            keypair.public_key.as_bytes(),
            keypair.secret_key.as_bytes(),
        )?;

        // Create ENTER transaction
        let enter_tx = EnterTransaction {
            public_key: keypair.public_key.as_bytes().to_vec(),
            zk_proof,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Serialize and store transaction
        let tx_data = bincode::serialize(&enter_tx)
            .map_err(|e| crate::utils::Error::SerializationError(e.to_string()))?;

        let _record = self.storage.store_data(tx_data, RecordType::EnterTransaction).await?;

        // Update participant state
        let participant_state = ParticipantState {
            public_key: keypair.public_key.as_bytes().to_vec(),
            has_entered: true,
            has_chosen: false,
            chosen_by: None,
            has_revealed: false,
        };

        self.participants.insert(keypair.public_key.as_bytes().to_vec(), participant_state);
        self.current_phase = Phase::Enter;

        Ok(())
    }

    /// Execute CHOICE phase - participant chooses another participant
    pub async fn choice_phase(
        &mut self,
        chooser_keypair: &KeyPair,
        chosen_public_key: &[u8],
        dh_keypair: &crate::crypto::DHKeyExchange,
    ) -> crate::utils::Result<()> {
        if !matches!(self.current_phase, Phase::Enter | Phase::Choice) {
            return Err(crate::utils::Error::ProtocolError(
                "CHOICE phase not available in current state".to_string()
            ));
        }

        // Verify chooser has completed ENTER phase
        let chooser_pk = chooser_keypair.public_key.as_bytes();
        if !self.participants.get(chooser_pk)
            .map(|p| p.has_entered)
            .unwrap_or(false) {
            return Err(crate::utils::Error::ProtocolError(
                "Must complete ENTER phase before CHOICE phase".to_string()
            ));
        }

        // Verify chosen participant exists and hasn't been chosen
        let all_public_keys = self.storage.get_all_public_keys().await?;
        if !all_public_keys.contains(&chosen_public_key.to_vec()) {
            return Err(crate::utils::Error::ProtocolError(
                "Chosen participant not found".to_string()
            ));
        }

        // Generate zero-knowledge proof for CHOICE phase
        let zk_proof = self.zk_system.prove_choice_phase(
            chooser_pk,
            chosen_public_key,
            chooser_keypair.secret_key.as_bytes(),
        )?;

        // Create CHOICE transaction
        let choice_tx = ChoiceTransaction {
            chosen_public_key: chosen_public_key.to_vec(),
            chooser_dh_public_key: dh_keypair.public_key().to_vec(),
            zk_proof,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Serialize and store transaction
        let tx_data = bincode::serialize(&choice_tx)
            .map_err(|e| crate::utils::Error::SerializationError(e.to_string()))?;

        let _record = self.storage.store_data(tx_data, RecordType::ChoiceTransaction).await?;

        // Update participant states
        if let Some(chooser_state) = self.participants.get_mut(chooser_pk) {
            chooser_state.has_chosen = true;
        }

        if let Some(chosen_state) = self.participants.get_mut(chosen_public_key) {
            chosen_state.chosen_by = Some(chooser_pk.to_vec());
        }

        self.current_phase = Phase::Choice;
        Ok(())
    }

    /// Execute REVEAL phase - participant reveals identity to their Secret Santa
    pub async fn reveal_phase(
        &mut self,
        keypair: &KeyPair,
        identity_info: &str,
        dh_keypair: &crate::crypto::DHKeyExchange,
        santa_dh_public_key: &[u8],
    ) -> crate::utils::Result<()> {
        if !matches!(self.current_phase, Phase::Choice | Phase::Reveal) {
            return Err(crate::utils::Error::ProtocolError(
                "REVEAL phase not available in current state".to_string()
            ));
        }

        let participant_pk = keypair.public_key.as_bytes();
        
        // Verify participant has been chosen
        let participant_state = self.participants.get(participant_pk)
            .ok_or_else(|| crate::utils::Error::ProtocolError(
                "Participant not found".to_string()
            ))?;

        if participant_state.chosen_by.is_none() {
            return Err(crate::utils::Error::ProtocolError(
                "Participant has not been chosen by anyone".to_string()
            ));
        }

        // Generate shared secret and encrypt identity
        let shared_secret = dh_keypair.compute_shared_secret(santa_dh_public_key)?;
        let encrypted_identity = crate::crypto::encrypt_data(identity_info.as_bytes(), &shared_secret)?;

        // Create signature proving ownership of public key
        let message = format!("reveal:{}", hex::encode(participant_pk));
        let signature = keypair.sign(message.as_bytes());

        // Create REVEAL transaction
        let reveal_tx = RevealTransaction {
            public_key: participant_pk.to_vec(),
            encrypted_identity,
            dh_public_key: dh_keypair.public_key().to_vec(),
            signature: signature.to_bytes().to_vec(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Serialize and store transaction
        let tx_data = bincode::serialize(&reveal_tx)
            .map_err(|e| crate::utils::Error::SerializationError(e.to_string()))?;

        let _record = self.storage.store_data(tx_data, RecordType::RevealTransaction).await?;

        // Update participant state
        if let Some(participant_state) = self.participants.get_mut(participant_pk) {
            participant_state.has_revealed = true;
        }

        self.current_phase = Phase::Reveal;
        Ok(())
    }

    /// Get current phase of the protocol
    pub fn current_phase(&self) -> &Phase {
        &self.current_phase
    }

    /// Get list of available public keys for choosing
    pub async fn get_available_choices(&self) -> crate::utils::Result<Vec<Vec<u8>>> {
        let all_keys = self.storage.get_all_public_keys().await?;
        
        // Filter out keys that have already been chosen
        let available_keys = all_keys.into_iter()
            .filter(|key| {
                !self.participants.get(key)
                    .map(|p| p.chosen_by.is_some())
                    .unwrap_or(false)
            })
            .collect();

        Ok(available_keys)
    }
}
