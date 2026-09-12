use std::io::Write;

use crate::cli::FieldsCommand;
use crate::client::OnspringRunner;
use crate::error::CliResult;
use crate::output::{CollectionOutput, FieldOutput, PagedOutput, write_json};
use crate::validation::{paging_request, validate_ids, validate_positive_i32};

pub async fn handle<C: OnspringRunner, W: Write>(
  command: &FieldsCommand,
  client: &C,
  writer: &mut W,
  pretty: bool,
) -> CliResult<()> {
  match command {
    FieldsCommand::List { app_id, paging } => {
      validate_positive_i32(*app_id, "app_id")?;
      let paging = paging_request(paging)?;
      let response = client.list_fields(*app_id, paging).await?;
      let output: PagedOutput<FieldOutput> = response.into();
      write_json(writer, &output, pretty)?;
    }
    FieldsCommand::Get { field_id } => {
      validate_positive_i32(*field_id, "field_id")?;
      let response = client.get_field(*field_id).await?;
      let output: FieldOutput = response.into();
      write_json(writer, &output, pretty)?;
    }
    FieldsCommand::BatchGet { ids } => {
      validate_ids(ids, "ids", Some(100))?;
      let response = client.batch_get_fields(ids).await?;
      let output: CollectionOutput<FieldOutput> = response.into();
      write_json(writer, &output, pretty)?;
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use onspring::{
    CollectionResponse, Field, FormulaOutputType, ListFieldValue, Multiplicity, OnspringError,
    PagedResponse,
  };
  use uuid::Uuid;

  use crate::cli::PagingArgs;
  use crate::client::testing::MockClient;
  use crate::error::CliError;

  fn test_field(id: i32, app_id: i32, name: &str) -> Field {
    Field {
      id,
      app_id,
      name: Some(name.to_string()),
      field_type: Some("Text".to_string()),
      status: Some("Enabled".to_string()),
      is_required: false,
      is_unique: false,
      multiplicity: None,
      list_id: None,
      values: None,
      output_type: None,
      referenced_app_id: None,
    }
  }

  #[tokio::test]
  async fn handle_when_fields_list_succeeds_it_should_write_paged_output() {
    let mock_client = MockClient {
      list_fields_result: Ok(PagedResponse {
        page_number: Some(1),
        page_size: Some(50),
        total_pages: Some(1),
        total_records: Some(1),
        items: Some(vec![test_field(1, 10, "Test Field")]),
      }),
      ..Default::default()
    };

    let command = FieldsCommand::List {
      app_id: 10,
      paging: PagingArgs::default(),
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    assert_eq!(
      written.trim(),
      r#"{"pageNumber":1,"pageSize":50,"totalPages":1,"totalRecords":1,"items":[{"id":1,"appId":10,"name":"Test Field","type":"Text","status":"Enabled","isRequired":false,"isUnique":false,"multiplicity":null,"listId":null,"values":null,"outputType":null,"referencedAppId":null}]}"#
    );
    assert_eq!(*mock_client.list_fields_app_id.lock().unwrap(), Some(10));
    assert!(
      mock_client
        .list_fields_paging
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .is_none()
    );
  }

  #[tokio::test]
  async fn handle_when_fields_list_with_paging_args_it_should_pass_paging_request() {
    let mock_client = MockClient {
      list_fields_result: Ok(PagedResponse {
        page_number: Some(2),
        page_size: Some(10),
        total_pages: Some(5),
        total_records: Some(50),
        items: Some(vec![]),
      }),
      ..Default::default()
    };

    let command = FieldsCommand::List {
      app_id: 10,
      paging: PagingArgs {
        page_number: Some(2),
        page_size: Some(10),
      },
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    assert_eq!(*mock_client.list_fields_app_id.lock().unwrap(), Some(10));
    let paging = mock_client
      .list_fields_paging
      .lock()
      .unwrap()
      .take()
      .unwrap()
      .unwrap();
    assert_eq!(paging.page_number, 2);
    assert_eq!(paging.page_size, 10);
  }

  #[tokio::test]
  async fn handle_when_fields_list_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      list_fields_result: Ok(PagedResponse {
        page_number: Some(1),
        page_size: Some(50),
        total_pages: Some(1),
        total_records: Some(1),
        items: Some(vec![test_field(1, 10, "Test Field")]),
      }),
      ..Default::default()
    };

    let command = FieldsCommand::List {
      app_id: 10,
      paging: PagingArgs::default(),
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
      "      \"id\": 1,\n",
      "      \"appId\": 10,\n",
      "      \"name\": \"Test Field\",\n",
      "      \"type\": \"Text\",\n",
      "      \"status\": \"Enabled\",\n",
      "      \"isRequired\": false,\n",
      "      \"isUnique\": false,\n",
      "      \"multiplicity\": null,\n",
      "      \"listId\": null,\n",
      "      \"values\": null,\n",
      "      \"outputType\": null,\n",
      "      \"referencedAppId\": null\n",
      "    }\n",
      "  ]\n",
      "}"
    );

    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_fields_list_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      list_fields_result: Err(OnspringError::Api {
        status_code: 401,
        message: "Unauthorized".to_string(),
      }),
      ..Default::default()
    };

    let command = FieldsCommand::List {
      app_id: 10,
      paging: PagingArgs::default(),
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
  async fn handle_when_fields_list_has_invalid_app_id_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FieldsCommand::List {
      app_id: 0,
      paging: PagingArgs::default(),
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("app_id must be greater than 0."))
    );
    assert!(buffer.is_empty());
    assert!(mock_client.list_fields_app_id.lock().unwrap().is_none());
  }

  #[tokio::test]
  async fn handle_when_fields_list_has_invalid_paging_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FieldsCommand::List {
      app_id: 10,
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
    assert!(buffer.is_empty());
    assert!(mock_client.list_fields_paging.lock().unwrap().is_none());
  }

  #[tokio::test]
  async fn handle_when_fields_get_succeeds_it_should_write_field_output() {
    let mock_client = MockClient {
      get_field_result: Ok(Field {
        id: 123,
        app_id: 456,
        name: Some("Status Field".to_string()),
        field_type: Some("List".to_string()),
        status: Some("Enabled".to_string()),
        is_required: true,
        is_unique: false,
        multiplicity: Some(Multiplicity::SingleSelect),
        list_id: Some(789),
        values: Some(vec![ListFieldValue {
          id: Uuid::nil(),
          name: "Active".to_string(),
          sort_order: 1,
          numeric_value: Some(1.0),
          color: Some("#00ff00".to_string()),
        }]),
        output_type: Some(FormulaOutputType::Text),
        referenced_app_id: Some(999),
      }),
      ..Default::default()
    };

    let command = FieldsCommand::Get { field_id: 123 };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    assert_eq!(
      written.trim(),
      r##"{"id":123,"appId":456,"name":"Status Field","type":"List","status":"Enabled","isRequired":true,"isUnique":false,"multiplicity":"SingleSelect","listId":789,"values":[{"id":"00000000-0000-0000-0000-000000000000","name":"Active","sortOrder":1,"numericValue":1.0,"color":"#00ff00"}],"outputType":"Text","referencedAppId":999}"##
    );
    assert_eq!(*mock_client.get_field_id.lock().unwrap(), Some(123));
  }

  #[tokio::test]
  async fn handle_when_fields_get_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      get_field_result: Ok(test_field(123, 456, "Test Field")),
      ..Default::default()
    };

    let command = FieldsCommand::Get { field_id: 123 };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    let expected = concat!(
      "{\n",
      "  \"id\": 123,\n",
      "  \"appId\": 456,\n",
      "  \"name\": \"Test Field\",\n",
      "  \"type\": \"Text\",\n",
      "  \"status\": \"Enabled\",\n",
      "  \"isRequired\": false,\n",
      "  \"isUnique\": false,\n",
      "  \"multiplicity\": null,\n",
      "  \"listId\": null,\n",
      "  \"values\": null,\n",
      "  \"outputType\": null,\n",
      "  \"referencedAppId\": null\n",
      "}"
    );

    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_fields_get_with_invalid_id_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FieldsCommand::Get { field_id: 0 };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("field_id must be greater than 0."))
    );
    assert!(buffer.is_empty());
    assert_eq!(*mock_client.get_field_id.lock().unwrap(), None);
  }

  #[tokio::test]
  async fn handle_when_fields_get_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      get_field_result: Err(OnspringError::Api {
        status_code: 404,
        message: "Field not found".to_string(),
      }),
      ..Default::default()
    };

    let command = FieldsCommand::Get { field_id: 123 };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime("API request failed (404): Field not found"))
    );
    assert!(buffer.is_empty());
  }

  #[tokio::test]
  async fn handle_when_fields_batch_get_succeeds_it_should_write_collection_output() {
    let mock_client = MockClient {
      batch_get_fields_result: Ok(CollectionResponse {
        count: Some(2),
        items: Some(vec![
          test_field(1, 10, "Field 1"),
          test_field(2, 10, "Field 2"),
        ]),
      }),
      ..Default::default()
    };

    let command = FieldsCommand::BatchGet { ids: vec![1, 2] };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    assert_eq!(
      written.trim(),
      r#"{"count":2,"items":[{"id":1,"appId":10,"name":"Field 1","type":"Text","status":"Enabled","isRequired":false,"isUnique":false,"multiplicity":null,"listId":null,"values":null,"outputType":null,"referencedAppId":null},{"id":2,"appId":10,"name":"Field 2","type":"Text","status":"Enabled","isRequired":false,"isUnique":false,"multiplicity":null,"listId":null,"values":null,"outputType":null,"referencedAppId":null}]}"#
    );
    assert_eq!(
      *mock_client.batch_get_fields_ids.lock().unwrap(),
      Some(vec![1, 2])
    );
  }

  #[tokio::test]
  async fn handle_when_fields_batch_get_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      batch_get_fields_result: Ok(CollectionResponse {
        count: Some(1),
        items: Some(vec![test_field(1, 10, "Field 1")]),
      }),
      ..Default::default()
    };

    let command = FieldsCommand::BatchGet { ids: vec![1] };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    let expected = concat!(
      "{\n",
      "  \"count\": 1,\n",
      "  \"items\": [\n",
      "    {\n",
      "      \"id\": 1,\n",
      "      \"appId\": 10,\n",
      "      \"name\": \"Field 1\",\n",
      "      \"type\": \"Text\",\n",
      "      \"status\": \"Enabled\",\n",
      "      \"isRequired\": false,\n",
      "      \"isUnique\": false,\n",
      "      \"multiplicity\": null,\n",
      "      \"listId\": null,\n",
      "      \"values\": null,\n",
      "      \"outputType\": null,\n",
      "      \"referencedAppId\": null\n",
      "    }\n",
      "  ]\n",
      "}"
    );

    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_fields_batch_get_with_empty_ids_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FieldsCommand::BatchGet { ids: vec![] };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Err(CliError::usage("ids cannot be empty.")));
    assert!(buffer.is_empty());
    assert_eq!(*mock_client.batch_get_fields_ids.lock().unwrap(), None);
  }

  #[tokio::test]
  async fn handle_when_fields_batch_get_with_invalid_ids_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FieldsCommand::BatchGet {
      ids: vec![1, 0, 2],
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("ids must only contain values > 0."))
    );
    assert!(buffer.is_empty());
    assert_eq!(*mock_client.batch_get_fields_ids.lock().unwrap(), None);
  }

  #[tokio::test]
  async fn handle_when_fields_batch_get_with_too_many_ids_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FieldsCommand::BatchGet {
      ids: vec![1; 101],
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage(
        "ids cannot contain more than 100 values."
      ))
    );
    assert!(buffer.is_empty());
    assert_eq!(*mock_client.batch_get_fields_ids.lock().unwrap(), None);
  }

  #[tokio::test]
  async fn handle_when_fields_batch_get_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      batch_get_fields_result: Err(OnspringError::Api {
        status_code: 500,
        message: "".to_string(),
      }),
      ..Default::default()
    };

    let command = FieldsCommand::BatchGet { ids: vec![1, 2] };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime("API request failed with status 500."))
    );
    assert!(buffer.is_empty());
  }
}

