use std::collections::HashMap;
use std::io::{Read, Write};

use onspring::{
  BatchDeleteRecordsRequest, BatchGetRecordsRequest, DataFormat, QueryRecordsRequest,
  SaveRecordRequest,
};
use serde::Deserialize;

use crate::cli::{DataFormatArg, RecordsCommand};
use crate::client::OnspringRunner;
use crate::error::CliResult;
use crate::output::{
  CollectionOutput, PagedOutput, RecordOutput, SaveRecordResponseOutput, SuccessResponse,
  write_json,
};
use crate::validation::{parse_body, paging_request, validate_positive_i32};

impl From<DataFormatArg> for DataFormat {
  fn from(value: DataFormatArg) -> Self {
    match value {
      DataFormatArg::Raw => DataFormat::Raw,
      DataFormatArg::Formatted => DataFormat::Formatted,
    }
  }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveRecordInput {
  pub app_id: i32,
  pub record_id: Option<i32>,
  pub fields: HashMap<String, serde_json::Value>,
}

impl From<SaveRecordInput> for SaveRecordRequest {
  fn from(value: SaveRecordInput) -> Self {
    Self {
      app_id: value.app_id,
      record_id: value.record_id,
      fields: value.fields,
    }
  }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchGetRecordsInput {
  pub app_id: i32,
  pub record_ids: Vec<i32>,
  pub field_ids: Option<Vec<i32>>,
  pub data_format: Option<DataFormat>,
}

impl From<BatchGetRecordsInput> for BatchGetRecordsRequest {
  fn from(value: BatchGetRecordsInput) -> Self {
    Self {
      app_id: value.app_id,
      record_ids: value.record_ids,
      field_ids: value.field_ids,
      data_format: value.data_format,
    }
  }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryRecordsInput {
  pub app_id: i32,
  pub filter: String,
  pub field_ids: Option<Vec<i32>>,
  pub data_format: Option<DataFormat>,
}

impl From<QueryRecordsInput> for QueryRecordsRequest {
  fn from(value: QueryRecordsInput) -> Self {
    Self {
      app_id: value.app_id,
      filter: value.filter,
      field_ids: value.field_ids,
      data_format: value.data_format,
    }
  }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteRecordsInput {
  pub app_id: i32,
  pub record_ids: Vec<i32>,
}

impl From<BatchDeleteRecordsInput> for BatchDeleteRecordsRequest {
  fn from(value: BatchDeleteRecordsInput) -> Self {
    Self {
      app_id: value.app_id,
      record_ids: value.record_ids,
    }
  }
}

pub async fn handle<C: OnspringRunner, W: Write>(
  command: &RecordsCommand,
  client: &C,
  writer: &mut W,
  pretty: bool,
) -> CliResult<()> {
  handle_with_reader(command, client, writer, std::io::stdin(), pretty).await
}

pub async fn handle_with_reader<C: OnspringRunner, W: Write, R: Read>(
  command: &RecordsCommand,
  client: &C,
  writer: &mut W,
  reader: R,
  pretty: bool,
) -> CliResult<()> {
  match command {
    RecordsCommand::List {
      app_id,
      paging,
      field_ids,
      data_format,
    } => {
      validate_positive_i32(*app_id, "app_id")?;
      let paging = paging_request(paging)?;
      let field_ids_slice = if field_ids.is_empty() {
        None
      } else {
        Some(field_ids.as_slice())
      };
      let data_format = data_format.map(Into::into);

      let response = client
        .list_records(*app_id, paging, field_ids_slice, data_format)
        .await?;
      let output: PagedOutput<RecordOutput> = response.into();
      write_json(writer, &output, pretty)?;
    }
    RecordsCommand::Get {
      app_id,
      record_id,
      field_ids,
      data_format,
    } => {
      validate_positive_i32(*app_id, "app_id")?;
      validate_positive_i32(*record_id, "record_id")?;
      let field_ids_slice = if field_ids.is_empty() {
        None
      } else {
        Some(field_ids.as_slice())
      };
      let data_format = data_format.map(Into::into);

      let response = client
        .get_record(*app_id, *record_id, field_ids_slice, data_format)
        .await?;
      let output: RecordOutput = response.into();
      write_json(writer, &output, pretty)?;
    }
    RecordsCommand::Save { body } => {
      let input: SaveRecordInput = parse_body(body, reader)?;
      let request: SaveRecordRequest = input.into();
      let response = client.save_record(request).await?;
      let output: SaveRecordResponseOutput = response.into();
      write_json(writer, &output, pretty)?;
    }
    RecordsCommand::Delete { app_id, record_id } => {
      validate_positive_i32(*app_id, "app_id")?;
      validate_positive_i32(*record_id, "record_id")?;
      client.delete_record(*app_id, *record_id).await?;
      write_json(writer, &SuccessResponse { ok: true }, pretty)?;
    }
    RecordsCommand::BatchGet { body } => {
      let input: BatchGetRecordsInput = parse_body(body, reader)?;
      let request: BatchGetRecordsRequest = input.into();
      let response = client.batch_get_records(request).await?;
      let output: CollectionOutput<RecordOutput> = response.into();
      write_json(writer, &output, pretty)?;
    }
    RecordsCommand::Query { body, paging } => {
      let input: QueryRecordsInput = parse_body(body, reader)?;
      let request: QueryRecordsRequest = input.into();
      let paging = paging_request(paging)?;
      let response = client.query_records(request, paging).await?;
      let output: PagedOutput<RecordOutput> = response.into();
      write_json(writer, &output, pretty)?;
    }
    RecordsCommand::BatchDelete { body } => {
      let input: BatchDeleteRecordsInput = parse_body(body, reader)?;
      let request: BatchDeleteRecordsRequest = input.into();
      client.batch_delete_records(request).await?;
      write_json(writer, &SuccessResponse { ok: true }, pretty)?;
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use onspring::{
    CollectionResponse, OnspringError, PagedResponse, Record, RecordFieldValue, SaveRecordResponse,
    ValueType,
  };
  use serde_json::json;

  use crate::cli::{BodySource, PagingArgs};
  use crate::client::testing::MockClient;
  use crate::error::CliError;

  fn test_record(app_id: i32, record_id: i32) -> Record {
    Record {
      app_id,
      record_id,
      field_data: Some(vec![RecordFieldValue {
        value_type: ValueType::String,
        field_id: 100,
        value: json!("Test Value"),
      }]),
    }
  }

  // --- List tests ---

  #[tokio::test]
  async fn handle_when_records_list_succeeds_it_should_write_paged_output() {
    let mock_client = MockClient {
      list_records_result: Ok(PagedResponse {
        page_number: Some(1),
        page_size: Some(50),
        total_pages: Some(1),
        total_records: Some(1),
        items: Some(vec![test_record(1, 10)]),
      }),
      ..Default::default()
    };

    let command = RecordsCommand::List {
      app_id: 1,
      paging: PagingArgs::default(),
      field_ids: vec![],
      data_format: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    assert_eq!(
      written.trim(),
      r#"{"pageNumber":1,"pageSize":50,"totalPages":1,"totalRecords":1,"items":[{"appId":1,"recordId":10,"fieldData":[{"type":"String","fieldId":100,"value":"Test Value"}]}]}"#
    );
    assert_eq!(*mock_client.list_records_app_id.lock().unwrap(), Some(1));
    assert!(
      mock_client
        .list_records_paging
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .is_none()
    );
    assert!(
      mock_client
        .list_records_field_ids
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .is_none()
    );
    assert!(
      mock_client
        .list_records_data_format
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .is_none()
    );
  }

  #[tokio::test]
  async fn handle_when_records_list_with_args_it_should_pass_all_args() {
    let mock_client = MockClient {
      list_records_result: Ok(PagedResponse {
        page_number: Some(2),
        page_size: Some(10),
        total_pages: Some(5),
        total_records: Some(50),
        items: Some(vec![]),
      }),
      ..Default::default()
    };

    let command = RecordsCommand::List {
      app_id: 1,
      paging: PagingArgs {
        page_number: Some(2),
        page_size: Some(10),
      },
      field_ids: vec![100, 200],
      data_format: Some(DataFormatArg::Formatted),
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    assert_eq!(*mock_client.list_records_app_id.lock().unwrap(), Some(1));
    let paging = mock_client
      .list_records_paging
      .lock()
      .unwrap()
      .as_ref()
      .unwrap()
      .clone()
      .unwrap();
    assert_eq!(paging.page_number, 2);
    assert_eq!(paging.page_size, 10);
    assert_eq!(
      *mock_client.list_records_field_ids.lock().unwrap(),
      Some(Some(vec![100, 200]))
    );
    assert_eq!(
      *mock_client.list_records_data_format.lock().unwrap(),
      Some(Some(DataFormat::Formatted))
    );
  }

  #[tokio::test]
  async fn handle_when_records_list_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      list_records_result: Ok(PagedResponse {
        page_number: Some(1),
        page_size: Some(50),
        total_pages: Some(1),
        total_records: Some(1),
        items: Some(vec![test_record(1, 10)]),
      }),
      ..Default::default()
    };

    let command = RecordsCommand::List {
      app_id: 1,
      paging: PagingArgs::default(),
      field_ids: vec![],
      data_format: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    let expected = concat!(
      "{\n",
      "  \"pageNumber\": 1,\n",
      "  \"pageSize\": 50,\n",
      "  \"totalPages\": 1,\n",
      "  \"totalRecords\": 1,\n",
      "  \"items\": [\n",
      "    {\n",
      "      \"appId\": 1,\n",
      "      \"recordId\": 10,\n",
      "      \"fieldData\": [\n",
      "        {\n",
      "          \"type\": \"String\",\n",
      "          \"fieldId\": 100,\n",
      "          \"value\": \"Test Value\"\n",
      "        }\n",
      "      ]\n",
      "    }\n",
      "  ]\n",
      "}"
    );

    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_records_list_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      list_records_result: Err(OnspringError::Api {
        status_code: 401,
        message: "Unauthorized".to_string(),
      }),
      ..Default::default()
    };

    let command = RecordsCommand::List {
      app_id: 1,
      paging: PagingArgs::default(),
      field_ids: vec![],
      data_format: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime("API request failed (401): Unauthorized"))
    );
    assert!(buffer.is_empty());
  }

  #[tokio::test]
  async fn handle_when_records_list_has_invalid_app_id_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = RecordsCommand::List {
      app_id: 0,
      paging: PagingArgs::default(),
      field_ids: vec![],
      data_format: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("app_id must be greater than 0."))
    );
    assert!(buffer.is_empty());
    assert!(mock_client.list_records_app_id.lock().unwrap().is_none());
  }

