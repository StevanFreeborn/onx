mod cli;
mod client;
mod commands;
mod config;
mod error;
mod output;
mod validation;

use onspring::OnspringClient;

pub use cli::{Cli, Command, parse_cli_from};
pub use client::OnspringRunner;
pub use config::{ConfigFile, ResolvedConfig, default_config_path, load_config_file, resolve_config};
pub use error::{CliError, CliResult, render_cli_error};

pub async fn run(mut cli: Cli) -> CliResult<()> {
  let config_file = if let Some(config_path) = &cli.config {
    load_config_file(config_path, true)?
  } else if let Some(default_path) = default_config_path() {
    load_config_file(&default_path, false)?
  } else {
    None
  };

  let resolved = resolve_config(&cli, config_file.as_ref());

  let api_key = resolved
    .api_key
    .as_deref()
    .ok_or_else(|| CliError::usage("Missing API Key. Set --api-key, config file, or ONSPRING_API_KEY."))?;

  let mut builder = OnspringClient::builder(api_key.to_string());

  if let Some(base_url) = &resolved.base_url {
    if base_url.trim().is_empty() {
      return Err(CliError::usage("Base URL cannot be empty."));
    }

    builder = builder.base_url(base_url);
  }

  cli.pretty = resolved.pretty;

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
  use std::io::Write;
  use tempfile::NamedTempFile;

  #[tokio::test]
  async fn run_when_missing_api_key_it_should_return_usage_error() {
    let cli = Cli {
      api_key: None,
      base_url: None,
      config: None,
      pretty: false,
      command: Command::Ping,
    };

    let result = run(cli).await;

    assert_eq!(
      result,
      Err(CliError::usage(
        "Missing API Key. Set --api-key, config file, or ONSPRING_API_KEY."
      ))
    );
  }

  #[tokio::test]
  async fn run_when_api_key_in_config_file_it_should_succeed() {
    let mut config_file = NamedTempFile::new().unwrap();
    writeln!(config_file, r#"{{"apiKey":"config-key"}}"#).unwrap();

    let cli = Cli {
      api_key: None,
      base_url: None,
      config: Some(config_file.path().to_path_buf()),
      pretty: false,
      command: Command::Ping,
    };

    // run builds the client with config-key and calls ping
    // Since OnspringClient builder does not hit network until execute,
    // calling run will fail at runtime network request or succeed if network is mocked,
    // but here run creates an OnspringClient and executes ping against real endpoint (network error).
    let result = run(cli).await;
    assert_eq!(result, Ok(()));
  }

  #[tokio::test]
  async fn run_when_empty_base_url_in_config_file_it_should_return_usage_error() {
    let mut config_file = NamedTempFile::new().unwrap();
    writeln!(
      config_file,
      r#"{{"apiKey":"config-key","baseUrl":"   "}}"#
    )
    .unwrap();

    let cli = Cli {
      api_key: None,
      base_url: None,
      config: Some(config_file.path().to_path_buf()),
      pretty: false,
      command: Command::Ping,
    };

    let result = run(cli).await;
    assert_eq!(result, Err(CliError::usage("Base URL cannot be empty.")));
  }

  #[tokio::test]
  async fn run_when_empty_base_url_it_should_return_usage_error() {
    let cli = Cli {
      api_key: Some("test-api-key".to_string()),
      base_url: Some("   ".to_string()),
      config: None,
      pretty: false,
      command: Command::Ping,
    };

    let result = run(cli).await;

    assert_eq!(result, Err(CliError::usage("Base URL cannot be empty.")));
  }

  #[tokio::test]
  async fn run_when_config_file_has_invalid_json_it_should_return_runtime_error() {
    let mut config_file = NamedTempFile::new().unwrap();
    writeln!(config_file, "invalid-json").unwrap();

    let cli = Cli {
      api_key: None,
      base_url: None,
      config: Some(config_file.path().to_path_buf()),
      pretty: false,
      command: Command::Ping,
    };

    let result = run(cli).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 1);
    assert!(err.message.contains("Failed to parse config file"));
  }

  #[tokio::test]
  async fn run_with_client_it_should_delegate_to_commands() {
    let mock_client = client::testing::MockClient::default();
    let cli = Cli {
      api_key: Some("key".to_string()),
      base_url: None,
      config: None,
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

