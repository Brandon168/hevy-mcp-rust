use crate::types::*;
use reqwest::{header, Client, StatusCode};
use thiserror::Error;
use tracing::instrument;

fn parse_time(value: &str) -> Result<chrono::DateTime<chrono::Utc>, HevyClientError> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|t| t.with_timezone(&chrono::Utc))
        .map_err(|e| HevyClientError::ParseError(format!("invalid timestamp {value:?}: {e}")))
}

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

    /// Deserialize a resource returned either directly or under its API wrapper key.
    async fn handle_resource_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
        wrapper_key: &str,
    ) -> Result<T, HevyClientError> {
        let body: serde_json::Value = self.handle_response(response).await?;
        let resource = body.get(wrapper_key).cloned().unwrap_or(body);
        serde_json::from_value(resource).map_err(|e| HevyClientError::ParseError(e.to_string()))
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
        self.handle_resource_response(res, "workout").await
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
        self.handle_resource_response(res, "workout").await
    }

    #[instrument(skip(self, payload), err)]
    pub async fn update_workout(
        &self,
        id: &str,
        payload: serde_json::Value,
    ) -> Result<Workout, HevyClientError> {
        let url = format!("{}/v1/workouts/{}", self.base_url, id);
        let res = self.http_client.put(&url).json(&payload).send().await?;
        self.handle_resource_response(res, "workout").await
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
        self.handle_resource_response(res, "routine").await
    }

    #[instrument(skip(self, payload), err)]
    pub async fn create_routine(
        &self,
        payload: serde_json::Value,
    ) -> Result<Routine, HevyClientError> {
        let url = format!("{}/v1/routines", self.base_url);
        let res = self.http_client.post(&url).json(&payload).send().await?;
        self.handle_resource_response(res, "routine").await
    }

    #[instrument(skip(self, payload), err)]
    pub async fn update_routine(
        &self,
        id: &str,
        payload: serde_json::Value,
    ) -> Result<Routine, HevyClientError> {
        let url = format!("{}/v1/routines/{}", self.base_url, id);
        let res = self.http_client.put(&url).json(&payload).send().await?;
        self.handle_resource_response(res, "routine").await
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
        self.handle_resource_response(res, "routine_folder").await
    }

    #[instrument(skip(self, payload), err)]
    pub async fn create_folder(
        &self,
        payload: serde_json::Value,
    ) -> Result<RoutineFolder, HevyClientError> {
        let url = format!("{}/v1/routine_folders", self.base_url);
        let res = self.http_client.post(&url).json(&payload).send().await?;
        self.handle_resource_response(res, "routine_folder").await
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
        self.handle_resource_response(res, "exercise_template")
            .await
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

    // --- USER ---

    /// Get the authenticated user's info (GET /v1/user/info).
    /// Returned under the `data` wrapper key.
    #[instrument(skip(self), err)]
    pub async fn get_user_info(&self) -> Result<UserInfo, HevyClientError> {
        let url = format!("{}/v1/user/info", self.base_url);
        let res = self.http_client.get(&url).send().await?;
        self.handle_resource_response(res, "data").await
    }

    // --- MEASUREMENTS ---

    #[instrument(skip(self), err)]
    pub async fn get_body_measurements(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<BodyMeasurementListSchema, HevyClientError> {
        let url = format!(
            "{}/v1/body_measurements?page={}&pageSize={}",
            self.base_url, page, page_size
        );
        let res = self.http_client.get(&url).send().await?;
        self.handle_response(res).await
    }

    /// Get the body measurement for one YYYY-MM-DD date.
    /// Returns None on 404 (no measurement for that date).
    #[instrument(skip(self), err)]
    pub async fn get_body_measurement(
        &self,
        date: &str,
    ) -> Result<Option<BodyMeasurement>, HevyClientError> {
        let url = format!("{}/v1/body_measurements/{}", self.base_url, date);
        let res = self.http_client.get(&url).send().await?;
        if res.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        self.handle_response(res).await.map(Some)
    }

    /// Create a body measurement entry. The `date` is part of the body;
    /// explicit nulls are omitted because the API rejects them.
    /// POST returns 200 with an empty body, so no response is parsed.
    /// A duplicate date surfaces as a 409 ClientError.
    #[instrument(skip(self, payload), err)]
    pub async fn create_body_measurement(
        &self,
        payload: serde_json::Value,
    ) -> Result<(), HevyClientError> {
        let url = format!("{}/v1/body_measurements", self.base_url);
        let res = self.http_client.post(&url).json(&payload).send().await?;
        let status = res.status();
        if status.is_success() {
            Ok(())
        } else if status.is_client_error() {
            let message = res.text().await.unwrap_or_default();
            Err(HevyClientError::ClientError { status, message })
        } else if status.is_server_error() {
            let message = res.text().await.unwrap_or_default();
            Err(HevyClientError::ServerError { status, message })
        } else {
            Err(HevyClientError::ApiError { status })
        }
    }

    /// Update the measurement for a date. The API overwrites every field and
    /// rejects explicit nulls, so callers must pass a payload where omitted
    /// fields were filled from the existing record and nulls were dropped.
    /// PUT returns 200 with an empty body, so no response is parsed.
    #[instrument(skip(self, payload), err)]
    pub async fn update_body_measurement(
        &self,
        date: &str,
        payload: serde_json::Value,
    ) -> Result<(), HevyClientError> {
        let url = format!("{}/v1/body_measurements/{}", self.base_url, date);
        let res = self.http_client.put(&url).json(&payload).send().await?;
        let status = res.status();
        if status.is_success() {
            Ok(())
        } else if status.is_client_error() {
            let message = res.text().await.unwrap_or_default();
            Err(HevyClientError::ClientError { status, message })
        } else if status.is_server_error() {
            let message = res.text().await.unwrap_or_default();
            Err(HevyClientError::ServerError { status, message })
        } else {
            Err(HevyClientError::ApiError { status })
        }
    }

    // --- SUMMARY ---

    /// Aggregate a training window client-side: scan workouts and body
    /// measurements (10 per page), keep items on/after `cutoff`, and total
    /// duration, exercises, sets, volume (weight_kg × reps when both are
    /// finite), plus the earliest/latest measurement and weight change.
    /// `weeks` bounds (1-12 tool, 1-520 CLI) are enforced by callers.
    #[instrument(skip(self), err)]
    pub async fn training_summary(
        &self,
        weeks: u32,
        cutoff: chrono::DateTime<chrono::Utc>,
    ) -> Result<TrainingSummary, HevyClientError> {
        let now = chrono::Utc::now();
        let mut summary = TrainingSummary {
            start_date: cutoff.format("%Y-%m-%d").to_string(),
            end_date: now.format("%Y-%m-%d").to_string(),
            weeks,
            workout_count: 0,
            total_duration_seconds: 0,
            exercise_count: 0,
            set_count: 0,
            total_volume_kg: 0.0,
            unique_exercise_template_ids: Vec::new(),
            sessions: Vec::new(),
            measurement_count: 0,
            earliest_measurement: None,
            latest_measurement: None,
            weight_change_kg: None,
            pages_scanned: 0,
        };
        let mut seen_templates = std::collections::HashSet::new();

        let mut page = 1u32;
        loop {
            let list = self.get_workouts(page, 10).await?;
            summary.pages_scanned += 1;
            let mut saw_recent = false;
            for w in list.workouts {
                let start = parse_time(&w.start_time)?;
                if start < cutoff {
                    continue;
                }
                saw_recent = true;
                summary.workout_count += 1;
                summary.exercise_count += w.exercises.len();
                let mut session_sets = 0usize;
                for e in &w.exercises {
                    session_sets += e.sets.len();
                    if let Some(id) = e.exercise_template_id.as_deref() {
                        if seen_templates.insert(id.to_string()) {
                            summary.unique_exercise_template_ids.push(id.to_string());
                        }
                    }
                    for s in &e.sets {
                        if let (Some(kg), Some(reps)) = (s.weight_kg, s.reps) {
                            let reps = reps as f64;
                            if kg.is_finite() && reps.is_finite() {
                                summary.total_volume_kg += kg * reps;
                            }
                        }
                    }
                }
                summary.set_count += session_sets;
                let duration = parse_time(&w.end_time)? - start;
                let duration_seconds = duration.num_seconds().max(0);
                summary.total_duration_seconds += duration_seconds;
                summary.sessions.push(SummarySession {
                    id: w.id,
                    title: w.title,
                    start_time: w.start_time,
                    end_time: w.end_time,
                    duration_seconds,
                    exercise_count: w.exercises.len(),
                    set_count: session_sets,
                });
            }
            if page >= list.page_count as u32 || !saw_recent {
                break;
            }
            page += 1;
        }

        // Measurements: API returns newest-first; collect in-window, then the
        // earliest/latest are the min/max by date.
        let mut in_window: Vec<BodyMeasurement> = Vec::new();
        let mut mpage = 1u32;
        loop {
            let list = self.get_body_measurements(mpage, 10).await?;
            summary.pages_scanned += 1;
            if list.body_measurements.is_empty() {
                break;
            }
            let mut saw_any = false;
            for m in list.body_measurements {
                if m.date.as_str() >= summary.start_date.as_str() {
                    saw_any = true;
                    in_window.push(m);
                }
            }
            if mpage >= list.page_count as u32 || !saw_any {
                break;
            }
            mpage += 1;
        }
        in_window.sort_by(|a, b| a.date.cmp(&b.date));
        summary.measurement_count = in_window.len();
        let compact = |m: &BodyMeasurement| SummaryMeasurement {
            date: m.date.clone(),
            weight_kg: m.weight_kg,
            lean_mass_kg: m.lean_mass_kg,
            fat_percent: m.fat_percent,
        };
        summary.earliest_measurement = in_window.first().map(compact);
        summary.latest_measurement = in_window.last().map(compact);
        summary.weight_change_kg = match (
            summary.earliest_measurement.as_ref(),
            summary.latest_measurement.as_ref(),
        ) {
            (Some(a), Some(b)) => match (a.weight_kg, b.weight_kg) {
                (Some(x), Some(y)) => Some(y - x),
                _ => None,
            },
            _ => None,
        };
        Ok(summary)
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
