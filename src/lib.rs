mod cli;
mod output;

use clap::{Parser, error::ErrorKind};
use cli::{Cli, Command};

use onspring::{OnspringClient, OnspringError};
use output::{SuccessResponse, render_error, write_json};

pub type CliResult<T> = Result<T, CliError>;

pub trait OnspringRunner {
  fn ping(&self) -> impl std::future::Future<Output = Result<(), OnspringError>> + Send;
}

impl OnspringRunner for OnspringClient {
  async fn ping(&self) -> Result<(), OnspringError> {
    self.ping().await
  }
}

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

impl From<OnspringError> for CliError {
  fn from(value: OnspringError) -> Self {
    match value {
      OnspringError::InvalidArgument(message) => Self::usage(format!("Invalid request: {message}")),
      OnspringError::Api {
        status_code,
        message,
      } => {
        if message.is_empty() {
          Self::runtime(format!("API request failed with status {status_code}."))
        } else {
          Self::runtime(format!("API request failed ({status_code}): {message}"))
        }
      }
      OnspringError::Http(_) => Self::runtime("Network request failed."),
      OnspringError::Serialization(_) => Self::runtime("Failed to parse API response."),
    }
  }
}

pub async fn run(cli: Cli) -> CliResult<()> {
  let api_key = cli
    .api_key
    .as_deref()
    .ok_or_else(|| CliError::usage("Missing API Key. Set --api-key or ONSPRING_API_KEY."))?;

  let mut builder = OnspringClient::builder(api_key.to_string());

  if let Some(base_url) = &cli.base_url {
    if base_url.trim().is_empty() {
      return Err(CliError::usage("Base URL cannot be empty."));
    }

    builder = builder.base_url(base_url);
  }

  let client = builder.build();
  let mut stdout = std::io::stdout();
  run_with_client(&cli, &client, &mut stdout).await
}

pub async fn run_with_client<C: OnspringRunner, W: std::io::Write>(
  cli: &Cli,
  client: &C,
  writer: &mut W,
) -> CliResult<()> {
  match cli.command {
    Command::Ping => {
      client.ping().await?;
      write_json(writer, &SuccessResponse { ok: true }, cli.pretty)?;
    }
  }

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

  struct MockClient {
    ping_result: Result<(), OnspringError>,
  }

  impl OnspringRunner for MockClient {
    async fn ping(&self) -> Result<(), OnspringError> {
      match &self.ping_result {
        Ok(()) => Ok(()),
        Err(OnspringError::InvalidArgument(msg)) => {
          Err(OnspringError::InvalidArgument(msg.clone()))
        }
        Err(OnspringError::Api {
          status_code,
          message,
        }) => Err(OnspringError::Api {
          status_code: *status_code,
          message: message.clone(),
        }),
        Err(OnspringError::Serialization(_)) => {
          let serde_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
          Err(OnspringError::Serialization(serde_err))
        }
        Err(OnspringError::Http(_)) => {
          Err(OnspringError::InvalidArgument("http error".to_string()))
        }
      }
    }
  }

  #[test]
  fn parse_cli_from_when_called_with_valid_subcommand_it_should_return_cli() {
    let result = parse_cli_from(["test", "ping"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Ping,
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_without_subcommand_it_should_return_usage_error() {
    let result = parse_cli_from(["test"]);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 2);
    assert!(err.message.contains("Usage: test [OPTIONS] <COMMAND>"));
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

  #[test]
  fn cli_error_constructors_should_set_expected_codes_and_messages() {
    let info = CliError::info("info message");
    assert_eq!(info.code, 0);
    assert_eq!(info.message, "info message");

    let runtime = CliError::runtime("runtime message");
    assert_eq!(runtime.code, 1);
    assert_eq!(runtime.message, "runtime message");

    let usage = CliError::usage("usage message");
    assert_eq!(usage.code, 2);
    assert_eq!(usage.message, "usage message");
  }

  #[test]
  fn cli_error_from_onspring_error_invalid_argument() {
    let err = OnspringError::InvalidArgument("bad param".to_string());
    let cli_err = CliError::from(err);

    assert_eq!(cli_err.code, 2);
    assert_eq!(cli_err.message, "Invalid request: bad param");
  }

  #[test]
  fn cli_error_from_onspring_error_api_with_message() {
    let err = OnspringError::Api {
      status_code: 404,
      message: "Resource not found".to_string(),
    };
    let cli_err = CliError::from(err);

    assert_eq!(cli_err.code, 1);
    assert_eq!(
      cli_err.message,
      "API request failed (404): Resource not found"
    );
  }

  #[test]
  fn cli_error_from_onspring_error_api_with_empty_message() {
    let err = OnspringError::Api {
      status_code: 500,
      message: String::new(),
    };
    let cli_err = CliError::from(err);

    assert_eq!(cli_err.code, 1);
    assert_eq!(cli_err.message, "API request failed with status 500.");
  }

  #[test]
  fn cli_error_from_onspring_error_serialization() {
    let serde_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let err = OnspringError::Serialization(serde_err);
    let cli_err = CliError::from(err);

    assert_eq!(cli_err.code, 1);
    assert_eq!(cli_err.message, "Failed to parse API response.");
  }

  #[tokio::test]
  async fn run_when_missing_api_key_it_should_return_usage_error() {
    let cli = Cli {
      api_key: None,
      base_url: None,
      pretty: false,
      command: Command::Ping,
    };

    let result = run(cli).await;

    assert_eq!(
      result,
      Err(CliError::usage(
        "Missing API Key. Set --api-key or ONSPRING_API_KEY."
      ))
    );
  }

  #[tokio::test]
  async fn run_when_empty_base_url_it_should_return_usage_error() {
    let cli = Cli {
      api_key: Some("test-api-key".to_string()),
      base_url: Some("   ".to_string()),
      pretty: false,
      command: Command::Ping,
    };

    let result = run(cli).await;

    assert_eq!(result, Err(CliError::usage("Base URL cannot be empty.")));
  }

  #[tokio::test]
  async fn run_with_client_when_ping_succeeds_it_should_write_success_response() {
    let mock_client = MockClient {
      ping_result: Ok(()),
    };

    let cli = Cli {
      api_key: Some("key".to_string()),
      base_url: None,
      pretty: false,
      command: Command::Ping,
    };

    let mut buffer = Vec::new();

    let result = run_with_client(&cli, &mock_client, &mut buffer).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    assert_eq!(written.trim(), r#"{"ok":true}"#);
  }

  #[tokio::test]
  async fn run_with_client_when_ping_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      ping_result: Ok(()),
    };

    let cli = Cli {
      api_key: Some("key".to_string()),
      base_url: None,
      pretty: true,
      command: Command::Ping,
    };

    let mut buffer = Vec::new();

    let result = run_with_client(&cli, &mock_client, &mut buffer).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    assert_eq!(written.trim(), "{\n  \"ok\": true\n}");
  }

  #[tokio::test]
  async fn run_with_client_when_ping_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      ping_result: Err(OnspringError::InvalidArgument("Invalid Key".to_string())),
    };

    let cli = Cli {
      api_key: Some("key".to_string()),
      base_url: None,
      pretty: false,
      command: Command::Ping,
    };

    let mut buffer = Vec::new();

    let result = run_with_client(&cli, &mock_client, &mut buffer).await;

    assert_eq!(result, Err(CliError::usage("Invalid request: Invalid Key")));
    assert!(buffer.is_empty());
  }
}
