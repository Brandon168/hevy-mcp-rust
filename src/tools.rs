
use crate::client::HevyClient;
use crate::types::*;
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    tool, tool_handler, tool_router, Json,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A struct that groups all Hevy tools and implements `ToolRouter`.
#[derive(Clone)]
pub struct HevyTools {
    pub client: Arc<HevyClient>,
    pub tool_router: ToolRouter<Self>,
}

// ─── Pagination ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PaginationParams {
    /// Page number (1-indexed)
    #[schemars(range(min = 1))]
    pub page: u32,
    /// Number of results per page (must be between 1 and 10)
    #[schemars(range(min = 1, max = 10))]
    pub page_size: u32,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct TemplatePaginationParams {
    /// Page number (1-indexed)
    #[schemars(range(min = 1))]
    pub page: u32,
    /// Number of results per page (must be between 1 and 100)
    #[schemars(range(min = 1, max = 100))]
    pub page_size: u32,
}

// ─── Workout params ──────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GetWorkoutParams {
    /// The unique workout ID
    pub id: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GetEventsParams {
    /// Page number (1-indexed)
    #[schemars(range(min = 1))]
    pub page: u32,
    /// Number of results per page (must be between 1 and 10)
    #[schemars(range(min = 1, max = 10))]
    pub page_size: u32,
    /// ISO 8601 date-time; only events after this timestamp are returned
    pub since: String,
}

/// Tool input: create a new workout (typed, no opaque serde_json::Value)
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct CreateWorkoutParams {
    pub workout: WorkoutInput,
}

/// Tool input: update an existing workout
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct UpdateWorkoutParams {
    /// The unique workout ID to update
    pub id: String,
    pub workout: WorkoutInput,
}

// ─── Routine params ──────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GetRoutineParams {
    /// The unique routine ID
    pub id: String,
}

/// Tool input: create a new routine
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct CreateRoutineParams {
    /// Routine title
    pub title: String,
    /// Optional folder ID to assign the routine to
    pub folder_id: Option<i32>,
    pub exercises: Vec<RoutineExerciseInput>,
}

/// Tool input: update an existing routine
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct UpdateRoutineParams {
    /// The unique routine ID to update
    pub id: String,
    /// Routine title
    pub title: String,
    /// Optional folder ID to assign the routine to
    pub folder_id: Option<i32>,
    pub exercises: Vec<RoutineExerciseInput>,
}

// ─── Folder params ───────────────────────────────────────────────────────────

/// Tool input: create a new routine folder
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct CreateFolderParams {
    /// Name for the new routine folder
    pub title: String,
}

/// Tool input: fetch a single routine folder by ID
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GetFolderParams {
    /// The unique routine folder ID
    pub id: String,
}

// ─── Template params ─────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GetTemplateParams {
    /// The unique exercise template ID
    pub id: String,
}

/// Tool input: get exercise history for a specific template, with optional date range
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GetExerciseHistoryParams {
    /// The exercise template ID to get history for
    pub exercise_template_id: String,
    /// ISO 8601 start date for filtering (optional, e.g. 2024-01-01)
    pub start_date: Option<String>,
    /// ISO 8601 end date for filtering (optional, e.g. 2024-12-31)
    pub end_date: Option<String>,
}

/// Tool input: create a custom exercise template.
/// Uses typed enums so JSON Schema carries enum constraint info (matches reference TS Zod schemas)
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct CreateExerciseTemplateParams {
    /// Exercise name / title
    pub title: String,
    /// Exercise type
    pub exercise_type: ExerciseType,
    /// Equipment category
    pub equipment_category: EquipmentCategory,
    /// Primary muscle group
    pub muscle_group: MuscleGroup,
    /// Additional secondary muscle groups
    #[serde(default)]
    pub other_muscles: Vec<MuscleGroup>,
}

// ─── Webhook params ─────────────────────────────────────────────────────────

/// Tool input: create a new webhook subscription
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct CreateWebhookParams {
    /// The HTTPS URL that will receive POST requests for workout events
    pub url: String,
}

// ─── Misc ────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct EmptyParams {}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct WorkoutCountResponse {
    pub count: u32,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct PaginatedWorkoutEvents {
    pub page: u32,
    pub page_count: u32,
    pub events: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct SuccessResponse {
    pub status: String,
}

/// Typed response wrapper for exercise history (avoids serde_json::Value schema issues)
#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct ExerciseHistoryResponse {
    pub page: Option<i32>,
    pub page_count: Option<i32>,
    pub exercise_history: Vec<serde_json::Value>,
}

