use onspring::{
  App, BatchDeleteRecordsRequest, BatchGetRecordsRequest, CollectionResponse,
  CreatedWithIdResponse, DataFormat, Field, FileInfo, FileResponse, OnspringClient, OnspringError,
  PagedResponse, PagingRequest, QueryRecordsRequest, Record, SaveFileRequest, SaveRecordRequest,
  SaveRecordResponse,
};

pub trait OnspringRunner {
  fn ping(&self) -> impl std::future::Future<Output = Result<(), OnspringError>> + Send;

  fn list_apps(
    &self,
    paging: Option<PagingRequest>,
  ) -> impl std::future::Future<Output = Result<PagedResponse<App>, OnspringError>> + Send;

  fn get_app(
    &self,
    app_id: i32,
  ) -> impl std::future::Future<Output = Result<App, OnspringError>> + Send;

  fn batch_get_apps(
    &self,
    ids: &[i32],
  ) -> impl std::future::Future<Output = Result<CollectionResponse<App>, OnspringError>> + Send;

  fn list_fields(
    &self,
    app_id: i32,
    paging: Option<PagingRequest>,
  ) -> impl std::future::Future<Output = Result<PagedResponse<Field>, OnspringError>> + Send;

  fn get_field(
    &self,
    field_id: i32,
  ) -> impl std::future::Future<Output = Result<Field, OnspringError>> + Send;

  fn batch_get_fields(
    &self,
    ids: &[i32],
  ) -> impl std::future::Future<Output = Result<CollectionResponse<Field>, OnspringError>> + Send;

  fn list_records(
    &self,
    app_id: i32,
    paging: Option<PagingRequest>,
    field_ids: Option<&[i32]>,
    data_format: Option<DataFormat>,
  ) -> impl std::future::Future<Output = Result<PagedResponse<Record>, OnspringError>> + Send;

  fn get_record(
    &self,
    app_id: i32,
    record_id: i32,
    field_ids: Option<&[i32]>,
    data_format: Option<DataFormat>,
  ) -> impl std::future::Future<Output = Result<Record, OnspringError>> + Send;

  fn save_record(
    &self,
    request: SaveRecordRequest,
  ) -> impl std::future::Future<Output = Result<SaveRecordResponse, OnspringError>> + Send;

  fn delete_record(
    &self,
    app_id: i32,
    record_id: i32,
  ) -> impl std::future::Future<Output = Result<(), OnspringError>> + Send;

  fn batch_get_records(
    &self,
    request: BatchGetRecordsRequest,
  ) -> impl std::future::Future<Output = Result<CollectionResponse<Record>, OnspringError>> + Send;

  fn query_records(
    &self,
    request: QueryRecordsRequest,
    paging: Option<PagingRequest>,
  ) -> impl std::future::Future<Output = Result<PagedResponse<Record>, OnspringError>> + Send;

  fn batch_delete_records(
    &self,
    request: BatchDeleteRecordsRequest,
  ) -> impl std::future::Future<Output = Result<(), OnspringError>> + Send;

  fn get_file_info(
    &self,
    record_id: i32,
    field_id: i32,
    file_id: i32,
  ) -> impl std::future::Future<Output = Result<FileInfo, OnspringError>> + Send;

  fn get_file(
    &self,
    record_id: i32,
    field_id: i32,
    file_id: i32,
  ) -> impl std::future::Future<Output = Result<FileResponse, OnspringError>> + Send;

  fn upload_file(
    &self,
    request: SaveFileRequest,
  ) -> impl std::future::Future<Output = Result<CreatedWithIdResponse, OnspringError>> + Send;

  fn delete_file(
    &self,
    record_id: i32,
    field_id: i32,
    file_id: i32,
  ) -> impl std::future::Future<Output = Result<(), OnspringError>> + Send;
}

impl OnspringRunner for OnspringClient {
  async fn ping(&self) -> Result<(), OnspringError> {
    self.ping().await
  }

  async fn list_apps(
    &self,
    paging: Option<PagingRequest>,
  ) -> Result<PagedResponse<App>, OnspringError> {
    self.list_apps(paging).await
  }

  async fn get_app(&self, app_id: i32) -> Result<App, OnspringError> {
    self.get_app(app_id).await
  }

  async fn batch_get_apps(&self, ids: &[i32]) -> Result<CollectionResponse<App>, OnspringError> {
    self.batch_get_apps(ids).await
  }

