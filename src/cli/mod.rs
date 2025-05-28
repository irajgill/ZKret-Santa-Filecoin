use clap::{Parser, Subcommand};
use crate::crypto::KeyPair;
use crate::protocol::SecretSantaProtocol;

#[derive(Parser)]
#[command(name = "zkretctl", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Keygen,
    Register,
    Verify { cid: String },
}

pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Keygen => {
            let keypair = KeyPair::generate();
            println!("Generated keypair. Public key: {:?}", keypair.public.to_bytes());
        }
        Commands::Register => {
            let protocol = SecretSantaProtocol::new();
            let keypair = KeyPair::generate();
            protocol.register(&keypair).unwrap();
            println!("Registered!");
        }
        Commands::Verify { cid } => {
            let protocol = SecretSantaProtocol::new();
            let valid = protocol.verify_registration(&cid).unwrap();
            println!("Verification result: {}", valid);
        }
    }
}
