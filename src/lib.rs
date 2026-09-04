mod cli;
mod client;
mod commands;
mod error;
mod output;
mod validation;

use onspring::OnspringClient;

pub use cli::{Cli, Command, parse_cli_from};
pub use client::OnspringRunner;
pub use error::{CliError, CliResult, render_cli_error};

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
  commands::handle(cli, client, writer).await
}

#[cfg(test)]
mod tests {
  use super::*;

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
  async fn run_with_client_it_should_delegate_to_commands() {
    let mock_client = client::testing::MockClient::default();
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
}

