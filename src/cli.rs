use clap::{Parser, Subcommand};

#[derive(Debug, PartialEq, Parser)]
#[command(name = "onx")]
#[command(version)]
#[command(about)]
pub struct Cli {
  #[arg(long)]
  #[arg(short = 'k')]
  #[arg(env = "ONSPRING_API_KEY")]
  #[arg(help = "The API key for the Onspring instance")]
  pub api_key: Option<String>,

  #[arg(long)]
  #[arg(short = 'u')]
  #[arg(env = "ONSPRING_BASE_URL")]
  #[arg(help = "The base URL for the Onspring API")]
  pub base_url: Option<String>,

  #[arg(long)]
  #[arg(short = 'p')]
  #[arg(help = "Pretty-print JSON output")]
  #[arg(default_value_t = false)]
  pub pretty: bool,

  #[command(subcommand)]
  pub command: Command,
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum Command { 
  #[command(about = "Check connectivity to the Onspring API")]
  Ping,
}