  async fn list_fields(
    &self,
    app_id: i32,
    paging: Option<PagingRequest>,
  ) -> Result<PagedResponse<Field>, OnspringError> {
    self.list_fields(app_id, paging).await
  }

  async fn get_field(&self, field_id: i32) -> Result<Field, OnspringError> {
    self.get_field(field_id).await
  }

  async fn batch_get_fields(&self, ids: &[i32]) -> Result<CollectionResponse<Field>, OnspringError> {
    self.batch_get_fields(ids).await
  }

  async fn list_records(
    &self,
    app_id: i32,
    paging: Option<PagingRequest>,
    field_ids: Option<&[i32]>,
    data_format: Option<DataFormat>,
  ) -> Result<PagedResponse<Record>, OnspringError> {
    self.list_records(app_id, paging, field_ids, data_format).await
  }

  async fn get_record(
    &self,
    app_id: i32,
    record_id: i32,
    field_ids: Option<&[i32]>,
    data_format: Option<DataFormat>,
  ) -> Result<Record, OnspringError> {
    self.get_record(app_id, record_id, field_ids, data_format).await
  }

  async fn save_record(
    &self,
    request: SaveRecordRequest,
  ) -> Result<SaveRecordResponse, OnspringError> {
    self.save_record(request).await
  }

  async fn delete_record(&self, app_id: i32, record_id: i32) -> Result<(), OnspringError> {
    self.delete_record(app_id, record_id).await
  }

  async fn batch_get_records(
    &self,
    request: BatchGetRecordsRequest,
  ) -> Result<CollectionResponse<Record>, OnspringError> {
    self.batch_get_records(request).await
  }

  async fn query_records(
    &self,
    request: QueryRecordsRequest,
    paging: Option<PagingRequest>,
  ) -> Result<PagedResponse<Record>, OnspringError> {
    self.query_records(request, paging).await
  }

  async fn batch_delete_records(
    &self,
    request: BatchDeleteRecordsRequest,
  ) -> Result<(), OnspringError> {
    self.batch_delete_records(request).await
  }

  async fn get_file_info(
    &self,
    record_id: i32,
    field_id: i32,
    file_id: i32,
  ) -> Result<FileInfo, OnspringError> {
    self.get_file_info(record_id, field_id, file_id).await
  }

  async fn get_file(
    &self,
    record_id: i32,
    field_id: i32,
    file_id: i32,
  ) -> Result<FileResponse, OnspringError> {
    self.get_file(record_id, field_id, file_id).await
  }

  async fn upload_file(
    &self,
    request: SaveFileRequest,
  ) -> Result<CreatedWithIdResponse, OnspringError> {
    self.upload_file(request).await
  }

  async fn delete_file(
    &self,
    record_id: i32,
    field_id: i32,
    file_id: i32,
  ) -> Result<(), OnspringError> {
    self.delete_file(record_id, field_id, file_id).await
  }
}

#[cfg(test)]
pub mod testing {
  use super::*;
  use std::sync::Mutex;

  pub struct MockClient {
    pub ping_result: Result<(), OnspringError>,
    pub list_apps_result: Result<PagedResponse<App>, OnspringError>,
    pub get_app_result: Result<App, OnspringError>,
    pub batch_get_apps_result: Result<CollectionResponse<App>, OnspringError>,
    pub list_apps_paging: Mutex<Option<Option<PagingRequest>>>,
    pub get_app_id: Mutex<Option<i32>>,
    pub batch_get_apps_ids: Mutex<Option<Vec<i32>>>,

    pub list_fields_result: Result<PagedResponse<Field>, OnspringError>,
    pub get_field_result: Result<Field, OnspringError>,
    pub batch_get_fields_result: Result<CollectionResponse<Field>, OnspringError>,
    pub list_fields_app_id: Mutex<Option<i32>>,
    pub list_fields_paging: Mutex<Option<Option<PagingRequest>>>,
    pub get_field_id: Mutex<Option<i32>>,
    pub batch_get_fields_ids: Mutex<Option<Vec<i32>>>,

    pub list_records_result: Result<PagedResponse<Record>, OnspringError>,
    pub get_record_result: Result<Record, OnspringError>,
    pub save_record_result: Result<SaveRecordResponse, OnspringError>,
    pub delete_record_result: Result<(), OnspringError>,
    pub batch_get_records_result: Result<CollectionResponse<Record>, OnspringError>,
    pub query_records_result: Result<PagedResponse<Record>, OnspringError>,
    pub batch_delete_records_result: Result<(), OnspringError>,

