//! ZKretSanta: A trustless Secret Santa protocol using zero-knowledge proofs and Filecoin storage
//! 
//! This library provides a complete implementation of a decentralized Secret Santa protocol
//! that preserves participant privacy through zero-knowledge proofs while using Filecoin
//! for persistent, decentralized storage.

pub mod cli;
pub mod crypto;
pub mod filecoin;
pub mod protocol;
pub mod utils;

pub use crypto::{KeyPair, ZKProof, DHKeyExchange};
pub use filecoin::{FilecoinStorage, StorageClient};
pub use protocol::{SecretSantaProtocol, Phase, ProtocolState};
pub use utils::{Error, Result};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        crypto::{KeyPair, ZKProof},
        filecoin::FilecoinStorage,
        protocol::{SecretSantaProtocol, Phase},
        utils::{Error, Result},
    };
}