/// Typed response wrapper for exercise template creation
#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct ExerciseTemplateResponse {
    pub exercise_template: Option<serde_json::Value>,
}

/// Typed response wrapper for webhook subscription
#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct WebhookResponse {
    pub webhook: Option<serde_json::Value>,
}

// ─── ServerHandler ───────────────────────────────────────────────────────────

use rmcp::handler::server::ServerHandler;

#[tool_handler(router = self.tool_router)]
impl ServerHandler for HevyTools {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            server_info: rmcp::model::Implementation {
                name: "hevy-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: None,
                description: None,
                icons: None,
                website_url: None,
            },
            capabilities: rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}

// ─── Tool implementations ─────────────────────────────────────────────────────

#[tool_router]
impl HevyTools {
    pub fn new(client: Arc<HevyClient>) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    // ─── Workouts ────────────────────────────────────────────────────────

    #[tool(
        name = "get-workouts",
        description = "Get a paginated list of workouts. Returns workout details including title, description, start/end times, and exercises performed. Results are ordered from newest to oldest (most recent first). page_size must be between 1 and 10. To fetch recent workouts, use page=1 and a small page_size (e.g. 5), then filter the results by date."
    )]
    async fn get_workouts(
        &self,
        params: Parameters<PaginationParams>,
    ) -> Result<Json<WorkoutListSchema>, String> {
        let res = self
            .client
            .get_workouts(params.0.page, params.0.page_size)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(res))
    }

    #[tool(
        name = "get-workout",
        description = "Get complete details of a specific workout by ID. Returns all workout information including title, description, start/end times, and detailed exercise data."
    )]
    async fn get_workout(
        &self,
        params: Parameters<GetWorkoutParams>,
    ) -> Result<Json<Workout>, String> {
        let res = self
            .client
            .get_workout(&params.0.id)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(res))
    }

    #[tool(
        name = "get-workout-count",
        description = "Get the total number of workouts on the account. Useful for pagination or statistics."
    )]
    async fn get_workout_count(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<Json<WorkoutCountResponse>, String> {
        let res = self
            .client
            .get_workout_count()
            .await
            .map_err(|e| e.to_string())?;
        let count_res: WorkoutCountResponse =
            serde_json::from_value(res).map_err(|e| e.to_string())?;
        Ok(Json(count_res))
    }

    #[tool(
        name = "get-workout-events",
        description = "Retrieve a paged list of workout events (updates or deletes) since a given date. Events are ordered from newest to oldest. page_size must be between 1 and 10. The intention is to allow clients to keep their local cache of workouts up to date without having to fetch the entire list of workouts."
    )]
    async fn get_workout_events(
        &self,
        params: Parameters<GetEventsParams>,
    ) -> Result<Json<PaginatedWorkoutEvents>, String> {
        let res = self
            .client
            .get_workout_events(params.0.page, params.0.page_size, &params.0.since)
            .await
            .map_err(|e| e.to_string())?;
        let events_res: PaginatedWorkoutEvents =
            serde_json::from_value(res).map_err(|e| e.to_string())?;
        Ok(Json(events_res))
    }

    #[tool(
        name = "create-workout",
        description = "Create a new workout in your Hevy account. Requires title, start/end times, and at least one exercise with sets. Returns the complete workout details upon successful creation including the newly assigned workout ID."
    )]
    async fn create_workout(
        &self,
        params: Parameters<CreateWorkoutParams>,
    ) -> Result<Json<Workout>, String> {
        let payload = serde_json::json!({ "workout": params.0.workout });
        let res = self
            .client
            .create_workout(payload)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(res))
    }

    #[tool(
        name = "update-workout",
        description = "Update an existing workout by ID. You can modify the title, description, start/end times, privacy setting, and exercise data. Returns the updated workout with all changes applied."
    )]
    async fn update_workout(
        &self,
        params: Parameters<UpdateWorkoutParams>,
    ) -> Result<Json<Workout>, String> {
        let payload = serde_json::json!({ "workout": params.0.workout });
        let res = self
            .client
            .update_workout(&params.0.id, payload)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(res))
    }

    // ─── Routines ────────────────────────────────────────────────────────

    #[tool(
        name = "get-routines",
        description = "Get a paginated list of your workout routines, including custom and default routines. page_size must be between 1 and 10. Useful for browsing or searching your available routines."
    )]
    async fn get_routines(
        &self,
        params: Parameters<PaginationParams>,
    ) -> Result<Json<RoutineListSchema>, String> {
        let res = self
            .client
            .get_routines(params.0.page, params.0.page_size)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(res))
    }

    #[tool(
        name = "get-routine",
        description = "Get a routine by its ID using the direct endpoint. Returns all details for the specified routine."
    )]
    async fn get_routine(
        &self,
        params: Parameters<GetRoutineParams>,
    ) -> Result<Json<Routine>, String> {
        let res = self
            .client
            .get_routine(&params.0.id)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(res))
    }

    #[tool(
        name = "create-routine",
        description = "Create a new workout routine in your Hevy account. Requires a title and at least one exercise with sets. Optionally assign to a folder. Returns the full routine details including the new routine ID."
    )]
    async fn create_routine(
        &self,
        params: Parameters<CreateRoutineParams>,
    ) -> Result<Json<Routine>, String> {
        let payload = serde_json::json!({
            "routine": {
                "title": params.0.title,
                "folder_id": params.0.folder_id,
                "exercises": params.0.exercises,
            }
        });
        let res = self
            .client
            .create_routine(payload)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(res))
    }

    #[tool(
        name = "update-routine",
        description = "Update an existing routine by ID. You can modify the title, notes, and exercise configurations. Returns the updated routine with all changes applied."
    )]
    async fn update_routine(
        &self,
        params: Parameters<UpdateRoutineParams>,
    ) -> Result<Json<Routine>, String> {
        let payload = serde_json::json!({
            "routine": {
                "title": params.0.title,
                "folder_id": params.0.folder_id,
                "exercises": params.0.exercises,
            }
        });
        let res = self
            .client
            .update_routine(&params.0.id, payload)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(res))
    }

    // ─── Folders ─────────────────────────────────────────────────────────

    #[tool(
        name = "get-routine-folders",
        description = "Get a paginated list of your routine folders, including both default and custom folders. page_size must be between 1 and 10. Useful for organizing and browsing your workout routines."
    )]
    async fn get_folders(
        &self,
        params: Parameters<PaginationParams>,
    ) -> Result<Json<FolderListSchema>, String> {
        let res = self
            .client
            .get_folders(params.0.page, params.0.page_size)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(res))
    }

    /// Get a single routine folder by ID
    #[tool(
        name = "get-routine-folder",
        description = "Get complete details of a specific routine folder by its ID, including name, creation date, and associated routines."
    )]
    async fn get_routine_folder(
        &self,
        params: Parameters<GetFolderParams>,
    ) -> Result<Json<RoutineFolder>, String> {
        let res = self
            .client
            .get_folder(&params.0.id)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(res))
    }

    #[tool(
        name = "create-routine-folder",
        description = "Create a new routine folder in your Hevy account. Requires a name for the folder. Returns the full folder details including the new folder ID."
    )]
    async fn create_folder(
        &self,
        params: Parameters<CreateFolderParams>,
    ) -> Result<Json<RoutineFolder>, String> {
        let payload = serde_json::json!({ "routine_folder": { "title": params.0.title } });
        let res = self
            .client
            .create_folder(payload)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(res))
    }

    // ─── Templates ───────────────────────────────────────────────────────

    #[tool(
        name = "get-exercise-templates",
        description = "Get a paginated list of exercise templates (default and custom) with details like name, category, equipment, and muscle groups. page_size must be between 1 and 100. Useful for browsing or searching available exercises."
    )]
    async fn get_templates(
        &self,
        params: Parameters<TemplatePaginationParams>,
    ) -> Result<Json<TemplateListSchema>, String> {
        let res = self
            .client
            .get_templates(params.0.page, params.0.page_size)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(res))
    }

    #[tool(
        name = "get-exercise-template",
        description = "Get complete details of a specific exercise template by its ID, including name, category, equipment, muscle groups, and notes."
    )]
    async fn get_template(
        &self,
        params: Parameters<GetTemplateParams>,
    ) -> Result<Json<ExerciseTemplate>, String> {
        let res = self
            .client
            .get_template(&params.0.id)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(res))
    }

    /// Get exercise history for a specific exercise template
    #[tool(
        name = "get-exercise-history",
        description = "Get past sets for a specific exercise template, optionally filtered by start and end dates. Returns workout context and set details for each historical entry."
    )]
    async fn get_exercise_history(
        &self,
        params: Parameters<GetExerciseHistoryParams>,
    ) -> Result<Json<ExerciseHistoryResponse>, String> {
        let res = self
            .client
            .get_exercise_history(
                &params.0.exercise_template_id,
                params.0.start_date.as_deref(),
                params.0.end_date.as_deref(),
            )
            .await
            .map_err(|e| e.to_string())?;
        let typed: ExerciseHistoryResponse =
            serde_json::from_value(res).map_err(|e| e.to_string())?;
        Ok(Json(typed))
    }

    /// Create a custom exercise template
    #[tool(
        name = "create-exercise-template",
        description = "Create a custom exercise template with title, type, equipment, and muscle groups."
    )]
    async fn create_exercise_template(
        &self,
        params: Parameters<CreateExerciseTemplateParams>,
    ) -> Result<Json<ExerciseTemplateResponse>, String> {
        let payload = serde_json::json!({
            "exercise_template": {
                "title": params.0.title,
                "type": params.0.exercise_type,
                "equipment_category": params.0.equipment_category,
                "primary_muscle_group": params.0.muscle_group,
                "secondary_muscle_groups": params.0.other_muscles,
            }
        });
        let res = self
            .client
            .create_exercise_template(payload)
            .await
            .map_err(|e| e.to_string())?;
        let typed: ExerciseTemplateResponse =
            serde_json::from_value(res).map_err(|e| e.to_string())?;
        Ok(Json(typed))
    }

    // ─── Webhooks ────────────────────────────────────────────────────────────

    #[tool(
        name = "get-webhook-subscription",
        description = "Get the current webhook subscription for this account. Returns the webhook URL and auth token if a subscription exists."
    )]
    async fn get_webhook_subscription(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<Json<WebhookResponse>, String> {
        let res = self
            .client
            .get_webhook_subscription()
            .await
            .map_err(|e| e.to_string())?;
        let typed: WebhookResponse =
            serde_json::from_value(res).map_err(|e| e.to_string())?;
        Ok(Json(typed))
    }

    #[tool(
        name = "create-webhook-subscription",
        description = "Create a new webhook subscription for this account. The webhook will receive POST requests when workouts are created. Your endpoint must respond with 200 OK within 5 seconds."
    )]
    async fn create_webhook_subscription(
        &self,
        params: Parameters<CreateWebhookParams>,
    ) -> Result<Json<WebhookResponse>, String> {
        let payload = serde_json::json!({ "webhook": { "url": params.0.url } });
        let res = self
            .client
            .create_webhook_subscription(payload)
            .await
            .map_err(|e| e.to_string())?;
        let typed: WebhookResponse =
            serde_json::from_value(res).map_err(|e| e.to_string())?;
        Ok(Json(typed))
    }

    #[tool(
        name = "delete-webhook-subscription",
        description = "Delete the current webhook subscription for this account. This will stop all webhook notifications."
    )]
    async fn delete_webhook_subscription(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<Json<SuccessResponse>, String> {
        self.client
            .delete_webhook_subscription()
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(SuccessResponse {
            status: "success".to_string(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::HevyClient;

    fn make_tools() -> HevyTools {
        let client = std::sync::Arc::new(HevyClient::new("test".to_string()).unwrap());
        HevyTools::new(client)
    }

    /// Phase 4: verify the tool count matches the reference TS implementation (20 tools).
    #[tokio::test]
    async fn test_tools_list_count() {
        let tools = make_tools();
        let list = tools.tool_router.list_all();
        assert_eq!(
            list.len(),
            20,
            "Expected 20 tools (matching reference TS), got {}",
            list.len()
        );
        let resp = rmcp::model::ListToolsResult {
            tools: list,
            meta: None,
            next_cursor: None,
        };
        let js = serde_json::to_string(&resp).unwrap();
        println!("Payload size: {}", js.len());
        assert!(!js.is_empty());
    }

    /// Phase 4: verify each tool's JSON Schema matches the reference TS Zod schemas.
    ///
    /// Checks:
    /// - All 20 expected tool names are present
    /// - Tools with typed inputs have a `properties` object in their schema
    /// - `create-exercise-template` has enum constraints for type/equipment/muscle
    /// - `get-workout` has the expected `id` property
    /// - `get-workout-events` has `page`, `page_size`, and `since` properties
    /// - `create-workout` has `workout` as a composite object property
    #[test]
    fn test_tool_schemas() {
        let tools = make_tools();
        let list = tools.tool_router.list_all();

        // Build a name → schema map for easy lookup
        let schema_map: std::collections::HashMap<String, serde_json::Value> = list
            .iter()
            .map(|t| {
                let schema: serde_json::Value =
                    serde_json::to_value(&t.input_schema).unwrap();
                (t.name.to_string(), schema)
            })
            .collect();

        // ── All expected tool names must be present ──────────────────────────
        let expected_names = [
            "get-workouts",
            "get-workout",
            "get-workout-count",
            "get-workout-events",
            "create-workout",
            "update-workout",
            "get-routines",
            "get-routine",
            "create-routine",
            "update-routine",
            "get-routine-folders",
            "get-routine-folder",
            "create-routine-folder",
            "get-exercise-templates",
            "get-exercise-template",
            "get-exercise-history",
            "create-exercise-template",
            "get-webhook-subscription",
            "create-webhook-subscription",
            "delete-webhook-subscription",
        ];
        for name in &expected_names {
            assert!(
                schema_map.contains_key(*name),
                "Missing tool: {name}"
            );
        }

        // ── get-workout must have an `id` property ───────────────────────────
        let gw = &schema_map["get-workout"];
        assert!(
            gw["properties"]["id"].is_object(),
            "get-workout schema missing `id` property: {gw}"
        );

        // ── get-workout-events must have page, page_size, since ──────────────
        let gwe = &schema_map["get-workout-events"];
        for field in &["page", "page_size", "since"] {
            assert!(
                gwe["properties"][field].is_object(),
                "get-workout-events schema missing `{field}` property: {gwe}"
            );
        }

        // ── create-workout must have a `workout` object property ─────────────
        let cw = &schema_map["create-workout"];
        assert!(
            cw["properties"]["workout"].is_object(),
            "create-workout schema missing `workout` property: {cw}"
        );

        // ── create-exercise-template must have enum constraints ──────────────
        // (This is the key schema-equivalence check vs the TS Zod enums)
        let cet = &schema_map["create-exercise-template"];
        let props = &cet["properties"];

        // exercise_type should be an enum
        let et_schema = &props["exercise_type"];
        let et_enum = et_schema.get("enum").or_else(|| {
            // schemars may nest it under `oneOf` or inline — also check `$ref`-resolved
            et_schema.get("oneOf")
        });
        assert!(
            et_enum.is_some() || et_schema.get("$ref").is_some(),
            "create-exercise-template.exercise_type should be an enum or $ref, got: {et_schema}"
        );

        // equipment_category should be an enum
        let eq_schema = &props["equipment_category"];
        let eq_enum = eq_schema.get("enum").or_else(|| eq_schema.get("oneOf"));
        assert!(
            eq_enum.is_some() || eq_schema.get("$ref").is_some(),
            "create-exercise-template.equipment_category should be an enum or $ref, got: {eq_schema}"
        );

        // muscle_group should be an enum
        let mg_schema = &props["muscle_group"];
        let mg_enum = mg_schema.get("enum").or_else(|| mg_schema.get("oneOf"));
        assert!(
            mg_enum.is_some() || mg_schema.get("$ref").is_some(),
            "create-exercise-template.muscle_group should be an enum or $ref, got: {mg_schema}"
        );

        // ── get-exercise-history must have exerciseTemplateId / exercise_template_id ─
        let geh = &schema_map["get-exercise-history"];
        assert!(
            geh["properties"]["exercise_template_id"].is_object(),
            "get-exercise-history schema missing `exercise_template_id` property: {geh}"
        );

        // ── page_size must carry maximum:10 so models don't guess out-of-range values ─
        // This is the constraint that prevents the "400: pageSize must be <= 10" error.
        let gw_paginated = &schema_map["get-workouts"];
        let page_size_schema = &gw_paginated["properties"]["page_size"];
        assert_eq!(
            page_size_schema["maximum"],
            serde_json::json!(10),
            "get-workouts.page_size must have maximum:10 in its JSON Schema to guide models; got: {page_size_schema}"
        );

        // ── get-exercise-templates allows up to 100 items ───────────────────
        let get_templates = &schema_map["get-exercise-templates"];
        let template_page_size = &get_templates["properties"]["page_size"];
        assert_eq!(
            template_page_size["maximum"],
            serde_json::json!(100),
            "get-exercise-templates.page_size should allow maximum:100; got: {template_page_size}"
        );

        println!("All tool schema checks passed!");
    }
}

