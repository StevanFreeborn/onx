use std::io::{Read, Write};

use onspring::SaveListItemRequest;
use serde::Deserialize;
use uuid::Uuid;

use crate::cli::ListsCommand;
use crate::client::OnspringRunner;
use crate::error::CliResult;
use crate::output::{SaveListItemResponseOutput, SuccessResponse, write_json};
use crate::validation::{parse_body, validate_positive_i32};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveListItemInput {
  pub id: Option<Uuid>,
  pub name: String,
  pub numeric_value: Option<f64>,
  pub color: Option<String>,
}

impl From<SaveListItemInput> for SaveListItemRequest {
  fn from(value: SaveListItemInput) -> Self {
    Self {
      id: value.id,
      name: value.name,
      numeric_value: value.numeric_value,
      color: value.color,
    }
  }
}

pub async fn handle<C: OnspringRunner, W: Write>(
  command: &ListsCommand,
  client: &C,
  writer: &mut W,
  pretty: bool,
) -> CliResult<()> {
  handle_with_reader(command, client, writer, std::io::stdin(), pretty).await
}

pub async fn handle_with_reader<C: OnspringRunner, W: Write, R: Read>(
  command: &ListsCommand,
  client: &C,
  writer: &mut W,
  reader: R,
  pretty: bool,
) -> CliResult<()> {
  match command {
    ListsCommand::SaveItem { list_id, body } => {
      validate_positive_i32(*list_id, "list_id")?;
      let input: SaveListItemInput = parse_body(body, reader)?;
      let request: SaveListItemRequest = input.into();
      let response = client.save_list_item(*list_id, request).await?;
      let output: SaveListItemResponseOutput = response.into();
      write_json(writer, &output, pretty)?;
    }
    ListsCommand::DeleteItem { list_id, item_id } => {
      validate_positive_i32(*list_id, "list_id")?;
      client.delete_list_item(*list_id, *item_id).await?;
      write_json(writer, &SuccessResponse { ok: true }, pretty)?;
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use onspring::{OnspringError, SaveListItemResponse};

  use crate::cli::BodySource;
  use crate::client::testing::MockClient;
  use crate::error::CliError;

  // --- SaveItem tests ---

  #[tokio::test]
  async fn handle_when_save_item_succeeds_it_should_write_save_response() {
    let item_uuid = Uuid::parse_str("d3b07384-d113-40e1-a20c-55c3c0a4e70e").unwrap();
    let mock_client = MockClient {
      save_list_item_result: Ok(SaveListItemResponse { id: item_uuid }),
      ..Default::default()
    };

    let command = ListsCommand::SaveItem {
      list_id: 1,
      body: BodySource {
        json: Some(
          r##"{"id":"d3b07384-d113-40e1-a20c-55c3c0a4e70e","name":"Item 1","numericValue":10.5,"color":"#ffffff"}"##
            .to_string(),
        ),
        file: None,
        stdin: false,
      },
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();
    assert_eq!(
      written.trim(),
      r#"{"id":"d3b07384-d113-40e1-a20c-55c3c0a4e70e"}"#
    );
    assert_eq!(*mock_client.save_list_item_list_id.lock().unwrap(), Some(1));
    let req = mock_client
      .save_list_item_request
      .lock()
      .unwrap()
      .clone()
      .unwrap();
    assert_eq!(req.id, Some(item_uuid));
    assert_eq!(req.name, "Item 1");
    assert_eq!(req.numeric_value, Some(10.5));
    assert_eq!(req.color, Some("#ffffff".to_string()));
  }

  #[tokio::test]
  async fn handle_when_save_item_succeeds_pretty_it_should_write_pretty_json() {
    let item_uuid = Uuid::parse_str("d3b07384-d113-40e1-a20c-55c3c0a4e70e").unwrap();
    let mock_client = MockClient {
      save_list_item_result: Ok(SaveListItemResponse { id: item_uuid }),
      ..Default::default()
    };

    let command = ListsCommand::SaveItem {
      list_id: 1,
      body: BodySource {
        json: Some(r#"{"name":"New Item"}"#.to_string()),
        file: None,
        stdin: false,
      },
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();
    let expected = concat!(
      "{\n",
      "  \"id\": \"d3b07384-d113-40e1-a20c-55c3c0a4e70e\"\n",
      "}"
    );
    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_save_item_with_invalid_list_id_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = ListsCommand::SaveItem {
      list_id: 0,
      body: BodySource {
        json: Some(r#"{"name":"Item"}"#.to_string()),
        file: None,
        stdin: false,
      },
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("list_id must be greater than 0."))
    );
    assert!(buffer.is_empty());
  }

  #[tokio::test]
  async fn handle_when_save_item_has_missing_body_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = ListsCommand::SaveItem {
      list_id: 1,
      body: BodySource::default(),
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage(
        "A request body is required. Provide --json, --file, or --stdin."
      ))
    );
    assert!(buffer.is_empty());
  }

  #[tokio::test]
  async fn handle_when_save_item_reads_from_stdin() {
    let item_uuid = Uuid::parse_str("d3b07384-d113-40e1-a20c-55c3c0a4e70e").unwrap();
    let mock_client = MockClient {
      save_list_item_result: Ok(SaveListItemResponse { id: item_uuid }),
      ..Default::default()
    };

    let command = ListsCommand::SaveItem {
      list_id: 5,
      body: BodySource {
        json: None,
        file: None,
        stdin: true,
      },
    };
    let mut buffer = Vec::new();
    let stdin_data = r#"{"name":"From Stdin","numericValue":2.0}"#;

    let result = handle_with_reader(
      &command,
      &mock_client,
      &mut buffer,
      stdin_data.as_bytes(),
      false,
    )
    .await;

    assert_eq!(result, Ok(()));
    let written = String::from_utf8(buffer).unwrap();
    assert_eq!(
      written.trim(),
      r#"{"id":"d3b07384-d113-40e1-a20c-55c3c0a4e70e"}"#
    );
    let req = mock_client
      .save_list_item_request
      .lock()
      .unwrap()
      .clone()
      .unwrap();
    assert_eq!(req.name, "From Stdin");
    assert_eq!(req.numeric_value, Some(2.0));
  }

  #[tokio::test]
  async fn handle_when_save_item_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      save_list_item_result: Err(OnspringError::Api {
        status_code: 400,
        message: "Invalid list item".to_string(),
      }),
      ..Default::default()
    };

    let command = ListsCommand::SaveItem {
      list_id: 1,
      body: BodySource {
        json: Some(r#"{"name":"Item"}"#.to_string()),
        file: None,
        stdin: false,
      },
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime(
        "API request failed (400): Invalid list item"
      ))
    );
    assert!(buffer.is_empty());
  }

  // --- DeleteItem tests ---

  #[tokio::test]
  async fn handle_when_delete_item_succeeds_it_should_write_success_response() {
    let item_uuid = Uuid::parse_str("d3b07384-d113-40e1-a20c-55c3c0a4e70e").unwrap();
    let mock_client = MockClient {
      delete_list_item_result: Ok(()),
      ..Default::default()
    };

    let command = ListsCommand::DeleteItem {
      list_id: 1,
      item_id: item_uuid,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();
    assert_eq!(written.trim(), r#"{"ok":true}"#);
    assert_eq!(*mock_client.delete_list_item_list_id.lock().unwrap(), Some(1));
    assert_eq!(
      *mock_client.delete_list_item_item_id.lock().unwrap(),
      Some(item_uuid)
    );
  }

  #[tokio::test]
  async fn handle_when_delete_item_succeeds_pretty_it_should_write_pretty_json() {
    let item_uuid = Uuid::parse_str("d3b07384-d113-40e1-a20c-55c3c0a4e70e").unwrap();
    let mock_client = MockClient {
      delete_list_item_result: Ok(()),
      ..Default::default()
    };

    let command = ListsCommand::DeleteItem {
      list_id: 1,
      item_id: item_uuid,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();
    let expected = concat!("{\n", "  \"ok\": true\n", "}");
    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_delete_item_with_invalid_list_id_it_should_return_usage_error() {
    let item_uuid = Uuid::parse_str("d3b07384-d113-40e1-a20c-55c3c0a4e70e").unwrap();
    let mock_client = MockClient::default();

    let command = ListsCommand::DeleteItem {
      list_id: 0,
      item_id: item_uuid,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("list_id must be greater than 0."))
    );
    assert!(buffer.is_empty());
  }

  #[tokio::test]
  async fn handle_when_delete_item_fails_it_should_return_mapped_cli_error() {
    let item_uuid = Uuid::parse_str("d3b07384-d113-40e1-a20c-55c3c0a4e70e").unwrap();
    let mock_client = MockClient {
      delete_list_item_result: Err(OnspringError::Api {
        status_code: 404,
        message: "List item not found".to_string(),
      }),
      ..Default::default()
    };

    let command = ListsCommand::DeleteItem {
      list_id: 1,
      item_id: item_uuid,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime(
        "API request failed (404): List item not found"
      ))
    );
    assert!(buffer.is_empty());
  }
}

