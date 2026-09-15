use std::io::{Read, Write};

use chrono::{DateTime, Utc};
use onspring::SaveFileRequest;

use crate::cli::FilesCommand;
use crate::client::OnspringRunner;
use crate::error::{CliError, CliResult};
use crate::output::{CreatedWithIdResponseOutput, FileInfoOutput, SuccessResponse, write_json};
use crate::validation::validate_positive_i32;

pub async fn handle<C: OnspringRunner, W: Write>(
  command: &FilesCommand,
  client: &C,
  writer: &mut W,
  pretty: bool,
) -> CliResult<()> {
  handle_with_reader(command, client, writer, std::io::stdin(), pretty).await
}

pub async fn handle_with_reader<C: OnspringRunner, W: Write, R: Read>(
  command: &FilesCommand,
  client: &C,
  writer: &mut W,
  mut reader: R,
  pretty: bool,
) -> CliResult<()> {
  match command {
    FilesCommand::Info {
      record_id,
      field_id,
      file_id,
    } => {
      validate_positive_i32(*record_id, "record_id")?;
      validate_positive_i32(*field_id, "field_id")?;
      validate_positive_i32(*file_id, "file_id")?;

      let response = client
        .get_file_info(*record_id, *field_id, *file_id)
        .await?;
      let output: FileInfoOutput = response.into();
      write_json(writer, &output, pretty)?;
    }
    FilesCommand::Get {
      record_id,
      field_id,
      file_id,
      output,
    } => {
      validate_positive_i32(*record_id, "record_id")?;
      validate_positive_i32(*field_id, "field_id")?;
      validate_positive_i32(*file_id, "file_id")?;

      let response = client.get_file(*record_id, *field_id, *file_id).await?;

      if let Some(output_path) = output {
        std::fs::write(output_path, &response.data).map_err(|e| {
          CliError::runtime(format!(
            "Failed to write file to '{}': {e}",
            output_path.display()
          ))
        })?;
      } else {
        writer
          .write_all(&response.data)
          .map_err(|_| CliError::runtime("Failed to write output"))?;
      }
    }
    FilesCommand::Upload {
      record_id,
      field_id,
      input,
      stdin,
      file_name,
      content_type,
      notes,
      modified_date,
    } => {
      validate_positive_i32(*record_id, "record_id")?;
      validate_positive_i32(*field_id, "field_id")?;

      if input.is_none() && !*stdin {
        return Err(CliError::usage(
          "A file source is required. Provide --input or --stdin.",
        ));
      }

      if input.is_some() && *stdin {
        return Err(CliError::usage(
          "Only one file source may be specified (--input or --stdin).",
        ));
      }

      let (file_bytes, resolved_file_name) = if let Some(path) = input {
        let bytes = std::fs::read(path).map_err(|e| {
          CliError::usage(format!("Failed to read file '{}': {e}", path.display()))
        })?;
        let default_name = path
          .file_name()
          .and_then(|n| n.to_str())
          .map(String::from);
        let name = file_name.clone().or(default_name).ok_or_else(|| {
          CliError::usage("A file name is required. Provide --name.")
        })?;
        (bytes, name)
      } else {
        let mut buffer = Vec::new();
        reader
          .read_to_end(&mut buffer)
          .map_err(|e| CliError::runtime(format!("Failed to read from stdin: {e}")))?;
        let name = file_name
          .clone()
          .ok_or_else(|| CliError::usage("A file name is required when reading from stdin. Provide --name."))?;
        (buffer, name)
      };

      let resolved_content_type = match content_type {
        Some(ct) => ct.clone(),
        None => mime_guess::from_path(&resolved_file_name)
          .first_or_octet_stream()
          .to_string(),
      };

      let parsed_modified_date = match modified_date {
        Some(date_str) => Some(DateTime::parse_from_rfc3339(date_str).map_err(|e| {
          CliError::usage(format!(
            "Invalid modified_date '{date_str}'. Expected RFC3339 format: {e}"
          ))
        })?.with_timezone(&Utc)),
        None => None,
      };

      let request = SaveFileRequest {
        record_id: *record_id,
        field_id: *field_id,
        notes: notes.clone(),
        modified_date: parsed_modified_date,
        file_name: resolved_file_name,
        file_data: file_bytes,
        content_type: resolved_content_type,
      };

      let response = client.upload_file(request).await?;
      let output: CreatedWithIdResponseOutput = response.into();
      write_json(writer, &output, pretty)?;
    }
    FilesCommand::Delete {
      record_id,
      field_id,
      file_id,
    } => {
      validate_positive_i32(*record_id, "record_id")?;
      validate_positive_i32(*field_id, "field_id")?;
      validate_positive_i32(*file_id, "file_id")?;

      client.delete_file(*record_id, *field_id, *file_id).await?;
      write_json(writer, &SuccessResponse { ok: true }, pretty)?;
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

  use bytes::Bytes;
  use chrono::TimeZone;
  use onspring::{CreatedWithIdResponse, FileInfo, FileResponse, OnspringError};

  use crate::client::testing::MockClient;
  use crate::error::CliError;

  // --- Info tests ---

  #[tokio::test]
  async fn handle_when_files_info_succeeds_it_should_write_file_info_output() {
    let created = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
    let modified = Utc.with_ymd_and_hms(2026, 1, 16, 12, 0, 0).unwrap();

    let mock_client = MockClient {
      get_file_info_result: Ok(FileInfo {
        file_type: Some("Attachment".to_string()),
        content_type: Some("application/pdf".to_string()),
        name: Some("test.pdf".to_string()),
        created_date: Some(created),
        modified_date: Some(modified),
        owner: Some("User 1".to_string()),
        notes: Some("Some notes".to_string()),
        file_href: Some("https://api.onspring.com/files/3/file".to_string()),
      }),
      ..Default::default()
    };

    let command = FilesCommand::Info {
      record_id: 1,
      field_id: 2,
      file_id: 3,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();
    assert_eq!(
      written.trim(),
      r#"{"type":"Attachment","contentType":"application/pdf","name":"test.pdf","createdDate":"2026-01-15T10:00:00Z","modifiedDate":"2026-01-16T12:00:00Z","owner":"User 1","notes":"Some notes","fileHref":"https://api.onspring.com/files/3/file"}"#
    );
    assert_eq!(*mock_client.get_file_info_record_id.lock().unwrap(), Some(1));
    assert_eq!(*mock_client.get_file_info_field_id.lock().unwrap(), Some(2));
    assert_eq!(*mock_client.get_file_info_file_id.lock().unwrap(), Some(3));
  }

  #[tokio::test]
  async fn handle_when_files_info_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      get_file_info_result: Ok(FileInfo {
        file_type: Some("Attachment".to_string()),
        content_type: Some("text/plain".to_string()),
        name: Some("note.txt".to_string()),
        created_date: None,
        modified_date: None,
        owner: None,
        notes: None,
        file_href: None,
      }),
      ..Default::default()
    };

    let command = FilesCommand::Info {
      record_id: 1,
      field_id: 2,
      file_id: 3,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();
    let expected = concat!(
      "{\n",
      "  \"type\": \"Attachment\",\n",
      "  \"contentType\": \"text/plain\",\n",
      "  \"name\": \"note.txt\",\n",
      "  \"createdDate\": null,\n",
      "  \"modifiedDate\": null,\n",
      "  \"owner\": null,\n",
      "  \"notes\": null,\n",
      "  \"fileHref\": null\n",
      "}"
    );
    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_files_info_with_invalid_record_id_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FilesCommand::Info {
      record_id: 0,
      field_id: 2,
      file_id: 3,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("record_id must be greater than 0."))
    );
    assert!(buffer.is_empty());
  }

  #[tokio::test]
  async fn handle_when_files_info_with_invalid_field_id_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FilesCommand::Info {
      record_id: 1,
      field_id: -1,
      file_id: 3,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("field_id must be greater than 0."))
    );
    assert!(buffer.is_empty());
  }

  #[tokio::test]
  async fn handle_when_files_info_with_invalid_file_id_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FilesCommand::Info {
      record_id: 1,
      field_id: 2,
      file_id: 0,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage("file_id must be greater than 0."))
    );
    assert!(buffer.is_empty());
  }

  #[tokio::test]
  async fn handle_when_files_info_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      get_file_info_result: Err(OnspringError::Api {
        status_code: 404,
        message: "File not found".to_string(),
      }),
      ..Default::default()
    };

    let command = FilesCommand::Info {
      record_id: 1,
      field_id: 2,
      file_id: 3,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime("API request failed (404): File not found"))
    );
    assert!(buffer.is_empty());
  }

  // --- Get tests ---

  #[tokio::test]
  async fn handle_when_files_get_without_output_flag_it_should_write_bytes_to_writer() {
    let file_bytes = b"Hello, World!";
    let mock_client = MockClient {
      get_file_result: Ok(FileResponse {
        content_type: Some("text/plain".to_string()),
        file_name: Some("hello.txt".to_string()),
        data: Bytes::from_static(file_bytes),
      }),
      ..Default::default()
    };

    let command = FilesCommand::Get {
      record_id: 10,
      field_id: 20,
      file_id: 30,
      output: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));
    assert_eq!(buffer, file_bytes);
    assert_eq!(*mock_client.get_file_record_id.lock().unwrap(), Some(10));
    assert_eq!(*mock_client.get_file_field_id.lock().unwrap(), Some(20));
    assert_eq!(*mock_client.get_file_file_id.lock().unwrap(), Some(30));
  }

  #[tokio::test]
  async fn handle_when_files_get_with_output_flag_it_should_write_to_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("downloaded.bin");
    let file_bytes = vec![1, 2, 3, 4, 5];

    let mock_client = MockClient {
      get_file_result: Ok(FileResponse {
        content_type: Some("application/octet-stream".to_string()),
        file_name: Some("data.bin".to_string()),
        data: Bytes::from(file_bytes.clone()),
      }),
      ..Default::default()
    };

    let command = FilesCommand::Get {
      record_id: 1,
      field_id: 2,
      file_id: 3,
      output: Some(file_path.clone()),
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));
    assert!(buffer.is_empty());
    let read_back = std::fs::read(&file_path).unwrap();
    assert_eq!(read_back, file_bytes);
  }

  #[tokio::test]
  async fn handle_when_files_get_with_invalid_output_path_it_should_return_runtime_error() {
    let mock_client = MockClient {
      get_file_result: Ok(FileResponse {
        content_type: None,
        file_name: None,
        data: Bytes::from_static(b"data"),
      }),
      ..Default::default()
    };

    let command = FilesCommand::Get {
      record_id: 1,
      field_id: 2,
      file_id: 3,
      output: Some(PathBuf::from("/nonexistent_dir_12345/sub/file.txt")),
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 1);
    assert!(err.message.contains("Failed to write file to"));
  }

  #[tokio::test]
  async fn handle_when_files_get_with_invalid_ids_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FilesCommand::Get {
      record_id: 0,
      field_id: 1,
      file_id: 1,
      output: None,
    };
    let mut buffer = Vec::new();
    assert_eq!(
      handle(&command, &mock_client, &mut buffer, false).await,
      Err(CliError::usage("record_id must be greater than 0."))
    );

    let command = FilesCommand::Get {
      record_id: 1,
      field_id: 0,
      file_id: 1,
      output: None,
    };
    assert_eq!(
      handle(&command, &mock_client, &mut buffer, false).await,
      Err(CliError::usage("field_id must be greater than 0."))
    );

    let command = FilesCommand::Get {
      record_id: 1,
      field_id: 1,
      file_id: 0,
      output: None,
    };
    assert_eq!(
      handle(&command, &mock_client, &mut buffer, false).await,
      Err(CliError::usage("file_id must be greater than 0."))
    );
  }

  #[tokio::test]
  async fn handle_when_files_get_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      get_file_result: Err(OnspringError::Api {
        status_code: 500,
        message: "Internal error".to_string(),
      }),
      ..Default::default()
    };

    let command = FilesCommand::Get {
      record_id: 1,
      field_id: 2,
      file_id: 3,
      output: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime(
        "API request failed (500): Internal error"
      ))
    );
  }

  // --- Upload tests ---

  #[tokio::test]
  async fn handle_when_files_upload_from_input_file_succeeds() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), b"Upload content").unwrap();

    let mock_client = MockClient {
      upload_file_result: Ok(CreatedWithIdResponse { id: 99 }),
      ..Default::default()
    };

    let command = FilesCommand::Upload {
      record_id: 1,
      field_id: 2,
      input: Some(temp_file.path().to_path_buf()),
      stdin: false,
      file_name: Some("custom_name.txt".to_string()),
      content_type: Some("text/custom".to_string()),
      notes: Some("Upload note".to_string()),
      modified_date: Some("2026-03-01T12:00:00Z".to_string()),
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();
    assert_eq!(written.trim(), r#"{"id":99}"#);

    let req = mock_client.upload_file_request.lock().unwrap().clone().unwrap();
    assert_eq!(req.record_id, 1);
    assert_eq!(req.field_id, 2);
    assert_eq!(req.file_name, "custom_name.txt");
    assert_eq!(req.content_type, "text/custom");
    assert_eq!(req.file_data, b"Upload content");
    assert_eq!(req.notes, Some("Upload note".to_string()));
    assert_eq!(
      req.modified_date,
      Some(Utc.with_ymd_and_hms(2026, 3, 1, 12, 0, 0).unwrap())
    );
  }

  #[tokio::test]
  async fn handle_when_files_upload_succeeds_pretty_it_should_write_pretty_json() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), b"content").unwrap();

    let mock_client = MockClient {
      upload_file_result: Ok(CreatedWithIdResponse { id: 101 }),
      ..Default::default()
    };

    let command = FilesCommand::Upload {
      record_id: 1,
      field_id: 2,
      input: Some(temp_file.path().to_path_buf()),
      stdin: false,
      file_name: Some("data.txt".to_string()),
      content_type: None,
      notes: None,
      modified_date: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();
    let expected = concat!("{\n", "  \"id\": 101\n", "}");
    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_files_upload_infers_file_name_and_content_type_from_input() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("report.csv");
    std::fs::write(&file_path, b"a,b,c\n1,2,3").unwrap();

    let mock_client = MockClient {
      upload_file_result: Ok(CreatedWithIdResponse { id: 50 }),
      ..Default::default()
    };

    let command = FilesCommand::Upload {
      record_id: 1,
      field_id: 2,
      input: Some(file_path),
      stdin: false,
      file_name: None,
      content_type: None,
      notes: None,
      modified_date: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let req = mock_client.upload_file_request.lock().unwrap().clone().unwrap();
    assert_eq!(req.file_name, "report.csv");
    assert_eq!(req.content_type, "text/csv");
  }

  #[tokio::test]
  async fn handle_when_files_upload_reads_from_stdin() {
    let mock_client = MockClient {
      upload_file_result: Ok(CreatedWithIdResponse { id: 75 }),
      ..Default::default()
    };

    let command = FilesCommand::Upload {
      record_id: 5,
      field_id: 10,
      input: None,
      stdin: true,
      file_name: Some("data.json".to_string()),
      content_type: None,
      notes: None,
      modified_date: None,
    };
    let mut buffer = Vec::new();
    let stdin_bytes = b"{\"hello\":\"world\"}";

    let result = handle_with_reader(
      &command,
      &mock_client,
      &mut buffer,
      &stdin_bytes[..],
      false,
    )
    .await;

    assert_eq!(result, Ok(()));

    let req = mock_client.upload_file_request.lock().unwrap().clone().unwrap();
    assert_eq!(req.record_id, 5);
    assert_eq!(req.field_id, 10);
    assert_eq!(req.file_name, "data.json");
    assert_eq!(req.content_type, "application/json");
    assert_eq!(req.file_data, stdin_bytes);
  }

  #[tokio::test]
  async fn handle_when_files_upload_without_source_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FilesCommand::Upload {
      record_id: 1,
      field_id: 2,
      input: None,
      stdin: false,
      file_name: Some("file.txt".to_string()),
      content_type: None,
      notes: None,
      modified_date: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage(
        "A file source is required. Provide --input or --stdin."
      ))
    );
  }

  #[tokio::test]
  async fn handle_when_files_upload_with_both_sources_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FilesCommand::Upload {
      record_id: 1,
      field_id: 2,
      input: Some(PathBuf::from("test.txt")),
      stdin: true,
      file_name: Some("file.txt".to_string()),
      content_type: None,
      notes: None,
      modified_date: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::usage(
        "Only one file source may be specified (--input or --stdin)."
      ))
    );
  }

  #[tokio::test]
  async fn handle_when_files_upload_input_file_not_found_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FilesCommand::Upload {
      record_id: 1,
      field_id: 2,
      input: Some(PathBuf::from("nonexistent_file_12345.txt")),
      stdin: false,
      file_name: None,
      content_type: None,
      notes: None,
      modified_date: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 2);
    assert!(err.message.contains("Failed to read file"));
  }

  #[tokio::test]
  async fn handle_when_files_upload_stdin_missing_name_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FilesCommand::Upload {
      record_id: 1,
      field_id: 2,
      input: None,
      stdin: true,
      file_name: None,
      content_type: None,
      notes: None,
      modified_date: None,
    };
    let mut buffer = Vec::new();

    let result = handle_with_reader(
      &command,
      &mock_client,
      &mut buffer,
      std::io::empty(),
      false,
    )
    .await;

    assert_eq!(
      result,
      Err(CliError::usage(
        "A file name is required when reading from stdin. Provide --name."
      ))
    );
  }

  #[tokio::test]
  async fn handle_when_files_upload_invalid_modified_date_it_should_return_usage_error() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();

    let mock_client = MockClient::default();

    let command = FilesCommand::Upload {
      record_id: 1,
      field_id: 2,
      input: Some(temp_file.path().to_path_buf()),
      stdin: false,
      file_name: None,
      content_type: None,
      notes: None,
      modified_date: Some("not-a-date".to_string()),
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 2);
    assert!(err.message.contains("Invalid modified_date"));
  }

  #[tokio::test]
  async fn handle_when_files_upload_with_invalid_ids_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FilesCommand::Upload {
      record_id: 0,
      field_id: 2,
      input: Some(PathBuf::from("test.txt")),
      stdin: false,
      file_name: None,
      content_type: None,
      notes: None,
      modified_date: None,
    };
    let mut buffer = Vec::new();
    assert_eq!(
      handle(&command, &mock_client, &mut buffer, false).await,
      Err(CliError::usage("record_id must be greater than 0."))
    );

    let command = FilesCommand::Upload {
      record_id: 1,
      field_id: 0,
      input: Some(PathBuf::from("test.txt")),
      stdin: false,
      file_name: None,
      content_type: None,
      notes: None,
      modified_date: None,
    };
    assert_eq!(
      handle(&command, &mock_client, &mut buffer, false).await,
      Err(CliError::usage("field_id must be greater than 0."))
    );
  }

  #[tokio::test]
  async fn handle_when_files_upload_fails_it_should_return_mapped_cli_error() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();

    let mock_client = MockClient {
      upload_file_result: Err(OnspringError::Api {
        status_code: 400,
        message: "Invalid file type".to_string(),
      }),
      ..Default::default()
    };

    let command = FilesCommand::Upload {
      record_id: 1,
      field_id: 2,
      input: Some(temp_file.path().to_path_buf()),
      stdin: false,
      file_name: None,
      content_type: None,
      notes: None,
      modified_date: None,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime(
        "API request failed (400): Invalid file type"
      ))
    );
  }

  // --- Delete tests ---

  #[tokio::test]
  async fn handle_when_files_delete_succeeds_it_should_write_success_response() {
    let mock_client = MockClient {
      delete_file_result: Ok(()),
      ..Default::default()
    };

    let command = FilesCommand::Delete {
      record_id: 1,
      field_id: 2,
      file_id: 3,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();
    assert_eq!(written.trim(), r#"{"ok":true}"#);
    assert_eq!(*mock_client.delete_file_record_id.lock().unwrap(), Some(1));
    assert_eq!(*mock_client.delete_file_field_id.lock().unwrap(), Some(2));
    assert_eq!(*mock_client.delete_file_file_id.lock().unwrap(), Some(3));
  }

  #[tokio::test]
  async fn handle_when_files_delete_succeeds_pretty_it_should_write_pretty_json() {
    let mock_client = MockClient {
      delete_file_result: Ok(()),
      ..Default::default()
    };

    let command = FilesCommand::Delete {
      record_id: 1,
      field_id: 2,
      file_id: 3,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, true).await;

    assert_eq!(result, Ok(()));

    let written = String::from_utf8(buffer).unwrap();
    let expected = concat!("{\n", "  \"ok\": true\n", "}");
    assert_eq!(written.trim(), expected);
  }

  #[tokio::test]
  async fn handle_when_files_delete_with_invalid_ids_it_should_return_usage_error() {
    let mock_client = MockClient::default();

    let command = FilesCommand::Delete {
      record_id: 0,
      field_id: 2,
      file_id: 3,
    };
    let mut buffer = Vec::new();
    assert_eq!(
      handle(&command, &mock_client, &mut buffer, false).await,
      Err(CliError::usage("record_id must be greater than 0."))
    );

    let command = FilesCommand::Delete {
      record_id: 1,
      field_id: 0,
      file_id: 3,
    };
    assert_eq!(
      handle(&command, &mock_client, &mut buffer, false).await,
      Err(CliError::usage("field_id must be greater than 0."))
    );

    let command = FilesCommand::Delete {
      record_id: 1,
      field_id: 2,
      file_id: -1,
    };
    assert_eq!(
      handle(&command, &mock_client, &mut buffer, false).await,
      Err(CliError::usage("file_id must be greater than 0."))
    );
  }

  #[tokio::test]
  async fn handle_when_files_delete_fails_it_should_return_mapped_cli_error() {
    let mock_client = MockClient {
      delete_file_result: Err(OnspringError::Api {
        status_code: 404,
        message: "File not found".to_string(),
      }),
      ..Default::default()
    };

    let command = FilesCommand::Delete {
      record_id: 1,
      field_id: 2,
      file_id: 3,
    };
    let mut buffer = Vec::new();

    let result = handle(&command, &mock_client, &mut buffer, false).await;

    assert_eq!(
      result,
      Err(CliError::runtime("API request failed (404): File not found"))
    );
  }
}

