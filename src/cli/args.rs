use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "zkretctl", about = "ZKretSanta CLI")]
pub struct CliArgs {
    #[arg(long, default_value = "key.zkret")]
    pub keypair_file: String,
    #[arg(subcommand)]
    pub command: super::commands::Commands,
}
