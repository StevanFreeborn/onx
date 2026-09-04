use std::io::Write;

use crate::client::OnspringRunner;
use crate::error::CliResult;
use crate::output::{SuccessResponse, write_json};

pub async fn handle<C: OnspringRunner, W: Write>(
  client: &C,
  writer: &mut W,
  pretty: bool,
) -> CliResult<()> {
  client.ping().await?;
  write_json(writer, &SuccessResponse { ok: true }, pretty)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use onspring::OnspringError;

  use crate::client::testing::MockClient;
  use crate::error::CliError;

  #[tokio::test]
  async fn handle_when_ping_succeeds_it_should_write_success_response() {
    let mock_client = MockClient {
      ping_result: Ok(()),
      ..Default::default()
    };

    let mut buffer = Vec::new();

    let result = handle(&mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    assert_eq!(written.trim(), r#"{"ok":true}"#);
  }

  #[tokio::test]
  async fn handle_when_ping_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      ping_result: Ok(()),
      ..Default::default()
    };

    let mut buffer = Vec::new();

    let result = handle(&mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    assert_eq!(written.trim(), "{\n  \"ok\": true\n}");
  }

  #[tokio::test]
  async fn handle_when_ping_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      ping_result: Err(OnspringError::InvalidArgument("Invalid Key".to_string())),
      ..Default::default()
    };

    let mut buffer = Vec::new();

    let result = handle(&mock_client, &mut buffer, false).await;

    assert_eq!(result, Err(CliError::usage("Invalid request: Invalid Key")));
    assert!(buffer.is_empty());
  }
}
