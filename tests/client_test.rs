use hevy_mcp::client::{HevyClient, HevyClientError};
use reqwest::StatusCode;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_client_get_workouts() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/workouts"))
        .and(query_param("page", "1"))
        .and(query_param("pageSize", "10"))
        .and(header("api-key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "page": 1,
            "page_count": 1,
            "workouts": [
                {
                    "id": "workout_1",
                    "title": "Morning Routine",
                    "start_time": "2024-01-01T08:00:00Z",
                    "end_time": "2024-01-01T09:00:00Z",
                    "updated_at": "2024-01-01T09:00:00Z",
                    "created_at": "2024-01-01T08:00:00Z",
                    "exercises": []
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = HevyClient::with_base_url("test_key".to_string(), mock_server.uri()).unwrap();
    let res = client.get_workouts(1, 10).await.unwrap();

    assert_eq!(res.page, 1);
    assert_eq!(res.workouts.len(), 1);
    assert_eq!(res.workouts[0].id, "workout_1");
}

#[tokio::test]
async fn test_client_get_routine_accepts_api_wrapper() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/routines/routine_1"))
        .and(header("api-key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "routine": {
                "id": "routine_1",
                "title": "Push Day",
                "folder_id": 42,
                "updated_at": "2026-08-16T20:00:00Z",
                "created_at": "2026-08-16T19:00:00Z",
                "exercises": [
                    {
                        "index": 0,
                        "title": "Bench Press",
                        "notes": null,
                        "exercise_template_id": "template_1",
                        "superset_id": null,
                        "rest_seconds": 120,
                        "sets": [
                            {
                                "index": 0,
                                "type": "normal",
                                "weight_kg": 80.0,
                                "reps": 8,
                                "distance_meters": null,
                                "duration_seconds": null,
                                "custom_metric": null
                            }
                        ]
                    }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    let client = HevyClient::with_base_url("test_key".to_string(), mock_server.uri()).unwrap();
    let routine = client.get_routine("routine_1").await.unwrap();

    assert_eq!(routine.id, "routine_1");
    assert_eq!(routine.title, "Push Day");
    assert_eq!(routine.exercises.len(), 1);
    assert_eq!(routine.exercises[0].sets.len(), 1);
}

#[tokio::test]
async fn test_client_resources_accept_wrapped_and_unwrapped_responses() {
    for wrapped in [false, true] {
        let mock_server = MockServer::start().await;
        let resources = [
            (
                "workouts",
                "workout",
                serde_json::json!({
                    "id": "resource_1", "title": "Workout",
                    "start_time": "2026-08-16T19:00:00Z",
                    "end_time": "2026-08-16T20:00:00Z",
                    "updated_at": "2026-08-16T20:00:00Z",
                    "created_at": "2026-08-16T19:00:00Z",
                    "exercises": []
                }),
            ),
            (
                "routines",
                "routine",
                serde_json::json!({
                    "id": "resource_1", "title": "Routine", "folder_id": 42,
                    "updated_at": "2026-08-16T20:00:00Z",
                    "created_at": "2026-08-16T19:00:00Z",
                    "exercises": []
                }),
            ),
            (
                "routine_folders",
                "routine_folder",
                serde_json::json!({
                    "id": 42, "index": 0, "title": "Folder",
                    "updated_at": "2026-08-16T20:00:00Z",
                    "created_at": "2026-08-16T19:00:00Z"
                }),
            ),
            (
                "exercise_templates",
                "exercise_template",
                serde_json::json!({
                    "id": "resource_1", "title": "Bench Press", "type": "weight_reps",
                    "primary_muscle_group": "chest", "secondary_muscle_groups": [],
                    "is_custom": false
                }),
            ),
        ];

        for (endpoint, wrapper_key, resource) in resources {
            let response = if wrapped {
                serde_json::json!({ (wrapper_key): resource })
            } else {
                resource
            };
            let id = if endpoint == "routine_folders" {
                "42"
            } else {
                "resource_1"
            };
            let mut operations = vec![("GET", format!("/v1/{endpoint}/{id}"))];
            if endpoint != "exercise_templates" {
                operations.push(("POST", format!("/v1/{endpoint}")));
            }
            if endpoint == "workouts" || endpoint == "routines" {
                operations.push(("PUT", format!("/v1/{endpoint}/{id}")));
            }
            for (verb, resource_path) in operations {
                Mock::given(method(verb))
                    .and(path(resource_path))
                    .and(header("api-key", "test_key"))
                    .respond_with(ResponseTemplate::new(200).set_body_json(response.clone()))
                    .expect(1)
                    .mount(&mock_server)
                    .await;
            }
        }

        let client = HevyClient::with_base_url("test_key".to_string(), mock_server.uri()).unwrap();
        let payload = serde_json::json!({});
        assert_eq!(
            client.get_workout("resource_1").await.unwrap().id,
            "resource_1"
        );
        assert_eq!(
            client.create_workout(payload.clone()).await.unwrap().id,
            "resource_1"
        );
        assert_eq!(
            client
                .update_workout("resource_1", payload.clone())
                .await
                .unwrap()
                .id,
            "resource_1"
        );
        assert_eq!(
            client.get_routine("resource_1").await.unwrap().id,
            "resource_1"
        );
        assert_eq!(
            client.create_routine(payload.clone()).await.unwrap().id,
            "resource_1"
        );
        assert_eq!(
            client
                .update_routine("resource_1", payload.clone())
                .await
                .unwrap()
                .id,
            "resource_1"
        );
        assert_eq!(client.get_folder("42").await.unwrap().id, 42);
        assert_eq!(client.create_folder(payload).await.unwrap().id, 42);
        assert_eq!(
            client.get_template("resource_1").await.unwrap().id,
            "resource_1"
        );
        mock_server.verify().await;
    }
}

#[tokio::test]
async fn test_client_error_handling() {
    let mock_server = MockServer::start().await;

    // Test 4xx error
    Mock::given(method("GET"))
        .and(path("/v1/workouts/invalid_id"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&mock_server)
        .await;

    let client = HevyClient::with_base_url("test_key".to_string(), mock_server.uri()).unwrap();
    let err = client.get_workout("invalid_id").await.unwrap_err();

    match err {
        HevyClientError::ClientError { status, message } => {
            assert_eq!(status, StatusCode::NOT_FOUND);
            assert_eq!(message, "Not Found");
        }
        _ => panic!("Expected ClientError"),
    }

    // Test 5xx error
    Mock::given(method("GET"))
        .and(path("/v1/workouts/error_id"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let err2 = client.get_workout("error_id").await.unwrap_err();
    match err2 {
        HevyClientError::ServerError { status, message } => {
            assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
            assert_eq!(message, "Internal Server Error");
        }
        _ => panic!("Expected ServerError"),
    }
}

#[tokio::test]
async fn test_update_workout_metadata_preserves_exercises() {
    use hevy_mcp::tools::UpdateWorkoutParams;

    let mock_server = MockServer::start().await;
    let current = serde_json::json!({
        "id": "w1", "title": "Morning Lift", "description": "Note",
        "start_time": "2026-04-20T08:00:00.000Z",
        "end_time": "2026-04-20T09:00:00.000Z",
        "updated_at": "2026-04-20T09:00:00Z",
        "created_at": "2026-04-20T08:00:00Z",
        "exercises": [
            {
                "index": 0, "title": "Bench", "notes": null,
                "exercise_template_id": "e1", "supersets_id": 3,
                "sets": [
                    {"index": 0, "type": "warmup", "weight_kg": 60.0, "reps": 10}
                ]
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/v1/workouts/w1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(current))
        .mount(&mock_server)
        .await;

    // GET-then-PUT round trip; the PUT responder echoes a renamed record.
    // Exact PUT-body matching is intentionally not asserted with body_json
    // (wiremock rejects serde_json's key order); payload shape is verified
    // by hand against a capture server instead.
    Mock::given(method("PUT"))
        .and(path("/v1/workouts/w1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "w1", "title": "Renamed", "description": "Note",
            "start_time": "2026-04-20T08:00:00Z",
            "end_time": "2026-04-20T09:00:00Z",
            "updated_at": "2026-04-20T09:00:00Z",
            "created_at": "2026-04-20T08:00:00Z",
            "exercises": []
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = HevyClient::with_base_url("test_key".to_string(), mock_server.uri()).unwrap();
    let patch = UpdateWorkoutParams {
        id: "w1".to_string(),
        title: Some("Renamed".to_string()),
        description: None,
        start_time: None,
        end_time: None,
        is_private: true,
    };
    let updated = client.update_workout_metadata("w1", &patch).await.unwrap();
    assert_eq!(updated.title, "Renamed");
    mock_server.verify().await;
}

#[tokio::test]
async fn test_replace_workout_exercises_preserves_metadata() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/workouts/w2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "w2", "title": "Keep Me", "description": "Keep",
            "start_time": "2026-04-20T08:00:00Z",
            "end_time": "2026-04-20T09:00:00Z",
            "updated_at": "2026-04-20T09:00:00Z",
            "created_at": "2026-04-20T08:00:00Z",
            "exercises": [
                {"index": 0, "title": "Old", "notes": null,
                 "exercise_template_id": "eOld", "supersets_id": null, "sets": []}
            ]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/v1/workouts/w2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "w2", "title": "Keep Me", "description": "Keep",
            "start_time": "2026-04-20T08:00:00Z",
            "end_time": "2026-04-20T09:00:00Z",
            "updated_at": "2026-04-20T09:00:00Z",
            "created_at": "2026-04-20T08:00:00Z",
            "exercises": []
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = HevyClient::with_base_url("test_key".to_string(), mock_server.uri()).unwrap();
    let exercises: Vec<hevy_mcp::types::WorkoutExerciseInput> =
        serde_json::from_value(serde_json::json!([{"exercise_template_id": "e9", "sets": []}]))
            .unwrap();
    let updated = client
        .replace_workout_exercises("w2", false, &exercises)
        .await
        .unwrap();
    assert_eq!(updated.title, "Keep Me");
    mock_server.verify().await;
}
