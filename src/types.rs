
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SetType {
    Warmup,
    Normal,
    Failure,
    Dropset,
}

/// Exercise type enum — matches reference TS `z.enum([...])` exactly
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExerciseType {
    WeightReps,
    RepsOnly,
    BodyweightReps,
    BodyweightAssistedReps,
    Duration,
    WeightDuration,
    DistanceDuration,
    ShortDistanceWeight,
}

/// Equipment category enum — matches reference TS `z.enum([...])` exactly
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentCategory {
    None,
    Barbell,
    Dumbbell,
    Kettlebell,
    Machine,
    Plate,
    ResistanceBand,
    Suspension,
    Other,
}

/// Muscle group enum — matches reference TS `z.enum([...])` exactly
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MuscleGroup {
    Abdominals,
    Shoulders,
    Biceps,
    Triceps,
    Forearms,
    Quadriceps,
    Hamstrings,
    Calves,
    Glutes,
    Abductors,
    Adductors,
    Lats,
    UpperBack,
    Traps,
    LowerBack,
    Chest,
    Cardio,
    Neck,
    FullBody,
    Other,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Set {
    pub index: i32,
    #[serde(rename = "type")]
    pub set_type: SetType,
    pub weight_kg: Option<f64>,
    pub reps: Option<i32>,
    pub distance_meters: Option<f64>,
    pub duration_seconds: Option<i32>,
    pub rpe: Option<f64>,
    pub custom_metric: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Exercise {
    pub index: i32,
    pub title: String,
    pub notes: Option<String>,
    pub exercise_template_id: Option<String>,
    pub supersets_id: Option<i32>,
    #[serde(default)]
    pub sets: Vec<Set>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Workout {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub routine_id: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub updated_at: String,
    pub created_at: String,
    #[serde(default)]
    pub exercises: Vec<Exercise>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct RoutineFolder {
    pub id: i32,
    pub index: i32,
    pub title: String,
    pub updated_at: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Routine {
    pub id: String,
    pub title: String,
    pub folder_id: Option<i32>,
    pub updated_at: String,
    pub created_at: String,
    #[serde(default)]
    pub exercises: Vec<Exercise>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct ExerciseTemplate {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub template_type: String,
    pub primary_muscle_group: String,
    #[serde(default)]
    pub secondary_muscle_groups: Vec<String>,
    pub is_custom: bool,
}

/// Mirrors Hevy's webhook object shape (kept for documentation; tool responses use WebhookResponse)
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Webhook {
    pub id: Option<String>,
    pub url: String,
    pub events: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct WorkoutListSchema {
    pub page: i32,
    pub page_count: i32,
    pub workouts: Vec<Workout>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct RoutineListSchema {
    pub page: i32,
    pub page_count: i32,
    pub routines: Vec<Routine>,
}

/// Renamed field `routine_folders` to match actual Hevy API response key.
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct FolderListSchema {
    pub page: i32,
    pub page_count: i32,
    pub routine_folders: Vec<RoutineFolder>,
}

/// Renamed field `exercise_templates` to match actual Hevy API response key.
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct TemplateListSchema {
    pub page: i32,
    pub page_count: i32,
    pub exercise_templates: Vec<ExerciseTemplate>,
}

/// Mirrors Hevy's webhook list shape (kept for documentation; not currently used by tools)
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct WebhookListSchema {
    pub webhooks: Vec<Webhook>,
}

/// Input types for typed parameters

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct WorkoutInput {
    /// Workout title
    pub title: String,
    pub description: Option<String>,
    /// ISO 8601 datetime (e.g. 2024-08-14T12:00:00Z)
    pub start_time: String,
    /// ISO 8601 datetime (e.g. 2024-08-14T12:30:00Z)
    pub end_time: String,
    #[serde(default)]
    pub is_private: bool,
    pub exercises: Vec<WorkoutExerciseInput>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct WorkoutExerciseInput {
    pub exercise_template_id: String,
    pub superset_id: Option<i32>,
    pub notes: Option<String>,
    pub sets: Vec<WorkoutSetInput>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct WorkoutSetInput {
    #[serde(rename = "type", default = "default_set_type")]
    pub set_type: SetType,
    pub weight_kg: Option<f64>,
    pub reps: Option<i32>,
    pub distance_meters: Option<i32>,
    pub duration_seconds: Option<i32>,
    pub rpe: Option<f64>,
    pub custom_metric: Option<f64>,
}

pub fn default_set_type() -> SetType {
    SetType::Normal
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct RoutineExerciseInput {
    pub exercise_template_id: String,
    pub superset_id: Option<i32>,
    pub notes: Option<String>,
    pub sets: Vec<RoutineSetInput>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct RoutineSetInput {
    #[serde(rename = "type", default = "default_set_type")]
    pub set_type: SetType,
    pub weight_kg: Option<f64>,
    pub reps: Option<i32>,
    pub distance_meters: Option<i32>,
    pub duration_seconds: Option<i32>,
    pub rpe: Option<f64>,
    pub custom_metric: Option<f64>,
}