    pub list_records_app_id: Mutex<Option<i32>>,
    pub list_records_paging: Mutex<Option<Option<PagingRequest>>>,
    pub list_records_field_ids: Mutex<Option<Option<Vec<i32>>>>,
    pub list_records_data_format: Mutex<Option<Option<DataFormat>>>,

    pub get_record_app_id: Mutex<Option<i32>>,
    pub get_record_record_id: Mutex<Option<i32>>,
    pub get_record_field_ids: Mutex<Option<Option<Vec<i32>>>>,
    pub get_record_data_format: Mutex<Option<Option<DataFormat>>>,

    pub save_record_request: Mutex<Option<SaveRecordRequest>>,
    pub delete_record_app_id: Mutex<Option<i32>>,
    pub delete_record_record_id: Mutex<Option<i32>>,
    pub batch_get_records_request: Mutex<Option<BatchGetRecordsRequest>>,
    pub query_records_request: Mutex<Option<QueryRecordsRequest>>,
    pub query_records_paging: Mutex<Option<Option<PagingRequest>>>,
    pub batch_delete_records_request: Mutex<Option<BatchDeleteRecordsRequest>>,

    pub get_file_info_result: Result<FileInfo, OnspringError>,
    pub get_file_result: Result<FileResponse, OnspringError>,
    pub upload_file_result: Result<CreatedWithIdResponse, OnspringError>,
    pub delete_file_result: Result<(), OnspringError>,

    pub get_file_info_record_id: Mutex<Option<i32>>,
    pub get_file_info_field_id: Mutex<Option<i32>>,
    pub get_file_info_file_id: Mutex<Option<i32>>,

    pub get_file_record_id: Mutex<Option<i32>>,
    pub get_file_field_id: Mutex<Option<i32>>,
    pub get_file_file_id: Mutex<Option<i32>>,

    pub upload_file_request: Mutex<Option<SaveFileRequest>>,

    pub delete_file_record_id: Mutex<Option<i32>>,
    pub delete_file_field_id: Mutex<Option<i32>>,
    pub delete_file_file_id: Mutex<Option<i32>>,
  }

  impl Default for MockClient {
    fn default() -> Self {
      Self {
        ping_result: Ok(()),
        list_apps_result: Ok(PagedResponse {
          page_number: None,
          page_size: None,
          total_pages: None,
          total_records: None,
          items: None,
        }),
        get_app_result: Ok(App {
          href: None,
          id: 1,
          name: None,
        }),
        batch_get_apps_result: Ok(CollectionResponse {
          count: None,
          items: None,
        }),
        list_apps_paging: Mutex::new(None),
        get_app_id: Mutex::new(None),
        batch_get_apps_ids: Mutex::new(None),

        list_fields_result: Ok(PagedResponse {
          page_number: None,
          page_size: None,
          total_pages: None,
          total_records: None,
          items: None,
        }),
        get_field_result: Ok(Field {
          id: 1,
          app_id: 1,
          name: None,
          field_type: None,
          status: None,
          is_required: false,
          is_unique: false,
          multiplicity: None,
          list_id: None,
          values: None,
          output_type: None,
          referenced_app_id: None,
        }),
        batch_get_fields_result: Ok(CollectionResponse {
          count: None,
          items: None,
        }),
        list_fields_app_id: Mutex::new(None),
        list_fields_paging: Mutex::new(None),
        get_field_id: Mutex::new(None),
        batch_get_fields_ids: Mutex::new(None),

        list_records_result: Ok(PagedResponse {
          page_number: None,
          page_size: None,
          total_pages: None,
          total_records: None,
          items: None,
        }),
        get_record_result: Ok(Record {
          app_id: 1,
          record_id: 1,
          field_data: None,
        }),
        save_record_result: Ok(SaveRecordResponse {
          id: 1,
          warnings: None,
        }),
        delete_record_result: Ok(()),
        batch_get_records_result: Ok(CollectionResponse {
          count: None,
          items: None,
        }),
        query_records_result: Ok(PagedResponse {
          page_number: None,
          page_size: None,
          total_pages: None,
          total_records: None,
          items: None,
        }),
        batch_delete_records_result: Ok(()),

        list_records_app_id: Mutex::new(None),
        list_records_paging: Mutex::new(None),
        list_records_field_ids: Mutex::new(None),
        list_records_data_format: Mutex::new(None),

        get_record_app_id: Mutex::new(None),
        get_record_record_id: Mutex::new(None),
        get_record_field_ids: Mutex::new(None),
        get_record_data_format: Mutex::new(None),

        save_record_request: Mutex::new(None),
        delete_record_app_id: Mutex::new(None),
        delete_record_record_id: Mutex::new(None),
        batch_get_records_request: Mutex::new(None),
        query_records_request: Mutex::new(None),
        query_records_paging: Mutex::new(None),
        batch_delete_records_request: Mutex::new(None),

        get_file_info_result: Ok(FileInfo {
          file_type: None,
          content_type: None,
          name: None,
          created_date: None,
          modified_date: None,
          owner: None,
          notes: None,
          file_href: None,
        }),
        get_file_result: Ok(FileResponse {
          content_type: None,
          file_name: None,
          data: bytes::Bytes::new(),
        }),
        upload_file_result: Ok(CreatedWithIdResponse { id: 1 }),
        delete_file_result: Ok(()),

        get_file_info_record_id: Mutex::new(None),
        get_file_info_field_id: Mutex::new(None),
        get_file_info_file_id: Mutex::new(None),

        get_file_record_id: Mutex::new(None),
        get_file_field_id: Mutex::new(None),
        get_file_file_id: Mutex::new(None),

        upload_file_request: Mutex::new(None),

        delete_file_record_id: Mutex::new(None),
        delete_file_field_id: Mutex::new(None),
        delete_file_file_id: Mutex::new(None),
      }
    }
  }

