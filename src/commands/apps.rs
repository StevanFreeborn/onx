use std::io::Write;

use crate::cli::AppsCommand;
use crate::client::OnspringRunner;
use crate::error::CliResult;
use crate::output::{AppOutput, CollectionOutput, PagedOutput, write_json};
use crate::validation::{paging_request, validate_ids, validate_positive_i32};

pub async fn handle<C: OnspringRunner, W: Write>(
  command: &AppsCommand,
  client: &C,
  writer: &mut W,
  pretty: bool,
) -> CliResult<()> {
  match command {
    AppsCommand::List(paging_args) => {
      let paging = paging_request(paging_args)?;
      let response = client.list_apps(paging).await?;
      let output: PagedOutput<AppOutput> = response.into();
      write_json(writer, &output, pretty)?;
    }
    AppsCommand::Get { app_id } => {
      validate_positive_i32(*app_id, "app_id")?;
      let response = client.get_app(*app_id).await?;
      let output: AppOutput = response.into();
      write_json(writer, &output, pretty)?;
    }
    AppsCommand::BatchGet { ids } => {
      validate_ids(ids, "ids", Some(100))?;
      let response = client.batch_get_apps(ids).await?;
      let output: CollectionOutput<AppOutput> = response.into();
      write_json(writer, &output, pretty)?;
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use onspring::{App, CollectionResponse, OnspringError, PagedResponse};

  use crate::cli::PagingArgs;
  use crate::client::testing::MockClient;
  use crate::error::CliError;

  #[tokio::test]
  async fn handle_when_apps_list_succeeds_it_should_write_paged_output() {
    let mock_client = MockClient {
      list_apps_result: Ok(PagedResponse {
        page_number: Some(1),
        page_size: Some(50),
        total_pages: Some(1),
        total_records: Some(1),
        items: Some(vec![App {
          href: Some("https://api.onspring.com/apps/1".to_string()),
          id: 1,
          name: Some("Test App".to_string()),
        }]),
      }),
      ..Default::default()
    };

    let command = AppsCommand::List(PagingArgs::default());
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    assert_eq!(
      written.trim(),
      r#"{"pageNumber":1,"pageSize":50,"totalPages":1,"totalRecords":1,"items":[{"href":"https://api.onspring.com/apps/1","id":1,"name":"Test App"}]}"#
    );
    assert!(
      mock_client
        .list_apps_paging
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .is_none()
    );
  }

  #[tokio::test]
  async fn handle_when_apps_list_with_paging_args_it_should_pass_paging_request() {
    let mock_client = MockClient {
      list_apps_result: Ok(PagedResponse {
        page_number: Some(2),
        page_size: Some(10),
        total_pages: Some(5),
        total_records: Some(50),
        items: Some(vec![]),
      }),
      ..Default::default()
    };

    let command = AppsCommand::List(PagingArgs {
      page_number: Some(2),
      page_size: Some(10),
    });
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let paging = mock_client
      .list_apps_paging
      .lock()
      .unwrap()
      .take()
      .unwrap()
      .unwrap();
    assert_eq!(paging.page_number, 2);
    assert_eq!(paging.page_size, 10);
  }

  #[tokio::test]
  async fn handle_when_apps_list_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      list_apps_result: Ok(PagedResponse {
        page_number: Some(1),
        page_size: Some(50),
        total_pages: Some(1),
        total_records: Some(1),
        items: Some(vec![App {
          href: Some("https://api.onspring.com/apps/1".to_string()),
          id: 1,
          name: Some("Test App".to_string()),
        }]),
      }),
      ..Default::default()
    };

    let command = AppsCommand::List(PagingArgs::default());
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
      "      \"href\": \"https://api.onspring.com/apps/1\",\n",
      "      \"id\": 1,\n",
      "      \"name\": \"Test App\"\n",
      "    }\n",
      "  ]\n",
      "}"
    );

    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_apps_list_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      list_apps_result: Err(OnspringError::Api {
        status_code: 401,
        message: "Unauthorized".to_string(),
      }),
      ..Default::default()
    };

    let command = AppsCommand::List(PagingArgs::default());
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime("API request failed (401): Unauthorized"))
    );
    assert!(buffer.is_empty());
  }

  #[tokio::test]
  async fn handle_when_apps_list_has_invalid_paging_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = AppsCommand::List(PagingArgs {
      page_number: Some(0),
      page_size: None,
    });
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("page_number must be greater than 0."))
    );
    assert!(buffer.is_empty());
    assert!(mock_client.list_apps_paging.lock().unwrap().is_none());
  }

  #[tokio::test]
  async fn handle_when_apps_get_succeeds_it_should_write_app_output() {
    let mock_client = MockClient {
      get_app_result: Ok(App {
        href: Some("https://api.onspring.com/apps/123".to_string()),
        id: 123,
        name: Some("Test App".to_string()),
      }),
      ..Default::default()
    };

    let command = AppsCommand::Get { app_id: 123 };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    assert_eq!(
      written.trim(),
      r#"{"href":"https://api.onspring.com/apps/123","id":123,"name":"Test App"}"#
    );
    assert_eq!(*mock_client.get_app_id.lock().unwrap(), Some(123));
  }

  #[tokio::test]
  async fn handle_when_apps_get_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      get_app_result: Ok(App {
        href: Some("https://api.onspring.com/apps/123".to_string()),
        id: 123,
        name: Some("Test App".to_string()),
      }),
      ..Default::default()
    };

    let command = AppsCommand::Get { app_id: 123 };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    let expected = concat!(
      "{\n",
      "  \"href\": \"https://api.onspring.com/apps/123\",\n",
      "  \"id\": 123,\n",
      "  \"name\": \"Test App\"\n",
      "}"
    );

    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_apps_get_with_invalid_id_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = AppsCommand::Get { app_id: 0 };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("app_id must be greater than 0."))
    );
    assert!(buffer.is_empty());
    assert_eq!(*mock_client.get_app_id.lock().unwrap(), None);
  }

  #[tokio::test]
  async fn handle_when_apps_get_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      get_app_result: Err(OnspringError::Api {
        status_code: 404,
        message: "App not found".to_string(),
      }),
      ..Default::default()
    };

    let command = AppsCommand::Get { app_id: 123 };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime("API request failed (404): App not found"))
    );
    assert!(buffer.is_empty());
  }

  #[tokio::test]
  async fn handle_when_apps_batch_get_succeeds_it_should_write_collection_output() {
    let mock_client = MockClient {
      batch_get_apps_result: Ok(CollectionResponse {
        count: Some(2),
        items: Some(vec![
          App {
            href: Some("https://api.onspring.com/apps/1".to_string()),
            id: 1,
            name: Some("App 1".to_string()),
          },
          App {
            href: Some("https://api.onspring.com/apps/2".to_string()),
            id: 2,
            name: Some("App 2".to_string()),
          },
        ]),
      }),
      ..Default::default()
    };

    let command = AppsCommand::BatchGet { ids: vec![1, 2] };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    assert_eq!(
      written.trim(),
      r#"{"count":2,"items":[{"href":"https://api.onspring.com/apps/1","id":1,"name":"App 1"},{"href":"https://api.onspring.com/apps/2","id":2,"name":"App 2"}]}"#
    );
    assert_eq!(
      *mock_client.batch_get_apps_ids.lock().unwrap(),
      Some(vec![1, 2])
    );
  }

  #[tokio::test]
  async fn handle_when_apps_batch_get_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      batch_get_apps_result: Ok(CollectionResponse {
        count: Some(1),
        items: Some(vec![App {
          href: Some("https://api.onspring.com/apps/1".to_string()),
          id: 1,
          name: Some("App 1".to_string()),
        }]),
      }),
      ..Default::default()
    };

    let command = AppsCommand::BatchGet { ids: vec![1] };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();

    let expected = concat!(
      "{\n",
      "  \"count\": 1,\n",
      "  \"items\": [\n",
      "    {\n",
      "      \"href\": \"https://api.onspring.com/apps/1\",\n",
      "      \"id\": 1,\n",
      "      \"name\": \"App 1\"\n",
      "    }\n",
      "  ]\n",
      "}"
    );

    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_apps_batch_get_with_empty_ids_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = AppsCommand::BatchGet { ids: vec![] };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Err(CliError::usage("ids cannot be empty.")));
    assert!(buffer.is_empty());
    assert_eq!(*mock_client.batch_get_apps_ids.lock().unwrap(), None);
  }

  #[tokio::test]
  async fn handle_when_apps_batch_get_with_invalid_ids_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = AppsCommand::BatchGet {
      ids: vec![1, 0, 2],
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("ids must only contain values > 0."))
    );
    assert!(buffer.is_empty());
    assert_eq!(*mock_client.batch_get_apps_ids.lock().unwrap(), None);
  }

  #[tokio::test]
  async fn handle_when_apps_batch_get_with_too_many_ids_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = AppsCommand::BatchGet {
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
    assert_eq!(*mock_client.batch_get_apps_ids.lock().unwrap(), None);
  }

  #[tokio::test]
  async fn handle_when_apps_batch_get_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      batch_get_apps_result: Err(OnspringError::Api {
        status_code: 500,
        message: "".to_string(),
      }),
      ..Default::default()
    };

    let command = AppsCommand::BatchGet { ids: vec![1, 2] };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime("API request failed with status 500."))
    );
    assert!(buffer.is_empty());
  }
}