  #[tokio::test]
  async fn handle_when_records_list_has_invalid_paging_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = RecordsCommand::List {
      app_id: 1,
      paging: PagingArgs {
        page_number: Some(-1),
        page_size: None,
      },
      field_ids: vec![],
      data_format: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("page_number must be greater than 0."))
    );
    assert!(buffer.is_empty());
  }

  // --- Get tests ---

  #[tokio::test]
  async fn handle_when_records_get_succeeds_it_should_write_record_output() {
    let mock_client = MockClient {
      get_record_result: Ok(test_record(1, 10)),
      ..Default::default()
    };

    let command = RecordsCommand::Get {
      app_id: 1,
      record_id: 10,
      field_ids: vec![],
      data_format: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    assert_eq!(
      written.trim(),
      r#"{"appId":1,"recordId":10,"fieldData":[{"type":"String","fieldId":100,"value":"Test Value"}]}"#
    );
    assert_eq!(*mock_client.get_record_app_id.lock().unwrap(), Some(1));
    assert_eq!(*mock_client.get_record_record_id.lock().unwrap(), Some(10));
    assert!(
      mock_client
        .get_record_field_ids
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .is_none()
    );
    assert!(
      mock_client
        .get_record_data_format
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .is_none()
    );
  }

  #[tokio::test]
  async fn handle_when_records_get_with_args_it_should_pass_all_args() {
    let mock_client = MockClient {
      get_record_result: Ok(test_record(1, 10)),
      ..Default::default()
    };

    let command = RecordsCommand::Get {
      app_id: 1,
      record_id: 10,
      field_ids: vec![100],
      data_format: Some(DataFormatArg::Raw),
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    assert_eq!(*mock_client.get_record_app_id.lock().unwrap(), Some(1));
    assert_eq!(*mock_client.get_record_record_id.lock().unwrap(), Some(10));
    assert_eq!(
      *mock_client.get_record_field_ids.lock().unwrap(),
      Some(Some(vec![100]))
    );
    assert_eq!(
      *mock_client.get_record_data_format.lock().unwrap(),
      Some(Some(DataFormat::Raw))
    );
  }

  #[tokio::test]
  async fn handle_when_records_get_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      get_record_result: Ok(test_record(1, 10)),
      ..Default::default()
    };

    let command = RecordsCommand::Get {
      app_id: 1,
      record_id: 10,
      field_ids: vec![],
      data_format: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    let expected = concat!(
      "{\n",
      "  \"appId\": 1,\n",
      "  \"recordId\": 10,\n",
      "  \"fieldData\": [\n",
      "    {\n",
      "      \"type\": \"String\",\n",
      "      \"fieldId\": 100,\n",
      "      \"value\": \"Test Value\"\n",
      "    }\n",
      "  ]\n",
      "}"
    );

    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_records_get_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      get_record_result: Err(OnspringError::Api {
        status_code: 404,
        message: "Record not found".to_string(),
      }),
      ..Default::default()
    };

    let command = RecordsCommand::Get {
      app_id: 1,
      record_id: 999,
      field_ids: vec![],
      data_format: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime(
        "API request failed (404): Record not found"
      ))
    );
    assert!(buffer.is_empty());
  }

