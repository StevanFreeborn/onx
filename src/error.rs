use onspring::OnspringError;

use crate::output::render_error;

pub type CliResult<T> = Result<T, CliError>;

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

pub fn render_cli_error(err: &CliError, pretty: bool) -> String {
  render_error(err, pretty)
    .unwrap_or_else(|_| "{\"error\":{\"code\":1,\"message\":\"Unexpected error.\"}}".to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

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
}