  fn clone_onspring_error(err: &OnspringError) -> OnspringError {
    match err {
      OnspringError::InvalidArgument(msg) => OnspringError::InvalidArgument(msg.clone()),
      OnspringError::Api {
        status_code,
        message,
      } => OnspringError::Api {
        status_code: *status_code,
        message: message.clone(),
      },
      OnspringError::Serialization(_) => {
        let serde_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        OnspringError::Serialization(serde_err)
      }
      OnspringError::Http(_) => OnspringError::InvalidArgument("http error".to_string()),
    }
  }

  impl OnspringRunner for MockClient {
    async fn ping(&self) -> Result<(), OnspringError> {
      match &self.ping_result {
        Ok(()) => Ok(()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn list_apps(
      &self,
      paging: Option<PagingRequest>,
    ) -> Result<PagedResponse<App>, OnspringError> {
      *self.list_apps_paging.lock().unwrap() = Some(paging);
      match &self.list_apps_result {
        Ok(res) => Ok(res.clone()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn get_app(&self, app_id: i32) -> Result<App, OnspringError> {
      *self.get_app_id.lock().unwrap() = Some(app_id);
      match &self.get_app_result {
        Ok(res) => Ok(res.clone()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn batch_get_apps(&self, ids: &[i32]) -> Result<CollectionResponse<App>, OnspringError> {
      *self.batch_get_apps_ids.lock().unwrap() = Some(ids.to_vec());
      match &self.batch_get_apps_result {
        Ok(res) => Ok(res.clone()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn list_fields(
      &self,
      app_id: i32,
      paging: Option<PagingRequest>,
    ) -> Result<PagedResponse<Field>, OnspringError> {
      *self.list_fields_app_id.lock().unwrap() = Some(app_id);
      *self.list_fields_paging.lock().unwrap() = Some(paging);
      match &self.list_fields_result {
        Ok(res) => Ok(res.clone()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn get_field(&self, field_id: i32) -> Result<Field, OnspringError> {
      *self.get_field_id.lock().unwrap() = Some(field_id);
      match &self.get_field_result {
        Ok(res) => Ok(res.clone()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn batch_get_fields(&self, ids: &[i32]) -> Result<CollectionResponse<Field>, OnspringError> {
      *self.batch_get_fields_ids.lock().unwrap() = Some(ids.to_vec());
      match &self.batch_get_fields_result {
        Ok(res) => Ok(res.clone()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn list_records(
      &self,
      app_id: i32,
      paging: Option<PagingRequest>,
      field_ids: Option<&[i32]>,
      data_format: Option<DataFormat>,
    ) -> Result<PagedResponse<Record>, OnspringError> {
      *self.list_records_app_id.lock().unwrap() = Some(app_id);
      *self.list_records_paging.lock().unwrap() = Some(paging);
      *self.list_records_field_ids.lock().unwrap() = Some(field_ids.map(|ids| ids.to_vec()));
      *self.list_records_data_format.lock().unwrap() = Some(data_format);
      match &self.list_records_result {
        Ok(res) => Ok(res.clone()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn get_record(
      &self,
      app_id: i32,
      record_id: i32,
      field_ids: Option<&[i32]>,
      data_format: Option<DataFormat>,
    ) -> Result<Record, OnspringError> {
      *self.get_record_app_id.lock().unwrap() = Some(app_id);
      *self.get_record_record_id.lock().unwrap() = Some(record_id);
      *self.get_record_field_ids.lock().unwrap() = Some(field_ids.map(|ids| ids.to_vec()));
      *self.get_record_data_format.lock().unwrap() = Some(data_format);
      match &self.get_record_result {
        Ok(res) => Ok(res.clone()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn save_record(
      &self,
      request: SaveRecordRequest,
    ) -> Result<SaveRecordResponse, OnspringError> {
      *self.save_record_request.lock().unwrap() = Some(request);
      match &self.save_record_result {
        Ok(res) => Ok(res.clone()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn delete_record(&self, app_id: i32, record_id: i32) -> Result<(), OnspringError> {
      *self.delete_record_app_id.lock().unwrap() = Some(app_id);
      *self.delete_record_record_id.lock().unwrap() = Some(record_id);
      match &self.delete_record_result {
        Ok(()) => Ok(()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn batch_get_records(
      &self,
      request: BatchGetRecordsRequest,
    ) -> Result<CollectionResponse<Record>, OnspringError> {
      *self.batch_get_records_request.lock().unwrap() = Some(request);
      match &self.batch_get_records_result {
        Ok(res) => Ok(res.clone()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn query_records(
      &self,
      request: QueryRecordsRequest,
      paging: Option<PagingRequest>,
    ) -> Result<PagedResponse<Record>, OnspringError> {
      *self.query_records_request.lock().unwrap() = Some(request);
      *self.query_records_paging.lock().unwrap() = Some(paging);
      match &self.query_records_result {
        Ok(res) => Ok(res.clone()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn batch_delete_records(
      &self,
      request: BatchDeleteRecordsRequest,
    ) -> Result<(), OnspringError> {
      *self.batch_delete_records_request.lock().unwrap() = Some(request);
      match &self.batch_delete_records_result {
        Ok(()) => Ok(()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn get_file_info(
      &self,
      record_id: i32,
      field_id: i32,
      file_id: i32,
    ) -> Result<FileInfo, OnspringError> {
      *self.get_file_info_record_id.lock().unwrap() = Some(record_id);
      *self.get_file_info_field_id.lock().unwrap() = Some(field_id);
      *self.get_file_info_file_id.lock().unwrap() = Some(file_id);
      match &self.get_file_info_result {
        Ok(res) => Ok(res.clone()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn get_file(
      &self,
      record_id: i32,
      field_id: i32,
      file_id: i32,
    ) -> Result<FileResponse, OnspringError> {
      *self.get_file_record_id.lock().unwrap() = Some(record_id);
      *self.get_file_field_id.lock().unwrap() = Some(field_id);
      *self.get_file_file_id.lock().unwrap() = Some(file_id);
      match &self.get_file_result {
        Ok(res) => Ok(res.clone()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn upload_file(
      &self,
      request: SaveFileRequest,
    ) -> Result<CreatedWithIdResponse, OnspringError> {
      *self.upload_file_request.lock().unwrap() = Some(request);
      match &self.upload_file_result {
        Ok(res) => Ok(res.clone()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }

    async fn delete_file(
      &self,
      record_id: i32,
      field_id: i32,
      file_id: i32,
    ) -> Result<(), OnspringError> {
      *self.delete_file_record_id.lock().unwrap() = Some(record_id);
      *self.delete_file_field_id.lock().unwrap() = Some(field_id);
      *self.delete_file_file_id.lock().unwrap() = Some(file_id);
      match &self.delete_file_result {
        Ok(()) => Ok(()),
        Err(err) => Err(clone_onspring_error(err)),
      }
    }
  }
}