  #[tokio::test]
  async fn handle_when_records_get_with_invalid_app_id_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = RecordsCommand::Get {
      app_id: -1,
      record_id: 10,
      field_ids: vec![],
      data_format: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("app_id must be greater than 0."))
    );
    assert!(buffer.is_empty());
  }

  #[tokio::test]
  async fn handle_when_records_get_with_invalid_record_id_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = RecordsCommand::Get {
      app_id: 1,
      record_id: 0,
      field_ids: vec![],
      data_format: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("record_id must be greater than 0."))
    );
    assert!(buffer.is_empty());
  }

  // --- Save tests ---

  #[tokio::test]
  async fn handle_when_records_save_succeeds_it_should_write_save_response() {
    let mock_client = MockClient {
      save_record_result: Ok(SaveRecordResponse {
        id: 42,
        warnings: Some(vec!["Test warning".to_string()]),
      }),
      ..Default::default()
    };

    let command = RecordsCommand::Save {
      body: BodySource {
        json: Some(r#"{"appId":1,"fields":{"100":"New Value"}}"#.to_string()),
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
      r#"{"id":42,"warnings":["Test warning"]}"#
    );
    let req = mock_client.save_record_request.lock().unwrap().clone().unwrap();
    assert_eq!(req.app_id, 1);
    assert_eq!(req.fields.get("100"), Some(&json!("New Value")));
  }

  #[tokio::test]
  async fn handle_when_records_save_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      save_record_result: Ok(SaveRecordResponse {
        id: 42,
        warnings: None,
      }),
      ..Default::default()
    };

    let command = RecordsCommand::Save {
      body: BodySource {
        json: Some(r#"{"appId":1,"fields":{}}"#.to_string()),
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
      "  \"id\": 42,\n",
      "  \"warnings\": null\n",
      "}"
    );
    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_records_save_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      save_record_result: Err(OnspringError::Api {
        status_code: 400,
        message: "Invalid field value".to_string(),
      }),
      ..Default::default()
    };

    let command = RecordsCommand::Save {
      body: BodySource {
        json: Some(r#"{"appId":1,"fields":{}}"#.to_string()),
        file: None,
        stdin: false,
      },
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime(
        "API request failed (400): Invalid field value"
      ))
    );
  }

  #[tokio::test]
  async fn handle_when_records_save_has_missing_body_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = RecordsCommand::Save {
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
  }

  #[tokio::test]
  async fn handle_when_records_save_reads_from_stdin() {
    let mock_client = MockClient {
      save_record_result: Ok(SaveRecordResponse {
        id: 77,
        warnings: None,
      }),
      ..Default::default()
    };

    let command = RecordsCommand::Save {
      body: BodySource {
        json: None,
        file: None,
        stdin: true,
      },
    };
    let mut buffer = Vec::new();
    let stdin_data = r#"{"appId":2,"fields":{"100":"From Stdin"}}"#;

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
    assert_eq!(written.trim(), r#"{"id":77,"warnings":null}"#);
    let req = mock_client.save_record_request.lock().unwrap().clone().unwrap();
    assert_eq!(req.app_id, 2);
  }

  // --- Delete tests ---

  #[tokio::test]
  async fn handle_when_records_delete_succeeds_it_should_write_success_response() {
    let mock_client = MockClient {
      delete_record_result: Ok(()),
      ..Default::default()
    };

    let command = RecordsCommand::Delete {
      app_id: 1,
      record_id: 10,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    assert_eq!(written.trim(), r#"{"ok":true}"#);
    assert_eq!(*mock_client.delete_record_app_id.lock().unwrap(), Some(1));
    assert_eq!(*mock_client.delete_record_record_id.lock().unwrap(), Some(10));
  }

  #[tokio::test]
  async fn handle_when_records_delete_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      delete_record_result: Ok(()),
      ..Default::default()
    };

    let command = RecordsCommand::Delete {
      app_id: 1,
      record_id: 10,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    let expected = concat!("{\n", "  \"ok\": true\n", "}");
    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_records_delete_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      delete_record_result: Err(OnspringError::Api {
        status_code: 404,
        message: "Record not found".to_string(),
      }),
      ..Default::default()
    };

    let command = RecordsCommand::Delete {
      app_id: 1,
      record_id: 999,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime(
        "API request failed (404): Record not found"
      ))
    );
  }

  #[tokio::test]
  async fn handle_when_records_delete_with_invalid_app_id_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = RecordsCommand::Delete {
      app_id: 0,
      record_id: 10,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("app_id must be greater than 0."))
    );
  }

  #[tokio::test]
  async fn handle_when_records_delete_with_invalid_record_id_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = RecordsCommand::Delete {
      app_id: 1,
      record_id: -5,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("record_id must be greater than 0."))
    );
  }

  // --- BatchGet tests ---

  #[tokio::test]
  async fn handle_when_records_batch_get_succeeds_it_should_write_collection_output() {
    let mock_client = MockClient {
      batch_get_records_result: Ok(CollectionResponse {
        count: Some(1),
        items: Some(vec![test_record(1, 10)]),
      }),
      ..Default::default()
    };

    let command = RecordsCommand::BatchGet {
      body: BodySource {
        json: Some(r#"{"appId":1,"recordIds":[10]}"#.to_string()),
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
      r#"{"count":1,"items":[{"appId":1,"recordId":10,"fieldData":[{"type":"String","fieldId":100,"value":"Test Value"}]}]}"#
    );
    let req = mock_client
      .batch_get_records_request
      .lock()
      .unwrap()
      .clone()
      .unwrap();
    assert_eq!(req.app_id, 1);
    assert_eq!(req.record_ids, vec![10]);
  }

  #[tokio::test]
  async fn handle_when_records_batch_get_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      batch_get_records_result: Ok(CollectionResponse {
        count: Some(1),
        items: Some(vec![test_record(1, 10)]),
      }),
      ..Default::default()
    };

    let command = RecordsCommand::BatchGet {
      body: BodySource {
        json: Some(r#"{"appId":1,"recordIds":[10]}"#.to_string()),
        file: None,
        stdin: false,
      },
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    let val: serde_json::Value = serde_json::from_str(&written).unwrap();
    assert_eq!(val["count"], 1);
    assert!(written.contains("  \"count\": 1"));
  }

  #[tokio::test]
  async fn handle_when_records_batch_get_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      batch_get_records_result: Err(OnspringError::Api {
        status_code: 500,
        message: "Internal Server Error".to_string(),
      }),
      ..Default::default()
    };

    let command = RecordsCommand::BatchGet {
      body: BodySource {
        json: Some(r#"{"appId":1,"recordIds":[10]}"#.to_string()),
        file: None,
        stdin: false,
      },
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime(
        "API request failed (500): Internal Server Error"
      ))
    );
  }

  // --- Query tests ---

  #[tokio::test]
  async fn handle_when_records_query_succeeds_it_should_write_paged_output() {
    let mock_client = MockClient {
      query_records_result: Ok(PagedResponse {
        page_number: Some(1),
        page_size: Some(50),
        total_pages: Some(1),
        total_records: Some(1),
        items: Some(vec![test_record(1, 10)]),
      }),
      ..Default::default()
    };

    let command = RecordsCommand::Query {
      body: BodySource {
        json: Some(r#"{"appId":1,"filter":"100 eq 'Test'"}"#.to_string()),
        file: None,
        stdin: false,
      },
      paging: PagingArgs::default(),
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    assert_eq!(
      written.trim(),
      r#"{"pageNumber":1,"pageSize":50,"totalPages":1,"totalRecords":1,"items":[{"appId":1,"recordId":10,"fieldData":[{"type":"String","fieldId":100,"value":"Test Value"}]}]}"#
    );
    let req = mock_client
      .query_records_request
      .lock()
      .unwrap()
      .clone()
      .unwrap();
    assert_eq!(req.app_id, 1);
    assert_eq!(req.filter, "100 eq 'Test'");
  }

  #[tokio::test]
  async fn handle_when_records_query_with_paging_it_should_pass_paging_request() {
    let mock_client = MockClient {
      query_records_result: Ok(PagedResponse {
        page_number: Some(3),
        page_size: Some(25),
        total_pages: Some(4),
        total_records: Some(100),
        items: Some(vec![]),
      }),
      ..Default::default()
    };

    let command = RecordsCommand::Query {
      body: BodySource {
        json: Some(r#"{"appId":1,"filter":"100 eq 'Test'"}"#.to_string()),
        file: None,
        stdin: false,
      },
      paging: PagingArgs {
        page_number: Some(3),
        page_size: Some(25),
      },
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let paging = mock_client
      .query_records_paging
      .lock()
      .unwrap()
      .as_ref()
      .unwrap()
      .clone()
      .unwrap();
    assert_eq!(paging.page_number, 3);
    assert_eq!(paging.page_size, 25);
  }

  #[tokio::test]
  async fn handle_when_records_query_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      query_records_result: Err(OnspringError::Api {
        status_code: 400,
        message: "Invalid filter expression".to_string(),
      }),
      ..Default::default()
    };

    let command = RecordsCommand::Query {
      body: BodySource {
        json: Some(r#"{"appId":1,"filter":"invalid"}"#.to_string()),
        file: None,
        stdin: false,
      },
      paging: PagingArgs::default(),
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime(
        "API request failed (400): Invalid filter expression"
      ))
    );
  }

  #[tokio::test]
  async fn handle_when_records_query_has_invalid_paging_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = RecordsCommand::Query {
      body: BodySource {
        json: Some(r#"{"appId":1,"filter":"100 eq 'Test'"}"#.to_string()),
        file: None,
        stdin: false,
      },
      paging: PagingArgs {
        page_number: Some(0),
        page_size: None,
      },
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("page_number must be greater than 0."))
    );
  }

  // --- BatchDelete tests ---

  #[tokio::test]
  async fn handle_when_records_batch_delete_succeeds_it_should_write_success_response() {
    let mock_client = MockClient {
      batch_delete_records_result: Ok(()),
      ..Default::default()
    };

    let command = RecordsCommand::BatchDelete {
      body: BodySource {
        json: Some(r#"{"appId":1,"recordIds":[10,20]}"#.to_string()),
        file: None,
        stdin: false,
      },
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    assert_eq!(written.trim(), r#"{"ok":true}"#);
    let req = mock_client
      .batch_delete_records_request
      .lock()
      .unwrap()
      .clone()
      .unwrap();
    assert_eq!(req.app_id, 1);
    assert_eq!(req.record_ids, vec![10, 20]);
  }

  #[tokio::test]
  async fn handle_when_records_batch_delete_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      batch_delete_records_result: Ok(()),
      ..Default::default()
    };

    let command = RecordsCommand::BatchDelete {
      body: BodySource {
        json: Some(r#"{"appId":1,"recordIds":[10,20]}"#.to_string()),
        file: None,
        stdin: false,
      },
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    let expected = concat!("{\n", "  \"ok\": true\n", "}");
    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_records_batch_delete_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      batch_delete_records_result: Err(OnspringError::Api {
        status_code: 500,
        message: "Failed to delete records".to_string(),
      }),
      ..Default::default()
    };

    let command = RecordsCommand::BatchDelete {
      body: BodySource {
        json: Some(r#"{"appId":1,"recordIds":[10,20]}"#.to_string()),
        file: None,
        stdin: false,
      },
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime(
        "API request failed (500): Failed to delete records"
      ))
    );
  }
}
