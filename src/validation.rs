use onspring::PagingRequest;

use crate::cli::PagingArgs;
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
}
