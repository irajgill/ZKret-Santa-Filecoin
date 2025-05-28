use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKProof {
    pub proof_data: Vec<u8>,
    pub public_inputs: Vec<String>,
    pub proof_type: ProofType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofType {
    EnterPhase,
    ChoicePhase,
    RevealPhase,
}

pub struct ZKProofSystem {
    proving_keys: HashMap<ProofType, ProvingKey<Bn254>>,
    verifying_keys: HashMap<ProofType, VerifyingKey<Bn254>>,
}

impl ZKProofSystem {
    
    pub fn new() -> crate::utils::Result<Self> {
        let mut proving_keys = HashMap::new();
        let mut verifying_keys = HashMap::new();

      
        for proof_type in [ProofType::EnterPhase, ProofType::ChoicePhase, ProofType::RevealPhase] {
            let (pk, vk) = Self::generate_keys_for_circuit(&proof_type)?;
            proving_keys.insert(proof_type.clone(), pk);
            verifying_keys.insert(proof_type, vk);
        }

        Ok(Self {
            proving_keys,
            verifying_keys,
        })
    }

    ///proof for the ENTER phase
    pub fn prove_enter_phase(
        &self,
        public_key: &[u8],
        secret_key: &[u8],
    ) -> crate::utils::Result<ZKProof> {
        let proving_key = self.proving_keys.get(&ProofType::EnterPhase)
            .ok_or_else(|| crate::utils::Error::CryptoError("Enter phase proving key not found".to_string()))?;

        
        let proof_data = self.generate_proof_data(proving_key, &[public_key, secret_key])?;
        let public_inputs = vec![hex::encode(public_key)];

        Ok(ZKProof {
            proof_data,
            public_inputs,
            proof_type: ProofType::EnterPhase,
        })
    }

    ///proof for the CHOICE phase
    pub fn prove_choice_phase(
        &self,
        chooser_public_key: &[u8],
        chosen_public_key: &[u8],
        secret_key: &[u8],
    ) -> crate::utils::Result<ZKProof> {
        let proving_key = self.proving_keys.get(&ProofType::ChoicePhase)
            .ok_or_else(|| crate::utils::Error::CryptoError("Choice phase proving key not found".to_string()))?;

        let proof_data = self.generate_proof_data(
            proving_key,
            &[chooser_public_key, chosen_public_key, secret_key]
        )?;
        let public_inputs = vec![
            hex::encode(chooser_public_key),
            hex::encode(chosen_public_key),
        ];

        Ok(ZKProof {
            proof_data,
            public_inputs,
            proof_type: ProofType::ChoicePhase,
        })
    }

    /
    pub fn verify_proof(&self, proof: &ZKProof) -> crate::utils::Result<bool> {
        let verifying_key = self.verifying_keys.get(&proof.proof_type)
            .ok_or_else(|| crate::utils::Error::CryptoError("Verifying key not found".to_string()))?;

    
        let groth16_proof = self.deserialize_proof(&proof.proof_data)?;
        let public_inputs = self.parse_public_inputs(&proof.public_inputs)?;

        let is_valid = Groth16::<Bn254>::verify(verifying_key, &public_inputs, &groth16_proof)
            .map_err(|e| crate::utils::Error::CryptoError(e.to_string()))?;

        Ok(is_valid)
    }


    fn generate_keys_for_circuit(
        proof_type: &ProofType,
    ) -> crate::utils::Result<(ProvingKey<Bn254>, VerifyingKey<Bn254>)> {
        todo!("Implement circuit-specific key generation")
    }

    fn generate_proof_data(
        &self,
        proving_key: &ProvingKey<Bn254>,
        inputs: &[&[u8]],
    ) -> crate::utils::Result<Vec<u8>> {
        
        todo!("Implement proof generation")
    }

    fn deserialize_proof(&self, proof_data: &[u8]) -> crate::utils::Result<Proof<Bn254>> {
        Proof::<Bn254>::deserialize_compressed(proof_data)
            .map_err(|e| crate::utils::Error::CryptoError(e.to_string()))
    }

    fn parse_public_inputs(&self, inputs: &[String]) -> crate::utils::Result<Vec<Fr>> {
        
        todo!("Implement public input parsing")
    }
}
