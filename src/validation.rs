use std::io::Read;

use onspring::PagingRequest;
use serde::de::DeserializeOwned;

use crate::cli::{BodySource, PagingArgs};
use crate::error::{CliError, CliResult};

pub fn paging_request(args: &PagingArgs) -> CliResult<Option<PagingRequest>> {
  if args.page_number.is_none() && args.page_size.is_none() {
    return Ok(None);
  }

  let page_number = args.page_number.unwrap_or(1);
  let page_size = args.page_size.unwrap_or(50);

  if page_number <= 0 {
    return Err(CliError::usage("page_number must be greater than 0."));
  }
  if page_size <= 0 {
    return Err(CliError::usage("page_size must be greater than 0."));
  }

  Ok(Some(PagingRequest {
    page_number,
    page_size,
  }))
}

pub fn validate_positive_i32(value: i32, field: &str) -> CliResult<()> {
  if value <= 0 {
    return Err(CliError::usage(format!("{field} must be greater than 0.")));
  }

  Ok(())
}

pub fn validate_ids(ids: &[i32], name: &str, max_len: Option<usize>) -> CliResult<()> {
  if ids.is_empty() {
    return Err(CliError::usage(format!("{name} cannot be empty.")));
  }

  if let Some(max_len) = max_len
    && ids.len() > max_len
  {
    return Err(CliError::usage(format!(
      "{name} cannot contain more than {max_len} values."
    )));
  }

  if ids.iter().any(|id| *id <= 0) {
    return Err(CliError::usage(format!(
      "{name} must only contain values > 0."
    )));
  }

  Ok(())
}

pub fn read_body<R: Read>(source: &BodySource, mut reader: R) -> CliResult<String> {
  let count = [
    source.json.is_some(),
    source.file.is_some(),
    source.stdin,
  ]
  .into_iter()
  .filter(|&b| b)
  .count();

  if count == 0 {
    return Err(CliError::usage(
      "A request body is required. Provide --json, --file, or --stdin.",
    ));
  }

  if count > 1 {
    return Err(CliError::usage(
      "Only one request body source may be specified (--json, --file, or --stdin).",
    ));
  }

  if let Some(ref json) = source.json {
    return Ok(json.clone());
  }

  if let Some(ref path) = source.file {
    return std::fs::read_to_string(path).map_err(|e| {
      CliError::usage(format!(
        "Failed to read file '{}': {}",
        path.display(),
        e
      ))
    });
  }

  if source.stdin {
    let mut buffer = String::new();
    reader
      .read_to_string(&mut buffer)
      .map_err(|e| CliError::runtime(format!("Failed to read from stdin: {e}")))?;
    return Ok(buffer);
  }

  unreachable!()
}

