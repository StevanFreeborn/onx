use std::io::Write;

use onspring::{DataFormat, ReportDataType};

use crate::cli::{ReportDataTypeArg, ReportsCommand};
use crate::client::OnspringRunner;
use crate::error::CliResult;
use crate::output::{PagedOutput, ReportDataOutput, ReportInfoOutput, write_json};
use crate::validation::{paging_request, validate_positive_i32};

impl From<ReportDataTypeArg> for ReportDataType {
  fn from(value: ReportDataTypeArg) -> Self {
    match value {
      ReportDataTypeArg::ReportData => ReportDataType::ReportData,
      ReportDataTypeArg::ChartData => ReportDataType::ChartData,
    }
  }
}

pub async fn handle<C: OnspringRunner, W: Write>(
  command: &ReportsCommand,
  client: &C,
  writer: &mut W,
  pretty: bool,
) -> CliResult<()> {
  match command {
    ReportsCommand::Get {
      report_id,
      data_format,
      data_type,
    } => {
      validate_positive_i32(*report_id, "report_id")?;
      let data_format = data_format.map(DataFormat::from);
      let data_type = data_type.map(ReportDataType::from);
      let response = client.get_report(*report_id, data_format, data_type).await?;
      let output: ReportDataOutput = response.into();
      write_json(writer, &output, pretty)?;
    }
    ReportsCommand::List { app_id, paging } => {
      validate_positive_i32(*app_id, "app_id")?;
      let paging = paging_request(paging)?;
      let response = client.list_reports(*app_id, paging).await?;
      let output: PagedOutput<ReportInfoOutput> = response.into();
      write_json(writer, &output, pretty)?;
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use onspring::{OnspringError, PagedResponse, ReportData, ReportInfo, ReportRow};
  use serde_json::json;

  use crate::cli::{DataFormatArg, PagingArgs};
  use crate::client::testing::MockClient;
  use crate::error::CliError;

  #[tokio::test]
  async fn handle_when_reports_get_succeeds_it_should_write_report_data_output() {
    let mock_client = MockClient {
      get_report_result: Ok(ReportData {
        columns: Some(vec!["Col1".to_string(), "Col2".to_string()]),
        rows: Some(vec![ReportRow {
          record_id: Some(10),
          cells: Some(vec![json!("Val1"), json!(42)]),
        }]),
      }),
      ..Default::default()
    };

    let command = ReportsCommand::Get {
      report_id: 123,
      data_format: None,
      data_type: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();
    assert_eq!(
      written.trim(),
      r#"{"columns":["Col1","Col2"],"rows":[{"recordId":10,"cells":["Val1",42]}]}"#
    );
    assert_eq!(*mock_client.get_report_report_id.lock().unwrap(), Some(123));
    assert_eq!(
      *mock_client.get_report_data_format.lock().unwrap(),
      Some(None)
    );
    assert_eq!(*mock_client.get_report_data_type.lock().unwrap(), Some(None));
  }

  #[tokio::test]
  async fn handle_when_reports_get_with_format_and_type_it_should_pass_mapped_arguments() {
    let mock_client = MockClient {
      get_report_result: Ok(ReportData {
        columns: Some(vec!["Header".to_string()]),
        rows: Some(vec![]),
      }),
      ..Default::default()
    };

    let command = ReportsCommand::Get {
      report_id: 456,
      data_format: Some(DataFormatArg::Formatted),
      data_type: Some(ReportDataTypeArg::ChartData),
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    assert_eq!(*mock_client.get_report_report_id.lock().unwrap(), Some(456));
    assert_eq!(
      *mock_client.get_report_data_format.lock().unwrap(),
      Some(Some(DataFormat::Formatted))
    );
    assert_eq!(
      *mock_client.get_report_data_type.lock().unwrap(),
      Some(Some(ReportDataType::ChartData))
    );
  }

  #[tokio::test]
  async fn handle_when_reports_get_with_raw_and_report_data_it_should_pass_mapped_arguments() {
    let mock_client = MockClient {
      get_report_result: Ok(ReportData {
        columns: None,
        rows: None,
      }),
      ..Default::default()
    };

    let command = ReportsCommand::Get {
      report_id: 789,
      data_format: Some(DataFormatArg::Raw),
      data_type: Some(ReportDataTypeArg::ReportData),
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    assert_eq!(*mock_client.get_report_report_id.lock().unwrap(), Some(789));
    assert_eq!(
      *mock_client.get_report_data_format.lock().unwrap(),
      Some(Some(DataFormat::Raw))
    );
    assert_eq!(
      *mock_client.get_report_data_type.lock().unwrap(),
      Some(Some(ReportDataType::ReportData))
    );
  }

  #[tokio::test]
  async fn handle_when_reports_get_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      get_report_result: Ok(ReportData {
        columns: Some(vec!["Col".to_string()]),
        rows: Some(vec![ReportRow {
          record_id: Some(1),
          cells: Some(vec![json!("Val")]),
        }]),
      }),
      ..Default::default()
    };

    let command = ReportsCommand::Get {
      report_id: 1,
      data_format: None,
      data_type: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();
    let expected = concat!(
      "{\n",
      "  \"columns\": [\n",
      "    \"Col\"\n",
      "  ],\n",
      "  \"rows\": [\n",
      "    {\n",
      "      \"recordId\": 1,\n",
      "      \"cells\": [\n",
      "        \"Val\"\n",
      "      ]\n",
      "    }\n",
      "  ]\n",
      "}"
    );

    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_reports_get_has_invalid_report_id_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = ReportsCommand::Get {
      report_id: 0,
      data_format: None,
      data_type: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("report_id must be greater than 0."))
    );
    assert!(buffer.is_empty());
    assert!(mock_client.get_report_report_id.lock().unwrap().is_none());
  }

  #[tokio::test]
  async fn handle_when_reports_get_has_negative_report_id_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = ReportsCommand::Get {
      report_id: -5,
      data_format: None,
      data_type: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("report_id must be greater than 0."))
    );
    assert!(buffer.is_empty());
    assert!(mock_client.get_report_report_id.lock().unwrap().is_none());
  }

  #[tokio::test]
  async fn handle_when_reports_get_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      get_report_result: Err(OnspringError::Api {
        status_code: 404,
        message: "Report not found".to_string(),
      }),
      ..Default::default()
    };

    let command = ReportsCommand::Get {
      report_id: 999,
      data_format: None,
      data_type: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime("API request failed (404): Report not found"))
    );
    assert!(buffer.is_empty());
  }

  #[tokio::test]
  async fn handle_when_reports_list_succeeds_it_should_write_paged_output() {
    let mock_client = MockClient {
      list_reports_result: Ok(PagedResponse {
        page_number: Some(1),
        page_size: Some(50),
        total_pages: Some(1),
        total_records: Some(1),
        items: Some(vec![ReportInfo {
          app_id: 10,
          id: 100,
          name: Some("Open Tasks".to_string()),
          description: Some("Report of open tasks".to_string()),
        }]),
      }),
      ..Default::default()
    };

    let command = ReportsCommand::List {
      app_id: 10,
      paging: PagingArgs::default(),
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();
    assert_eq!(
      written.trim(),
      r#"{"pageNumber":1,"pageSize":50,"totalPages":1,"totalRecords":1,"items":[{"appId":10,"id":100,"name":"Open Tasks","description":"Report of open tasks"}]}"#
    );
    assert_eq!(*mock_client.list_reports_app_id.lock().unwrap(), Some(10));
    assert!(
      mock_client
        .list_reports_paging
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .is_none()
    );
  }

  #[tokio::test]
  async fn handle_when_reports_list_with_paging_args_it_should_pass_paging_request() {
    let mock_client = MockClient {
      list_reports_result: Ok(PagedResponse {
        page_number: Some(2),
        page_size: Some(15),
        total_pages: Some(3),
        total_records: Some(40),
        items: Some(vec![]),
      }),
      ..Default::default()
    };

    let command = ReportsCommand::List {
      app_id: 10,
      paging: PagingArgs {
        page_number: Some(2),
        page_size: Some(15),
      },
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let paging = mock_client
      .list_reports_paging
      .lock()
      .unwrap()
      .take()
      .unwrap()
      .unwrap();
    assert_eq!(paging.page_number, 2);
    assert_eq!(paging.page_size, 15);
  }

  #[tokio::test]
  async fn handle_when_reports_list_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      list_reports_result: Ok(PagedResponse {
        page_number: Some(1),
        page_size: Some(50),
        total_pages: Some(1),
        total_records: Some(1),
        items: Some(vec![ReportInfo {
          app_id: 10,
          id: 100,
          name: Some("Open Tasks".to_string()),
          description: None,
        }]),
      }),
      ..Default::default()
    };

    let command = ReportsCommand::List {
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
      "      \"appId\": 10,\n",
      "      \"id\": 100,\n",
      "      \"name\": \"Open Tasks\",\n",
      "      \"description\": null\n",
      "    }\n",
      "  ]\n",
      "}"
    );

    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_reports_list_has_invalid_app_id_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = ReportsCommand::List {
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
    assert!(mock_client.list_reports_app_id.lock().unwrap().is_none());
  }

  #[tokio::test]
  async fn handle_when_reports_list_has_invalid_paging_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = ReportsCommand::List {
      app_id: 10,
      paging: PagingArgs {
        page_number: Some(-1),
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
    assert!(mock_client.list_reports_paging.lock().unwrap().is_none());
  }

  #[tokio::test]
  async fn handle_when_reports_list_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      list_reports_result: Err(OnspringError::Api {
        status_code: 403,
        message: "Forbidden".to_string(),
      }),
      ..Default::default()
    };

    let command = ReportsCommand::List {
      app_id: 10,
      paging: PagingArgs::default(),
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime("API request failed (403): Forbidden"))
    );
    assert!(buffer.is_empty());
  }
}
