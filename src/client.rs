use crate::types::*;
use reqwest::{header, Client, StatusCode};
use thiserror::Error;
use tracing::instrument;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)] // "Error" suffix is idiomatic for Rust error enums
pub enum HevyClientError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("API returned a client error ({status}): {message}")]
    ClientError { status: StatusCode, message: String },
    #[error("API returned a server error ({status}): {message}")]
    ServerError { status: StatusCode, message: String },
    #[error("API returned an unknown error state: {status}")]
    ApiError { status: StatusCode },
    #[error("Failed to parse API response: {0}")]
    ParseError(String),
}

/// A client for the Hevy API.
#[derive(Clone, Debug)]
pub struct HevyClient {
    #[allow(dead_code)] // stored for potential debug/introspection; key is set in HTTP headers
    pub api_key: String,
    pub http_client: Client,
    pub base_url: String,
}

impl HevyClient {
    pub fn new(api_key: String) -> Result<Self, anyhow::Error> {
        Self::with_base_url(api_key, "https://api.hevyapp.com".to_string())
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, anyhow::Error> {
        let mut headers = header::HeaderMap::new();
        let mut api_key_val = header::HeaderValue::from_str(&api_key)?;
        api_key_val.set_sensitive(true);
        headers.insert("api-key", api_key_val);
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        let http_client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            api_key,
            http_client,
            base_url,
        })
    }

    // Helper for handling responses uniformly
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, HevyClientError> {
        let status = response.status();
        if status.is_success() {
            response
                .json::<T>()
                .await
                .map_err(|e| HevyClientError::ParseError(e.to_string()))
        } else if status.is_client_error() {
            let message = response.text().await.unwrap_or_default();
            Err(HevyClientError::ClientError { status, message })
        } else if status.is_server_error() {
            let message = response.text().await.unwrap_or_default();
            Err(HevyClientError::ServerError { status, message })
        } else {
            Err(HevyClientError::ApiError { status })
        }
    }

    // --- WORKOUTS ---
    #[instrument(skip(self), err)]
    pub async fn get_workouts(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<WorkoutListSchema, HevyClientError> {
        let url = format!(
            "{}/v1/workouts?page={}&pageSize={}",
            self.base_url, page, page_size
        );
        let res = self.http_client.get(&url).send().await?;
        self.handle_response(res).await
    }

    #[instrument(skip(self), err)]
    pub async fn get_workout(&self, id: &str) -> Result<Workout, HevyClientError> {
        let url = format!("{}/v1/workouts/{}", self.base_url, id);
        let res = self.http_client.get(&url).send().await?;
        self.handle_response(res).await
    }

    #[instrument(skip(self), err)]
    pub async fn get_workout_count(&self) -> Result<serde_json::Value, HevyClientError> {
        let url = format!("{}/v1/workouts/count", self.base_url);
        let res = self.http_client.get(&url).send().await?;
        self.handle_response(res).await
    }

    #[instrument(skip(self), err)]
    pub async fn get_workout_events(
        &self,
        page: u32,
        page_size: u32,
        since: &str,
    ) -> Result<serde_json::Value, HevyClientError> {
        let url = format!(
            "{}/v1/workouts/events?page={}&pageSize={}&since={}",
            self.base_url, page, page_size, since
        );
        let res = self.http_client.get(&url).send().await?;
        self.handle_response(res).await
    }

    #[instrument(skip(self, payload), err)]
    pub async fn create_workout(
        &self,
        payload: serde_json::Value,
    ) -> Result<Workout, HevyClientError> {
        let url = format!("{}/v1/workouts", self.base_url);
        let res = self.http_client.post(&url).json(&payload).send().await?;
        self.handle_response(res).await
    }

    #[instrument(skip(self, payload), err)]
    pub async fn update_workout(
        &self,
        id: &str,
        payload: serde_json::Value,
    ) -> Result<Workout, HevyClientError> {
        let url = format!("{}/v1/workouts/{}", self.base_url, id);
        let res = self.http_client.put(&url).json(&payload).send().await?;
        self.handle_response(res).await
    }

    // --- ROUTINES ---
    #[instrument(skip(self), err)]
    pub async fn get_routines(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<RoutineListSchema, HevyClientError> {
        let url = format!(
            "{}/v1/routines?page={}&pageSize={}",
            self.base_url, page, page_size
        );
        let res = self.http_client.get(&url).send().await?;
        self.handle_response(res).await
    }

    #[instrument(skip(self), err)]
    pub async fn get_routine(&self, id: &str) -> Result<Routine, HevyClientError> {
        let url = format!("{}/v1/routines/{}", self.base_url, id);
        let res = self.http_client.get(&url).send().await?;
        self.handle_response(res).await
    }

    #[instrument(skip(self, payload), err)]
    pub async fn create_routine(
        &self,
        payload: serde_json::Value,
    ) -> Result<Routine, HevyClientError> {
        let url = format!("{}/v1/routines", self.base_url);
        let res = self.http_client.post(&url).json(&payload).send().await?;
        self.handle_response(res).await
    }

    #[instrument(skip(self, payload), err)]
    pub async fn update_routine(
        &self,
        id: &str,
        payload: serde_json::Value,
    ) -> Result<Routine, HevyClientError> {
        let url = format!("{}/v1/routines/{}", self.base_url, id);
        let res = self.http_client.put(&url).json(&payload).send().await?;
        self.handle_response(res).await
    }

    // --- FOLDERS ---
    #[instrument(skip(self), err)]
    pub async fn get_folders(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<FolderListSchema, HevyClientError> {
        let url = format!(
            "{}/v1/routine_folders?page={}&pageSize={}",
            self.base_url, page, page_size
        );
        let res = self.http_client.get(&url).send().await?;
        self.handle_response(res).await
    }

    /// Get a single routine folder by ID
    #[instrument(skip(self), err)]
    pub async fn get_folder(&self, id: &str) -> Result<RoutineFolder, HevyClientError> {
        let url = format!("{}/v1/routine_folders/{}", self.base_url, id);
        let res = self.http_client.get(&url).send().await?;
        self.handle_response(res).await
    }

    #[instrument(skip(self, payload), err)]
    pub async fn create_folder(
        &self,
        payload: serde_json::Value,
    ) -> Result<RoutineFolder, HevyClientError> {
        let url = format!("{}/v1/routine_folders", self.base_url);
        let res = self.http_client.post(&url).json(&payload).send().await?;
        self.handle_response(res).await
    }

    // --- TEMPLATES ---
    #[instrument(skip(self), err)]
    pub async fn get_templates(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<TemplateListSchema, HevyClientError> {
        let url = format!(
            "{}/v1/exercise_templates?page={}&pageSize={}",
            self.base_url, page, page_size
        );
        let res = self.http_client.get(&url).send().await?;
        self.handle_response(res).await
    }

    #[instrument(skip(self), err)]
    pub async fn get_template(&self, id: &str) -> Result<ExerciseTemplate, HevyClientError> {
        let url = format!("{}/v1/exercise_templates/{}", self.base_url, id);
        let res = self.http_client.get(&url).send().await?;
        self.handle_response(res).await
    }

    /// Get exercise history for a specific exercise template
    #[instrument(skip(self), err)]
    pub async fn get_exercise_history(
        &self,
        exercise_template_id: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<serde_json::Value, HevyClientError> {
        let mut url = format!(
            "{}/v1/exercise_history/{}",
            self.base_url, exercise_template_id
        );
        let mut params = vec![];
        if let Some(sd) = start_date {
            params.push(format!("start_date={}", sd));
        }
        if let Some(ed) = end_date {
            params.push(format!("end_date={}", ed));
        }
        if !params.is_empty() {
            url.push_str(&format!("?{}", params.join("&")));
        }
        let res = self.http_client.get(&url).send().await?;
        self.handle_response(res).await
    }

    /// Create a custom exercise template
    #[instrument(skip(self, payload), err)]
    pub async fn create_exercise_template(
        &self,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, HevyClientError> {
        let url = format!("{}/v1/exercise_templates", self.base_url);
        let res = self.http_client.post(&url).json(&payload).send().await?;
        self.handle_response(res).await
    }

    // --- WEBHOOKS (singleton, no ID) ---

    /// Get the account's webhook subscription (GET /v1/webhooks)
    #[instrument(skip(self), err)]
    pub async fn get_webhook_subscription(&self) -> Result<serde_json::Value, HevyClientError> {
        let url = format!("{}/v1/webhooks", self.base_url);
        let res = self.http_client.get(&url).send().await?;
        self.handle_response(res).await
    }

    /// Create a webhook subscription (POST /v1/webhooks)
    #[instrument(skip(self, payload), err)]
    pub async fn create_webhook_subscription(
        &self,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, HevyClientError> {
        let url = format!("{}/v1/webhooks", self.base_url);
        let res = self.http_client.post(&url).json(&payload).send().await?;
        self.handle_response(res).await
    }

    /// Delete the account's webhook subscription (DELETE /v1/webhooks)
    #[instrument(skip(self), err)]
    pub async fn delete_webhook_subscription(&self) -> Result<(), HevyClientError> {
        let url = format!("{}/v1/webhooks", self.base_url);
        let res = self.http_client.delete(&url).send().await?;
        let status = res.status();
        if status.is_success() {
            Ok(())
        } else {
            let message = res.text().await.unwrap_or_default();
            Err(HevyClientError::ServerError { status, message })
        }
    }
}
