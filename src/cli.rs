use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

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

  #[command(about = "Perform operations against the fields in the instance")]
  Fields {
    #[command(subcommand)]
    command: FieldsCommand,
  },

  #[command(about = "Perform operations against the records in the instance")]
  Records {
    #[command(subcommand)]
    command: RecordsCommand,
  },

  #[command(about = "Perform operations against the files in the instance")]
  Files {
    #[command(subcommand)]
    command: FilesCommand,
  },
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum AppsCommand {
  #[command(about = "Get information for a list of apps")]
  List(PagingArgs),

  #[command(about = "Get information about an app")]
  Get {
    #[arg(long = "app")]
    #[arg(short = 'a')]
    #[arg(help = "App id to get")]
    app_id: i32,
  },

  #[command(about = "Get information for a batch of apps")]
  BatchGet {
    #[arg(long = "apps")]
    #[arg(value_delimiter = ',')]
    #[arg(short = 'a')]
    #[arg(help = "Comma-separated list of app ids to get")]
    ids: Vec<i32>,
  },
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum FieldsCommand {
  #[command(about = "Get information for a list of fields")]
  List {
    #[arg(long = "app")]
    #[arg(short = 'a')]
    #[arg(help = "App id whose fields will be retrieved")]
    app_id: i32,

    #[command(flatten)]
    paging: PagingArgs,
  },

  #[command(about = "Get information about a field")]
  Get {
    #[arg(long = "field")]
    #[arg(short = 'f')]
    #[arg(help = "Field id to get")]
    field_id: i32,
  },

  #[command(about = "Get information for a batch of fields")]
  BatchGet {
    #[arg(long = "fields")]
    #[arg(value_delimiter = ',')]
    #[arg(short = 'f')]
    #[arg(help = "Comma-separated list of field ids to get")]
    ids: Vec<i32>,
  },
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum RecordsCommand {
  #[command(about = "Get information for a list of records")]
  List {
    #[arg(long = "app")]
    #[arg(short = 'a')]
    #[arg(help = "App id whose records will be retrieved")]
    app_id: i32,

    #[command(flatten)]
    paging: PagingArgs,

    #[arg(long = "fields")]
    #[arg(value_delimiter = ',')]
    #[arg(short = 'f')]
    #[arg(help = "Comma-separated list of fields whose data will be included in the response")]
    field_ids: Vec<i32>,

    #[arg(long)]
    data_format: Option<DataFormatArg>,
  },

  #[command(about = "Get information for a record")]
  Get {
    #[arg(long = "app")]
    #[arg(short = 'a')]
    #[arg(help = "App id for record to be retrieved")]
    app_id: i32,

    #[arg(long = "record")]
    #[arg(short = 'r')]
    #[arg(help = "Record id for record to be retrieved")]
    record_id: i32,

    #[arg(long = "fields")]
    #[arg(value_delimiter = ',')]
    #[arg(short = 'f')]
    #[arg(help = "Comma-separated list of fields whose data will be included in the response")]
    field_ids: Vec<i32>,

    #[arg(long = "format")]
    #[arg(short = 'd')]
    #[arg(help = "Format of data in response")]
    data_format: Option<DataFormatArg>,
  },

  #[command(about = "Add or update a record")]
  #[command()]
  Save {
    #[command(flatten)]
    body: BodySource,
  },

  #[command(about = "Delete a record")]
  Delete {
    #[arg(long = "app")]
    #[arg(short = 'a')]
    #[arg(help = "App id for record to delete")]
    app_id: i32,

    #[arg(long = "record")]
    #[arg(short = 'r')]
    #[arg(help = "Record id for record to delete")]
    record_id: i32,
  },

  #[command(about = "Get information for a batch of records")]
  BatchGet {
    #[command(flatten)]
    body: BodySource,
  },

  #[command(about = "Get information for a list of records based on a query")]
  Query {
    #[command(flatten)]
    body: BodySource,

    #[command(flatten)]
    paging: PagingArgs,
  },

  #[command(about = "Delete a batch of records")]
  BatchDelete {
    #[command(flatten)]
    body: BodySource,
  },
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum FilesCommand {
  #[command(about = "Get metadata info for a file")]
  Info {
    #[arg(long = "record")]
    #[arg(short = 'r')]
    #[arg(help = "Record id for record where the file is held")]
    record_id: i32,

    #[arg(long = "field")]
    #[arg(short = 'f')]
    #[arg(help = "Field id for field where the file is held")]
    field_id: i32,

    #[arg(long = "file")]
    #[arg(short = 'l')]
    #[arg(help = "The id of the file")]
    file_id: i32,
  },

  #[command(about = "Get the content for a file")]
  Get {
    #[arg(long = "record")]
    #[arg(short = 'r')]
    #[arg(help = "Record id for record where the file is held")]
    record_id: i32,

    #[arg(long = "field")]
    #[arg(short = 'f')]
    #[arg(help = "Field id for field where the file is held")]
    field_id: i32,

    #[arg(long = "file")]
    #[arg(short = 'l')]
    #[arg(help = "The id of the file")]
    file_id: i32,

    #[arg(long)]
    #[arg(short = 'o')]
    #[arg(help = "The output path for the file")]
    output: Option<PathBuf>,
  },

  Upload {
    #[arg(long = "record")]
    #[arg(short = 'r')]
    #[arg(help = "Record id for record where the file should be uploaded")]
    record_id: i32,

    #[arg(long = "field")]
    #[arg(short = 'f')]
    #[arg(help = "Field id for field where the file should be uploaded")]
    field_id: i32,

    #[arg(long)]
    #[arg(short = 'i')]
    #[arg(help = "The input path of the file to be uploaded")]
    input: Option<PathBuf>,

    #[arg(long)]
    #[arg(short = 's')]
    #[arg(help = "Indicates whether file data should be read in from standard in")]
    stdin: bool,

    #[arg(long = "name")]
    #[arg(short = 'n')]
    #[arg(help = "The name of the file to be uploaded")]
    file_name: Option<String>,

    #[arg(long = "type")]
    #[arg(short = 't')]
    #[arg(help = "The content type of the file to be uploaded")]
    content_type: Option<String>,

    #[arg(long)]
    #[arg(help = "Notes that should be included in the file upload")]
    notes: Option<String>,

    #[arg(long)]
    #[arg(help = "Modified date that should be included in the file upload as RFC3339 timestamp")]
    modified_date: Option<String>,
  },

  Delete {
    #[arg(long = "record")]
    #[arg(short = 'r')]
    #[arg(help = "Record id for record where the file is held")]
    record_id: i32,

    #[arg(long = "field")]
    #[arg(short = 'f')]
    #[arg(help = "Field id for field where the file is held")]
    field_id: i32,

    #[arg(long = "file")]
    #[arg(short = 'l')]
    #[arg(help = "The id of the file")]
    file_id: i32,
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

#[derive(Debug, PartialEq, Clone, Args, Default)]
pub struct BodySource {
  #[arg(long)]
  #[arg(short = 'j')]
  #[arg(help = "Inline JSON request body")]
  pub json: Option<String>,

  #[arg(long)]
  #[arg(short = 'f')]
  #[arg(help = "Path to a JSON request body file")]
  pub file: Option<PathBuf>,

  #[arg(long)]
  #[arg(help = "Read JSON request body from stdin")]
  pub stdin: bool,
}

#[derive(Debug, PartialEq, Clone, Copy, ValueEnum)]
pub enum DataFormatArg {
  Raw,
  Formatted,
}

pub fn parse_cli_from<I, T>(args: I) -> crate::error::CliResult<Cli>
where
  I: IntoIterator<Item = T>,
  T: Into<std::ffi::OsString> + Clone,
{
  use crate::error::CliError;
  use clap::error::ErrorKind;

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
    let result = parse_cli_from(["test", "apps", "get", "--app", "123"]);

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
    let result = parse_cli_from(["test", "apps", "get", "-a", "123"]);

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
    let result = parse_cli_from(["test", "apps", "batch-get", "--apps", "1,2,3"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Apps {
          command: AppsCommand::BatchGet { ids: vec![1, 2, 3] },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_apps_batch_get_without_ids_it_should_return_cli_with_empty_ids()
   {
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
  fn parse_cli_from_when_called_with_apps_batch_get_invalid_id_format_it_should_return_usage_error()
  {
    let result = parse_cli_from(["test", "apps", "batch-get", "--apps", "invalid"]);

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
  fn parse_cli_from_when_called_with_fields_list_it_should_return_cli() {
    let result = parse_cli_from(["test", "fields", "list", "--app", "10"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Fields {
          command: FieldsCommand::List {
            app_id: 10,
            paging: PagingArgs::default(),
          },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_fields_list_paging_args_it_should_return_cli() {
    let result = parse_cli_from([
      "test",
      "fields",
      "list",
      "-a",
      "10",
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
        command: Command::Fields {
          command: FieldsCommand::List {
            app_id: 10,
            paging: PagingArgs {
              page_number: Some(2),
              page_size: Some(25),
            },
          },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_fields_list_missing_app_id_it_should_return_usage_error() {
    let result = parse_cli_from(["test", "fields", "list"]);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 2);
  }

  #[test]
  fn parse_cli_from_when_called_with_fields_get_it_should_return_cli() {
    let result = parse_cli_from(["test", "fields", "get", "--field", "123"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Fields {
          command: FieldsCommand::Get { field_id: 123 },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_fields_get_short_flag_it_should_return_cli() {
    let result = parse_cli_from(["test", "fields", "get", "-f", "123"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Fields {
          command: FieldsCommand::Get { field_id: 123 },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_fields_get_missing_field_id_it_should_return_usage_error() {
    let result = parse_cli_from(["test", "fields", "get"]);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 2);
  }

  #[test]
  fn parse_cli_from_when_called_with_fields_batch_get_it_should_return_cli() {
    let result = parse_cli_from(["test", "fields", "batch-get", "--fields", "1,2,3"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Fields {
          command: FieldsCommand::BatchGet { ids: vec![1, 2, 3] },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_fields_batch_get_without_ids_it_should_return_cli_with_empty_ids()
   {
    let result = parse_cli_from(["test", "fields", "batch-get"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Fields {
          command: FieldsCommand::BatchGet { ids: vec![] },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_fields_batch_get_invalid_id_format_it_should_return_usage_error()
   {
    let result = parse_cli_from(["test", "fields", "batch-get", "--fields", "invalid"]);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 2);
  }

  #[test]
  fn parse_cli_from_when_called_with_fields_without_subcommand_it_should_return_usage_error() {
    let result = parse_cli_from(["test", "fields"]);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 2);
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
  fn parse_cli_from_when_called_with_records_list_it_should_return_cli() {
    let result = parse_cli_from(["test", "records", "list", "--app", "1"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Records {
          command: RecordsCommand::List {
            app_id: 1,
            paging: PagingArgs::default(),
            field_ids: vec![],
            data_format: None,
          },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_records_list_all_flags_it_should_return_cli() {
    let result = parse_cli_from([
      "test",
      "records",
      "list",
      "-a",
      "1",
      "-n",
      "2",
      "-s",
      "10",
      "-f",
      "100,200",
      "--data-format",
      "raw",
    ]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Records {
          command: RecordsCommand::List {
            app_id: 1,
            paging: PagingArgs {
              page_number: Some(2),
              page_size: Some(10),
            },
            field_ids: vec![100, 200],
            data_format: Some(DataFormatArg::Raw),
          },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_records_get_it_should_return_cli() {
    let result = parse_cli_from([
      "test",
      "records",
      "get",
      "-a",
      "1",
      "-r",
      "10",
      "-f",
      "100",
      "-d",
      "formatted",
    ]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Records {
          command: RecordsCommand::Get {
            app_id: 1,
            record_id: 10,
            field_ids: vec![100],
            data_format: Some(DataFormatArg::Formatted),
          },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_records_save_json_it_should_return_cli() {
    let result = parse_cli_from(["test", "records", "save", "--json", r#"{"appId":1}"#]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Records {
          command: RecordsCommand::Save {
            body: BodySource {
              json: Some(r#"{"appId":1}"#.to_string()),
              file: None,
              stdin: false,
            },
          },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_records_delete_it_should_return_cli() {
    let result = parse_cli_from(["test", "records", "delete", "-a", "1", "-r", "10"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Records {
          command: RecordsCommand::Delete {
            app_id: 1,
            record_id: 10,
          },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_records_batch_get_it_should_return_cli() {
    let result = parse_cli_from(["test", "records", "batch-get", "-f", "input.json"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Records {
          command: RecordsCommand::BatchGet {
            body: BodySource {
              json: None,
              file: Some(std::path::PathBuf::from("input.json")),
              stdin: false,
            },
          },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_records_query_it_should_return_cli() {
    let result = parse_cli_from(["test", "records", "query", "--stdin", "-n", "1", "-s", "50"]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Records {
          command: RecordsCommand::Query {
            body: BodySource {
              json: None,
              file: None,
              stdin: true,
            },
            paging: PagingArgs {
              page_number: Some(1),
              page_size: Some(50),
            },
          },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_records_batch_delete_it_should_return_cli() {
    let result = parse_cli_from([
      "test",
      "records",
      "batch-delete",
      "-j",
      r#"{"appId":1,"recordIds":[10]}"#,
    ]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Records {
          command: RecordsCommand::BatchDelete {
            body: BodySource {
              json: Some(r#"{"appId":1,"recordIds":[10]}"#.to_string()),
              file: None,
              stdin: false,
            },
          },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_records_without_subcommand_it_should_return_usage_error() {
    let result = parse_cli_from(["test", "records"]);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 2);
  }

  #[test]
  fn parse_cli_from_when_called_with_files_info_it_should_return_cli() {
    let result = parse_cli_from([
      "test", "files", "info", "--record", "1", "--field", "2", "--file", "3",
    ]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Files {
          command: FilesCommand::Info {
            record_id: 1,
            field_id: 2,
            file_id: 3,
          },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_files_get_it_should_return_cli() {
    let result = parse_cli_from([
      "test", "files", "get", "-r", "1", "-f", "2", "-l", "3", "-o", "out.bin",
    ]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Files {
          command: FilesCommand::Get {
            record_id: 1,
            field_id: 2,
            file_id: 3,
            output: Some(std::path::PathBuf::from("out.bin")),
          },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_files_upload_it_should_return_cli() {
    let result = parse_cli_from([
      "test", "files", "upload", "-r", "1", "-f", "2", "-i", "doc.pdf", "-n", "renamed.pdf",
      "-t", "application/pdf", "--notes", "test notes", "--modified-date",
      "2026-01-01T00:00:00Z",
    ]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Files {
          command: FilesCommand::Upload {
            record_id: 1,
            field_id: 2,
            input: Some(std::path::PathBuf::from("doc.pdf")),
            stdin: false,
            file_name: Some("renamed.pdf".to_string()),
            content_type: Some("application/pdf".to_string()),
            notes: Some("test notes".to_string()),
            modified_date: Some("2026-01-01T00:00:00Z".to_string()),
          },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_files_delete_it_should_return_cli() {
    let result = parse_cli_from([
      "test", "files", "delete", "--record", "10", "--field", "20", "--file", "30",
    ]);

    assert_eq!(
      result,
      Ok(Cli {
        api_key: None,
        base_url: None,
        pretty: false,
        command: Command::Files {
          command: FilesCommand::Delete {
            record_id: 10,
            field_id: 20,
            file_id: 30,
          },
        },
      })
    );
  }

  #[test]
  fn parse_cli_from_when_called_with_files_without_subcommand_it_should_return_usage_error() {
    let result = parse_cli_from(["test", "files"]);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 2);
  }
}
