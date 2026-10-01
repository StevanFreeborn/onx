use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::cli::Cli;
use crate::error::{CliError, CliResult};

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigFile {
  pub api_key: Option<String>,
  pub base_url: Option<String>,
  pub pretty: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedConfig {
  pub api_key: Option<String>,
  pub base_url: Option<String>,
  pub pretty: bool,
}

pub fn default_config_path() -> Option<PathBuf> {
  let home = std::env::var("HOME")
    .ok()
    .or_else(|| std::env::var("USERPROFILE").ok())?;

  Some(PathBuf::from(home).join(".config").join("onx").join("config.json"))
}

pub fn load_config_file(path: &Path, is_explicit: bool) -> CliResult<Option<ConfigFile>> {
  if !path.exists() {
    if is_explicit {
      return Err(CliError::usage(format!(
        "Config file not found: {}",
        path.display()
      )));
    }
    return Ok(None);
  }

  let contents = fs::read_to_string(path).map_err(|err| {
    CliError::runtime(format!(
      "Failed to read config file {}: {err}",
      path.display()
    ))
  })?;

  let config: ConfigFile = serde_json::from_str(&contents).map_err(|err| {
    CliError::runtime(format!(
      "Failed to parse config file {}: {err}",
      path.display()
    ))
  })?;

  Ok(Some(config))
}

pub fn resolve_config(cli: &Cli, config_file: Option<&ConfigFile>) -> ResolvedConfig {
  let config_file_default = ConfigFile::default();
  let file = config_file.unwrap_or(&config_file_default);

  // Precedence: CLI Flag > Config File > Environment Variable
  let api_key = cli
    .api_key
    .clone()
    .or_else(|| file.api_key.clone())
    .or_else(|| std::env::var("ONSPRING_API_KEY").ok());

  let base_url = cli
    .base_url
    .clone()
    .or_else(|| file.base_url.clone())
    .or_else(|| std::env::var("ONSPRING_BASE_URL").ok());

  let pretty = if cli.pretty {
    true
  } else {
    file.pretty.unwrap_or(false)
  };

  ResolvedConfig {
    api_key,
    base_url,
    pretty,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::cli::Command;
  use std::io::Write;
  use tempfile::NamedTempFile;

  #[test]
  fn load_config_file_when_file_exists_it_should_parse_camel_case_json() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
      file,
      r#"{{"apiKey":"file-key","baseUrl":"https://config.onspring.com","pretty":true}}"#
    )
    .unwrap();

    let loaded = load_config_file(file.path(), true).unwrap();
    assert_eq!(
      loaded,
      Some(ConfigFile {
        api_key: Some("file-key".to_string()),
        base_url: Some("https://config.onspring.com".to_string()),
        pretty: Some(true),
      })
    );
  }

  #[test]
  fn load_config_file_when_not_explicit_and_file_missing_it_should_return_none() {
    let missing_path = PathBuf::from("non_existent_config_file_path_12345.json");
    let loaded = load_config_file(&missing_path, false).unwrap();
    assert_eq!(loaded, None);
  }

  #[test]
  fn load_config_file_when_explicit_and_file_missing_it_should_return_usage_error() {
    let missing_path = PathBuf::from("non_existent_config_file_path_12345.json");
    let err = load_config_file(&missing_path, true).unwrap_err();
    assert_eq!(err.code, 2);
    assert!(err.message.contains("Config file not found"));
  }

  #[test]
  fn load_config_file_when_invalid_json_it_should_return_runtime_error() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "not json").unwrap();

    let err = load_config_file(file.path(), true).unwrap_err();
    assert_eq!(err.code, 1);
    assert!(err.message.contains("Failed to parse config file"));
  }

  #[test]
  fn default_config_path_when_home_or_userprofile_set_it_should_resolve_path() {
    let path = default_config_path();
    assert!(path.is_some());
    let p = path.unwrap();
    assert!(p.ends_with(".config/onx/config.json") || p.ends_with(r#".config\onx\config.json"#));
  }

  #[test]
  fn resolve_config_when_cli_flag_provided_it_should_take_precedence() {
    let cli = Cli {
      api_key: Some("cli-key".to_string()),
      base_url: Some("https://cli.onspring.com".to_string()),
      config: None,
      pretty: true,
      command: Command::Ping,
    };
    let file = ConfigFile {
      api_key: Some("file-key".to_string()),
      base_url: Some("https://file.onspring.com".to_string()),
      pretty: Some(false),
    };

    let resolved = resolve_config(&cli, Some(&file));
    assert_eq!(
      resolved,
      ResolvedConfig {
        api_key: Some("cli-key".to_string()),
        base_url: Some("https://cli.onspring.com".to_string()),
        pretty: true,
      }
    );
  }

  #[test]
  fn resolve_config_when_cli_flag_absent_it_should_use_config_file() {
    let cli = Cli {
      api_key: None,
      base_url: None,
      config: None,
      pretty: false,
      command: Command::Ping,
    };
    let file = ConfigFile {
      api_key: Some("file-key".to_string()),
      base_url: Some("https://file.onspring.com".to_string()),
      pretty: Some(true),
    };

    let resolved = resolve_config(&cli, Some(&file));
    assert_eq!(
      resolved,
      ResolvedConfig {
        api_key: Some("file-key".to_string()),
        base_url: Some("https://file.onspring.com".to_string()),
        pretty: true,
      }
    );
  }

  #[test]
  fn resolve_config_when_no_file_it_should_default_pretty_to_false() {
    let cli = Cli {
      api_key: None,
      base_url: None,
      config: None,
      pretty: false,
      command: Command::Ping,
    };

    let resolved = resolve_config(&cli, None);
    assert!(!resolved.pretty);
  }
}