pub fn parse_body<T: DeserializeOwned, R: Read>(
  source: &BodySource,
  reader: R,
) -> CliResult<T> {
  let content = read_body(source, reader)?;
  serde_json::from_str(&content)
    .map_err(|e| CliError::usage(format!("Invalid JSON request body: {e}")))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn paging_request_when_no_args_it_should_return_none() {
    let args = PagingArgs {
      page_number: None,
      page_size: None,
    };

    let result = paging_request(&args);

    assert!(matches!(result, Ok(None)));
  }

  #[test]
  fn paging_request_when_page_number_provided_it_should_default_page_size() {
    let args = PagingArgs {
      page_number: Some(3),
      page_size: None,
    };

    let result = paging_request(&args).unwrap();
    let paging = result.unwrap();

    assert_eq!(paging.page_number, 3);
    assert_eq!(paging.page_size, 50);
  }

  #[test]
  fn paging_request_when_page_size_provided_it_should_default_page_number() {
    let args = PagingArgs {
      page_number: None,
      page_size: Some(25),
    };

    let result = paging_request(&args).unwrap();
    let paging = result.unwrap();

    assert_eq!(paging.page_number, 1);
    assert_eq!(paging.page_size, 25);
  }

  #[test]
  fn paging_request_when_both_args_provided_it_should_return_paging_request() {
    let args = PagingArgs {
      page_number: Some(2),
      page_size: Some(10),
    };

    let result = paging_request(&args).unwrap();
    let paging = result.unwrap();

    assert_eq!(paging.page_number, 2);
    assert_eq!(paging.page_size, 10);
  }

  #[test]
  fn paging_request_when_page_number_zero_or_negative_it_should_return_usage_error() {
    let args_zero = PagingArgs {
      page_number: Some(0),
      page_size: Some(10),
    };
    assert_eq!(
      paging_request(&args_zero).unwrap_err(),
      CliError::usage("page_number must be greater than 0.")
    );

    let args_neg = PagingArgs {
      page_number: Some(-1),
      page_size: Some(10),
    };
    assert_eq!(
      paging_request(&args_neg).unwrap_err(),
      CliError::usage("page_number must be greater than 0.")
    );
  }

  #[test]
  fn paging_request_when_page_size_zero_or_negative_it_should_return_usage_error() {
    let args_zero = PagingArgs {
      page_number: Some(1),
      page_size: Some(0),
    };
    assert_eq!(
      paging_request(&args_zero).unwrap_err(),
      CliError::usage("page_size must be greater than 0.")
    );

    let args_neg = PagingArgs {
      page_number: Some(1),
      page_size: Some(-5),
    };
    assert_eq!(
      paging_request(&args_neg).unwrap_err(),
      CliError::usage("page_size must be greater than 0.")
    );
  }

  #[test]
  fn validate_positive_i32_when_valid_it_should_return_ok() {
    assert_eq!(validate_positive_i32(1, "app_id"), Ok(()));
    assert_eq!(validate_positive_i32(100, "app_id"), Ok(()));
  }

  #[test]
  fn validate_positive_i32_when_zero_or_negative_it_should_return_usage_error() {
    assert_eq!(
      validate_positive_i32(0, "app_id"),
      Err(CliError::usage("app_id must be greater than 0."))
    );
    assert_eq!(
      validate_positive_i32(-1, "app_id"),
      Err(CliError::usage("app_id must be greater than 0."))
    );
  }

  #[test]
  fn validate_ids_when_valid_it_should_return_ok() {
    assert_eq!(validate_ids(&[1, 2, 3], "ids", Some(100)), Ok(()));
  }

  #[test]
  fn validate_ids_when_empty_it_should_return_usage_error() {
    assert_eq!(
      validate_ids(&[], "ids", Some(100)),
      Err(CliError::usage("ids cannot be empty."))
    );
  }

  #[test]
  fn validate_ids_when_contains_zero_or_negative_it_should_return_usage_error() {
    assert_eq!(
      validate_ids(&[1, 0, 3], "ids", Some(100)),
      Err(CliError::usage("ids must only contain values > 0."))
    );
    assert_eq!(
      validate_ids(&[1, -2, 3], "ids", Some(100)),
      Err(CliError::usage("ids must only contain values > 0."))
    );
  }

  #[test]
  fn validate_ids_when_exceeds_max_len_it_should_return_usage_error() {
    let ids = vec![1; 101];
    assert_eq!(
      validate_ids(&ids, "ids", Some(100)),
      Err(CliError::usage(
        "ids cannot contain more than 100 values."
      ))
    );
  }

  #[test]
  fn read_body_when_no_source_provided_it_should_return_usage_error() {
    let source = BodySource::default();
    let result = read_body(&source, std::io::empty());
    assert_eq!(
      result,
      Err(CliError::usage(
        "A request body is required. Provide --json, --file, or --stdin."
      ))
    );
  }

  #[test]
  fn read_body_when_multiple_sources_provided_it_should_return_usage_error() {
    let source = BodySource {
      json: Some("{}".to_string()),
      stdin: true,
      file: None,
    };
    let result = read_body(&source, std::io::empty());
    assert_eq!(
      result,
      Err(CliError::usage(
        "Only one request body source may be specified (--json, --file, or --stdin)."
      ))
    );
  }

  #[test]
  fn read_body_when_json_provided_it_should_return_json_string() {
    let source = BodySource {
      json: Some(r#"{"appId":1}"#.to_string()),
      file: None,
      stdin: false,
    };
    let result = read_body(&source, std::io::empty());
    assert_eq!(result, Ok(r#"{"appId":1}"#.to_string()));
  }

  #[test]
  fn read_body_when_stdin_provided_it_should_read_from_reader() {
    let source = BodySource {
      json: None,
      file: None,
      stdin: true,
    };
    let input = r#"{"appId":2}"#;
    let result = read_body(&source, input.as_bytes());
    assert_eq!(result, Ok(input.to_string()));
  }

  #[test]
  fn read_body_when_file_provided_it_should_read_from_file() {
    use std::io::Write;
    let mut temp = tempfile::NamedTempFile::new().unwrap();
    write!(temp, r#"{{"appId":3}}"#).unwrap();

    let source = BodySource {
      json: None,
      file: Some(temp.path().to_path_buf()),
      stdin: false,
    };
    let result = read_body(&source, std::io::empty());
    assert_eq!(result, Ok(r#"{"appId":3}"#.to_string()));
  }

  #[test]
  fn read_body_when_file_not_found_it_should_return_usage_error() {
    let source = BodySource {
      json: None,
      file: Some(std::path::PathBuf::from("nonexistent_file_path_12345.json")),
      stdin: false,
    };
    let result = read_body(&source, std::io::empty());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 2);
    assert!(err.message.contains("Failed to read file"));
  }

  #[test]
  fn parse_body_when_valid_it_should_deserialize() {
    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct TestData {
      app_id: i32,
    }

    let source = BodySource {
      json: Some(r#"{"app_id":42}"#.to_string()),
      file: None,
      stdin: false,
    };

    let result: Result<TestData, _> = parse_body(&source, std::io::empty());
    assert_eq!(result, Ok(TestData { app_id: 42 }));
  }

  #[test]
  fn parse_body_when_invalid_json_it_should_return_usage_error() {
    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct TestData {
      app_id: i32,
    }

    let source = BodySource {
      json: Some("invalid json".to_string()),
      file: None,
      stdin: false,
    };

    let result: Result<TestData, _> = parse_body(&source, std::io::empty());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, 2);
    assert!(err.message.contains("Invalid JSON request body"));
  }
}
