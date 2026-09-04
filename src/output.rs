use serde::Serialize;

use crate::{CliError, CliResult};

#[derive(Debug, Serialize)]
struct ErrorBody<'a> {
  pub code: i32,
  pub message: &'a str,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope<'a> {
  pub error: ErrorBody<'a>,
}

pub fn render_json<T: Serialize>(value: &T, pretty: bool) -> CliResult<String> {
  if pretty {
    serde_json::to_string_pretty(value).map_err(|_| CliError::runtime("Failed to serialize output"))
  } else {
    serde_json::to_string(value).map_err(|_| CliError::runtime("Failed to serialize output"))
  }
}

pub fn render_error(err: &CliError, pretty: bool) -> CliResult<String> {
  let envelope = ErrorEnvelope {
    error: ErrorBody {
      code: err.code,
      message: &err.message,
    },
  };

  render_json(&envelope, pretty)
}
