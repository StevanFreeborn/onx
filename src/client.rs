use onspring::{
  App, CollectionResponse, OnspringClient, OnspringError, PagedResponse, PagingRequest,
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
  }
}
