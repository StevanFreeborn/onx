use std::io::Write;

use onspring::{App, CollectionResponse, PagedResponse};
use serde::Serialize;

use crate::{CliError, CliResult};

#[derive(Debug, Serialize)]
struct ErrorBody<'a> {
  pub code: i32,
  pub message: &'a str,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope<'a> {
  pub error: ErrorBody<'a>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessResponse {
  pub ok: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedOutput<T> {
  pub page_number: Option<i32>,
  pub page_size: Option<i32>,
  pub total_pages: Option<i32>,
  pub total_records: Option<i32>,
  pub items: Option<Vec<T>>,
}

impl<T, U> From<PagedResponse<U>> for PagedOutput<T>
where
  T: From<U>,
{
  fn from(value: PagedResponse<U>) -> Self {
    Self {
      page_number: value.page_number,
      page_size: value.page_size,
      total_pages: value.total_pages,
      total_records: value.total_records,
      items: value
        .items
        .map(|items| items.into_iter().map(T::from).collect()),
    }
  }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionOutput<T> {
  pub count: Option<i32>,
  pub items: Option<Vec<T>>,
}

impl<T, U> From<CollectionResponse<U>> for CollectionOutput<T>
where
  T: From<U>,
{
  fn from(value: CollectionResponse<U>) -> Self {
    Self {
      count: value.count,
      items: value
        .items
        .map(|items| items.into_iter().map(T::from).collect()),
    }
  }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppOutput {
  pub href: Option<String>,
  pub id: i32,
  pub name: Option<String>,
}

impl From<App> for AppOutput {
  fn from(value: App) -> Self {
    Self {
      href: value.href,
      id: value.id,
      name: value.name,
    }
  }
}

pub fn render_json<T: Serialize>(value: &T, pretty: bool) -> CliResult<String> {
  if pretty {
    serde_json::to_string_pretty(value).map_err(|_| CliError::runtime("Failed to serialize output"))
  } else {
    serde_json::to_string(value).map_err(|_| CliError::runtime("Failed to serialize output"))
  }
}

pub fn render_error(err: &CliError, pretty: bool) -> CliResult<String> {
  let envelope = ErrorEnvelope {
    error: ErrorBody {
      code: err.code,
      message: &err.message,
    },
  };

  render_json(&envelope, pretty)
}

pub fn write_json<W: Write, T: Serialize>(
  writer: &mut W,
  value: &T,
  pretty: bool,
) -> CliResult<()> {
  let rendered = render_json(value, pretty)?;
  writeln!(writer, "{rendered}").map_err(|_| CliError::runtime("Failed to write output"))?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn render_json_when_called_without_pretty_it_should_print_value() {
    let data = json!({
        "name": "Stevan",
    });

    let result = render_json(&data, false);

    let expected = Ok(String::from(r#"{"name":"Stevan"}"#));
    assert_eq!(result, expected);
  }

  #[test]
  fn render_json_when_called_with_pretty_it_should_pretty_print_value() {
    let data = json!({
        "name": "Stevan",
    });

    let result = render_json(&data, true);

    let expected = Ok(String::from("{\n  \"name\": \"Stevan\"\n}"));
    assert_eq!(result, expected);
  }

  #[test]
  fn render_error_when_called_with_error_it_should_render_properly() {
    let error = CliError {
      code: 1,
      message: String::from("This is a test"),
    };

    let result = render_error(&error, false);

    let expected = Ok(String::from(
      r#"{"error":{"code":1,"message":"This is a test"}}"#,
    ));
    assert_eq!(result, expected);
  }

  #[test]
  fn write_json_when_called_with_writer_it_should_write_formatted_string() {
    let data = json!({"ok": true});
    let mut buffer = Vec::new();

    let result = write_json(&mut buffer, &data, false);
    assert_eq!(result, Ok(()));

    let output_str = String::from_utf8(buffer).unwrap();
    assert_eq!(output_str, "{\"ok\":true}\n");
  }
}
