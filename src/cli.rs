use clap::{Args, Parser, Subcommand};

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
  #[command(about = "Perform operations against the apps in the instance")]
  Apps {
    #[command(subcommand)]
    command: AppsCommand,
  },
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum AppsCommand {
  #[command(about = "Get information for a list of apps")]
  List(PagingArgs),

  #[command(about = "Get information about an app")]
  Get {
    #[arg(long)]
    #[arg(short = 'i')]
    #[arg(help = "App id to get")]
    app_id: i32,
  },

  #[command(about = "Get information for a batch of apps")]
  BatchGet {
    #[arg(long)]
    #[arg(value_delimiter = ',')]
    #[arg(short = 'i')]
    #[arg(help = "Comma-separated list of app ids to get")]
    ids: Vec<i32>,
  },
}

#[derive(Debug, PartialEq, Clone, Args, Default)]
pub struct PagingArgs {
  #[arg(long)]
  #[arg(short = 'n')]
  #[arg(help = "The page number to retrieve")]
  pub page_number: Option<i32>,

  #[arg(long)]
  #[arg(short = 's')]
  #[arg(help = "The size of the page retrieve")]
  pub page_size: Option<i32>,
}

pub fn parse_cli_from<I, T>(args: I) -> crate::error::CliResult<Cli>
where
  I: IntoIterator<Item = T>,
  T: Into<std::ffi::OsString> + Clone,
{
  use clap::error::ErrorKind;
  use crate::error::CliError;

  Cli::try_parse_from(args).map_err(|e| match e.kind() {
    ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => CliError::info(e.to_string()),
    _ => CliError::usage(e.to_string()),
  })
}

#[cfg(test)]
mod tests {
  use super::*;

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
  fn parse_cli_from_when_called_with_apps_list_it_should_return_cli() {
    let result = parse_cli_from(["test", "apps", "list"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Apps {
          command: AppsCommand::List(PagingArgs::default()),
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_apps_list_paging_args_it_should_return_cli() {
    let result = parse_cli_from([
      "test",
      "apps",
      "list",
      "--page-number",
      "2",
      "--page-size",
      "25",
    ]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Apps {
          command: AppsCommand::List(PagingArgs {
            page_number: Some(2),
            page_size: Some(25),
          }),
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_apps_get_it_should_return_cli() {
    let result = parse_cli_from(["test", "apps", "get", "--app-id", "123"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Apps {
          command: AppsCommand::Get { app_id: 123 },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_apps_get_short_flag_it_should_return_cli() {
    let result = parse_cli_from(["test", "apps", "get", "-i", "123"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Apps {
          command: AppsCommand::Get { app_id: 123 },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_apps_get_missing_app_id_it_should_return_usage_error() {
    let result = parse_cli_from(["test", "apps", "get"]);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 2);
  }

  #[test]
  fn parse_cli_from_when_called_with_apps_batch_get_it_should_return_cli() {
    let result = parse_cli_from(["test", "apps", "batch-get", "--ids", "1,2,3"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Apps {
          command: AppsCommand::BatchGet {
            ids: vec![1, 2, 3],
          },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_apps_batch_get_without_ids_it_should_return_cli_with_empty_ids() {
    let result = parse_cli_from(["test", "apps", "batch-get"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Apps {
          command: AppsCommand::BatchGet { ids: vec![] },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_apps_batch_get_invalid_id_format_it_should_return_usage_error() {
    let result = parse_cli_from(["test", "apps", "batch-get", "--ids", "invalid"]);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 2);
  }

  #[test]
  fn parse_cli_from_when_called_with_apps_without_subcommand_it_should_return_usage_error() {
    let result = parse_cli_from(["test", "apps"]);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 2);
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
}

