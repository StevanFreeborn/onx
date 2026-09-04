pub mod apps;
pub mod ping;

use std::io::Write;

use crate::cli::{Cli, Command};
use crate::client::OnspringRunner;
use crate::error::CliResult;

pub async fn handle<C: OnspringRunner, W: Write>(
  cli: &Cli,
  client: &C,
  writer: &mut W,
) -> CliResult<()> {
  match &cli.command {
    Command::Ping => ping::handle(client, writer, cli.pretty).await,
    Command::Apps { command } => apps::handle(command, client, writer, cli.pretty).await,
  }
}
