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
