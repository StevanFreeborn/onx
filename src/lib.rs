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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_cli_from_when_called_with_valid_program_name_it_should_return_cli() {
    let result = parse_cli_from(["test"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_unknown_flag_it_should_return_usage_error() {
    let result = parse_cli_from(["test", "--bogus"]);

    assert!(result.is_err());
    assert_eq!(result.as_ref().unwrap_err().code, 2);
    assert!(
      result
        .as_ref()
        .unwrap_err()
        .message
        .contains("unexpected argument")
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_help_flag_it_should_return_info_with_code_0() {
    let result = parse_cli_from(["test", "--help"]);

    let err = result.unwrap_err();

    assert_eq!(err.code, 0);
    assert!(err.message.contains(env!("CARGO_PKG_DESCRIPTION")));
  }

  #[test]
  fn parse_cli_from_when_called_with_version_flag_it_should_return_info_with_code_0() {
    let result = parse_cli_from(["test", "--version"]);

    let err = result.unwrap_err();

    assert_eq!(err.code, 0);
    assert!(err.message.contains(env!("CARGO_PKG_NAME")));
    assert!(err.message.contains(env!("CARGO_PKG_VERSION")));
  }

  #[test]
  fn render_cli_error_when_called_with_error_it_should_render_json() {
    let error = CliError {
      code: 1,
      message: String::from("Something went wrong"),
    };

    let result = render_cli_error(&error, false);

    assert_eq!(
      result,
      r#"{"error":{"code":1,"message":"Something went wrong"}}"#
    );
  }
}

