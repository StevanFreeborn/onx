mod cli;
mod output;

use clap::{Parser, error::ErrorKind};
use cli::Cli;

use output::render_error;

#[derive(Debug, PartialEq)]
pub struct CliError {
  pub code: i32,
  pub message: String,
}

impl CliError {
  pub fn info(message: impl Into<String>) -> Self {
    Self {
      code: 0,
      message: message.into(),
    }
  }

  pub fn runtime(message: impl Into<String>) -> Self {
    Self {
      code: 1,
      message: message.into(),
    }
  }

  pub fn usage(message: impl Into<String>) -> Self {
    Self {
      code: 2,
      message: message.into(),
    }
  }
}

pub type CliResult<T> = Result<T, CliError>;

pub async fn run(_cli: Cli) -> CliResult<()> {
  Ok(())
}

pub fn parse_cli_from<I, T>(args: I) -> CliResult<Cli>
where
  I: IntoIterator<Item = T>,
  T: Into<std::ffi::OsString> + Clone,
{
  Cli::try_parse_from(args).map_err(|e| match e.kind() {
    ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => CliError::info(e.to_string()),
    _ => CliError::usage(e.to_string()),
  })
}

pub fn render_cli_error(err: &CliError, pretty: bool) -> String {
  render_error(err, pretty)
    .unwrap_or_else(|_| "{\"error\":{\"code\":1,\"message\":\"Unexpected error.\"}}".to_string())
}
