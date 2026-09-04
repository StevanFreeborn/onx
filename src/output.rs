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

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn render_json_when_called_without_pretty_it_should_print_value() {
    let data = json!({
        "name": "Stevan",
    });

    let result = render_json(&data, false);

    let expected = Ok(String::from(r#"{"name":"Stevan"}"#));
    assert_eq!(result, expected);
  }

  #[test]
  fn render_json_when_called_with_pretty_it_should_pretty_print_value() {
    let data = json!({
        "name": "Stevan",
    });

    let result = render_json(&data, true);

    let expected = Ok(String::from("{\n  \"name\": \"Stevan\"\n}"));
    assert_eq!(result, expected);
  }

  #[test]
  fn render_error_when_called_with_error_it_should_render_properly() {
    let error = CliError {
      code: 1,
      message: String::from("This is a test"),
    };

    let result = render_error(&error, false);

    let expected = Ok(String::from(
      r#"{"error":{"code":1,"message":"This is a test"}}"#,
    ));
    assert_eq!(result, expected);
  }
}

