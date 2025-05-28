use crate::crypto::{KeyPair, DHKeyExchange};
use crate::filecoin::FilecoinStorage;
use crate::protocol::SecretSantaProtocol;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Path to keypair file
    #[arg(short, long, default_value = "key.zkret")]
    pub keypair_file: PathBuf,
    
    /// Filecoin endpoint
    #[arg(long, default_value = "https://api.node.glif.io")]
    pub filecoin_endpoint: String,
    
    /// Authentication token for Filecoin
    #[arg(long, env = "FILECOIN_AUTH_TOKEN")]
    pub auth_token: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new keypair
    Keygen,
    
    /// Enter the Secret Santa protocol
    Enter,
    
    /// List available public keys for choosing
    ChoiceList,
    
    /// Choose a participant (make a choice)
    ChoiceMake {
        /// Public key of the chosen participant (hex encoded)
        chosen_public_key: String,
    },
    
    /// Check if you have a Secret Santa (someone chose you)
    CheckMySanta,
    
    /// Reveal your information to your Secret Santa
    Reveal {
        /// Information to reveal to your Secret Santa
        info_plaintext: String,
    },
    
    /// Check if your chosen participant (santee) has revealed their info
    CheckMySantee,
    
    /// Display protocol status
    Status,
}

pub async fn execute_command(cli: Cli) -> crate::utils::Result<()> {
    // Initialize Filecoin storage
    let mut storage = FilecoinStorage::new(&cli.filecoin_endpoint, &cli.auth_token).await?;
    let mut protocol = SecretSantaProtocol::new(storage).await?;

    match cli.command {
        Commands::Keygen => {
            let keypair = KeyPair::generate();
            save_keypair(&keypair, &cli.keypair_file)?;
            println!("Generated new keypair and saved to: {}", cli.keypair_file.display());
            println!("Public key: {}", hex::encode(keypair.public_key.as_bytes()));
        }

        Commands::Enter => {
            let keypair = load_keypair(&cli.keypair_file)?;
            protocol.enter_phase(&keypair).await?;
            println!("Successfully entered the Secret Santa protocol!");
        }

        Commands::ChoiceList => {
            let choices = protocol.get_available_choices().await?;
            println!("Available public keys to choose from:");
            for (i, pk) in choices.iter().enumerate() {
                println!("  {}: {}", i + 1, hex::encode(pk));
            }
        }

        Commands::ChoiceMake { chosen_public_key } => {
            let keypair = load_keypair(&cli.keypair_file)?;
            let chosen_pk_bytes = hex::decode(&chosen_public_key)
                .map_err(|e| crate::utils::Error::InvalidInput(e.to_string()))?;
            
            let dh_keypair = DHKeyExchange::generate();
            protocol.choice_phase(&keypair, &chosen_pk_bytes, &dh_keypair).await?;
            
            // Save DH keypair for later use in reveal phase
            save_dh_keypair(&dh_keypair, &cli.keypair_file)?;
            
            println!("Successfully chose participant: {}", chosen_public_key);
        }

        Commands::CheckMySanta => {
            let keypair = load_keypair(&cli.keypair_file)?;
            let has_santa = check_if_chosen(&protocol, keypair.public_key.as_bytes()).await?;
            
            if has_santa {
                println!("You have a Secret Santa! They will contact you once you reveal your info.");
            } else {
                println!("You don't have a Secret Santa yet. Wait for someone to choose you.");
            }
        }

        Commands::Reveal { info_plaintext } => {
            let keypair = load_keypair(&cli.keypair_file)?;
            let dh_keypair = load_dh_keypair(&cli.keypair_file)?;
            
            // Get Santa's DH public key from choice transaction
            let santa_dh_pk = get_santa_dh_public_key(&protocol, keypair.public_key.as_bytes()).await?;
            
            protocol.reveal_phase(&keypair, &info_plaintext, &dh_keypair, &santa_dh_pk).await?;
            println!("Successfully revealed your information to your Secret Santa!");
        }

        Commands::CheckMySantee => {
            let keypair = load_keypair(&cli.keypair_file)?;
            let dh_keypair = load_dh_keypair(&cli.keypair_file)?;
            
            let santee_info = get_santee_revealed_info(&protocol, &keypair, &dh_keypair).await?;
            
            match santee_info {
                Some(info) => {
                    println!("Your santee has revealed their information:");
                    println!("  {}", info);
                }
                None => {
                    println!("Your santee hasn't revealed their information yet.");
                }
            }
        }

        Commands::Status => {
            let current_phase = protocol.current_phase();
            println!("Current protocol phase: {:?}", current_phase);
            
            let choices = protocol.get_available_choices().await?;
            println!("Available participants: {}", choices.len());
        }
    }

    Ok(())
}

// Helper functions for file I/O and protocol queries
fn save_keypair(keypair: &KeyPair, path: &PathBuf) -> crate::utils::Result<()> {
    let (public_hex, secret_hex) = keypair.to_hex_strings();
    let data = format!("{}:{}", public_hex, secret_hex);
    
    std::fs::write(path, data)
        .map_err(|e| crate::utils::Error::FileError(e.to_string()))?;
    
    Ok(())
}

fn load_keypair(path: &PathBuf) -> crate::utils::Result<KeyPair> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| crate::utils::Error::FileError(e.to_string()))?;
    
    let parts: Vec<&str> = data.trim().split(':').collect();
    if parts.len() != 2 {
        return Err(crate::utils::Error::FileError("Invalid keypair file format".to_string()));
    }
    
    KeyPair::from_hex_strings(parts[0], parts[1])
}

fn save_dh_keypair(dh_keypair: &DHKeyExchange, base_path: &PathBuf) -> crate::utils::Result<()> {
    let dh_path = base_path.with_extension("dh");
    let hex_data = hex::encode(dh_keypair.secret_key());
    
    std::fs::write(dh_path, hex_data)
        .map_err(|e| crate::utils::Error::FileError(e.to_string()))?;
    
    Ok(())
}

fn load_dh_keypair(base_path: &PathBuf) -> crate::utils::Result<DHKeyExchange> {
    let dh_path = base_path.with_extension("dh");
    let hex_data = std::fs::read_to_string(dh_path)
        .map_err(|e| crate::utils::Error::FileError(e.to_string()))?;
    
    let secret_bytes = hex::decode(hex_data.trim())
        .map_err(|e| crate::utils::Error::SerializationError(e.to_string()))?;
    
    DHKeyExchange::from_secret_bytes(&secret_bytes)
}

async fn check_if_chosen(
    protocol: &SecretSantaProtocol,
    public_key: &[u8],
) -> crate::utils::Result<bool> {
    // Implementation would check if this public key appears in any choice transaction
    todo!("Implement check_if_chosen")
}

async fn get_santa_dh_public_key(
    protocol: &SecretSantaProtocol,
    public_key: &[u8],
) -> crate::utils::Result<Vec<u8>> {
    // Implementation would find the choice transaction where this key was chosen
    // and return the chooser's DH public key
    todo!("Implement get_santa_dh_public_key")
}

async fn get_santee_revealed_info(
    protocol: &SecretSantaProtocol,
    keypair: &KeyPair,
    dh_keypair: &DHKeyExchange,
) -> crate::utils::Result<Option<String>> {
    // Implementation would find reveal transaction from chosen participant
    // and decrypt their information using DH shared secret
    todo!("Implement get_santee_revealed_info")
}
