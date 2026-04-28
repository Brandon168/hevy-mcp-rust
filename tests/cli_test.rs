use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Output, Stdio};
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn run_cli(mock_server: &MockServer, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_hevy-cli"))
        .args(args)
        .env("HEVY_API_KEY", "test_key")
        .env("HEVY_BASE_URL", mock_server.uri())
        .output()
        .expect("failed to run hevy-cli")
}

fn assert_json_success(output: Output) -> Value {
    if !output.status.success() {
        panic!(
            "hevy-cli failed\nstatus: {}\nstderr: {}\nstdout: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
    }
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

fn workout_json(id: &str) -> Value {
    json!({
        "id": id,
        "title": "Morning Lift",
        "description": "Session note",
        "routine_id": null,
        "start_time": "2026-04-20T08:00:00Z",
        "end_time": "2026-04-20T09:00:00Z",
        "updated_at": "2026-04-20T09:00:00Z",
        "created_at": "2026-04-20T08:00:00Z",
        "exercises": [
            {
                "index": 0,
                "title": "Bench Press",
                "notes": "Exercise note",
                "exercise_template_id": "e1",
                "supersets_id": null,
                "sets": []
            }
        ]
    })
}

fn routine_json(id: &str) -> Value {
    json!({
        "id": id,
        "title": "Push Day",
        "folder_id": null,
        "updated_at": "2026-04-20T09:00:00Z",
        "created_at": "2026-04-20T08:00:00Z",
        "exercises": []
    })
}

fn folder_json() -> Value {
    json!({
        "id": 1,
        "index": 0,
        "title": "Strength",
        "updated_at": "2026-04-20T09:00:00Z",
        "created_at": "2026-04-20T08:00:00Z"
    })
}

fn template_json(id: &str) -> Value {
    json!({
        "id": id,
        "title": "Bench Press",
        "type": "weight_reps",
        "primary_muscle_group": "chest",
        "secondary_muscle_groups": ["triceps"],
        "is_custom": false
    })
}

#[tokio::test]
async fn test_cli_read_commands_emit_json() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/workouts"))
        .and(query_param("page", "1"))
        .and(query_param("pageSize", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1,
            "page_count": 1,
            "workouts": [workout_json("w1")]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/workouts/w1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(workout_json("w1")))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/workouts/count"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "count": 12 })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/workouts/events"))
        .and(query_param("page", "1"))
        .and(query_param("pageSize", "10"))
        .and(query_param("since", "2026-04-01T00:00:00Z"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1,
            "page_count": 1,
            "events": []
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/routines"))
        .and(query_param("page", "1"))
        .and(query_param("pageSize", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1,
            "page_count": 1,
            "routines": [routine_json("r1")]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/routines/r1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(routine_json("r1")))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/routine_folders"))
        .and(query_param("page", "1"))
        .and(query_param("pageSize", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1,
            "page_count": 1,
            "routine_folders": [folder_json()]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/routine_folders/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(folder_json()))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/exercise_templates"))
        .and(query_param("page", "1"))
        .and(query_param("pageSize", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1,
            "page_count": 1,
            "exercise_templates": [template_json("e1")]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/exercise_templates/e1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(template_json("e1")))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/exercise_history/e1"))
        .and(query_param("start_date", "2026-04-01"))
        .and(query_param("end_date", "2026-04-30"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1,
            "page_count": 1,
            "exercise_history": []
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/webhooks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "webhook": {
                "id": "wh1",
                "url": "https://example.com/hevy",
                "events": ["workout.created"]
            }
        })))
        .mount(&mock_server)
        .await;

    assert_eq!(
        assert_json_success(run_cli(&mock_server, &["workouts", "list"]))["workouts"][0]["id"],
        "w1"
    );
    assert_eq!(
        assert_json_success(run_cli(&mock_server, &["workouts", "get", "--id", "w1"]))["id"],
        "w1"
    );
    assert_eq!(
        assert_json_success(run_cli(&mock_server, &["workouts", "count"]))["count"],
        12
    );
    assert_json_success(run_cli(
        &mock_server,
        &["workouts", "events", "--since", "2026-04-01T00:00:00Z"],
    ));
    assert_eq!(
        assert_json_success(run_cli(&mock_server, &["routines", "list"]))["routines"][0]["id"],
        "r1"
    );
    assert_eq!(
        assert_json_success(run_cli(&mock_server, &["routines", "get", "--id", "r1"]))["id"],
        "r1"
    );
    assert_eq!(
        assert_json_success(run_cli(&mock_server, &["folders", "list"]))["routine_folders"][0]
            ["id"],
        1
    );
    assert_eq!(
        assert_json_success(run_cli(&mock_server, &["folders", "get", "--id", "1"]))["id"],
        1
    );
    assert_eq!(
        assert_json_success(run_cli(&mock_server, &["templates", "list"]))["exercise_templates"][0]
            ["id"],
        "e1"
    );
    assert_eq!(
        assert_json_success(run_cli(&mock_server, &["templates", "get", "--id", "e1"]))["id"],
        "e1"
    );
    assert_json_success(run_cli(
        &mock_server,
        &[
            "exercises",
            "history",
            "--template-id",
            "e1",
            "--start-date",
            "2026-04-01",
            "--end-date",
            "2026-04-30",
        ],
    ));
    assert_eq!(
        assert_json_success(run_cli(&mock_server, &["webhooks", "get"]))["webhook"]["id"],
        "wh1"
    );
    assert_eq!(
        assert_json_success(run_cli(&mock_server, &["auth", "test"]))["status"],
        "ok"
    );
}

#[tokio::test]
async fn test_cli_write_commands_require_confirm_and_wrap_payloads() {
    let mock_server = MockServer::start().await;
    let workout_input = json!({
        "title": "Morning Lift",
        "description": "Session note",
        "start_time": "2026-04-20T08:00:00Z",
        "end_time": "2026-04-20T09:00:00Z",
        "is_private": false,
        "exercises": []
    });
    let routine_input = json!({
        "title": "Push Day",
        "folder_id": null,
        "exercises": []
    });
    let template_input = json!({
        "title": "Custom Press",
        "type": "weight_reps",
        "equipment_category": "barbell",
        "primary_muscle_group": "chest",
        "secondary_muscle_groups": []
    });

    Mock::given(method("POST"))
        .and(path("/v1/workouts"))
        .and(body_json(json!({ "workout": workout_input.clone() })))
        .respond_with(ResponseTemplate::new(200).set_body_json(workout_json("w2")))
        .mount(&mock_server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/v1/workouts/w2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(workout_json("w2")))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/routines"))
        .and(body_json(json!({ "routine": routine_input.clone() })))
        .respond_with(ResponseTemplate::new(200).set_body_json(routine_json("r2")))
        .mount(&mock_server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/v1/routines/r2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(routine_json("r2")))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/routine_folders"))
        .and(body_json(
            json!({ "routine_folder": { "title": "Strength" } }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(folder_json()))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/exercise_templates"))
        .and(body_json(
            json!({ "exercise_template": template_input.clone() }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "exercise_template": template_json("e2")
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/webhooks"))
        .and(body_json(
            json!({ "webhook": { "url": "https://example.com/hevy" } }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "webhook": {
                "id": "wh2",
                "url": "https://example.com/hevy",
                "events": ["workout.created"]
            }
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v1/webhooks"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let refused = run_cli(&mock_server, &["folders", "create", "--title", "Strength"]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("--confirm"));

    let temp_dir = std::env::temp_dir();
    let workout_path = temp_dir.join(format!("hevy-cli-workout-{}.json", std::process::id()));
    let routine_path = temp_dir.join(format!("hevy-cli-routine-{}.json", std::process::id()));
    std::fs::write(
        &workout_path,
        serde_json::to_string(&workout_input).unwrap(),
    )
    .unwrap();
    std::fs::write(
        &routine_path,
        serde_json::to_string(&routine_input).unwrap(),
    )
    .unwrap();

    assert_eq!(
        assert_json_success(run_cli(
            &mock_server,
            &[
                "workouts",
                "create",
                "--input",
                workout_path.to_str().unwrap(),
                "--confirm",
            ],
        ))["id"],
        "w2"
    );
    assert_eq!(
        assert_json_success(run_cli(
            &mock_server,
            &[
                "workouts",
                "update",
                "--id",
                "w2",
                "--input",
                workout_path.to_str().unwrap(),
                "--confirm",
            ],
        ))["id"],
        "w2"
    );
    assert_eq!(
        assert_json_success(run_cli(
            &mock_server,
            &[
                "routines",
                "create",
                "--input",
                routine_path.to_str().unwrap(),
                "--confirm",
            ],
        ))["id"],
        "r2"
    );
    assert_eq!(
        assert_json_success(run_cli(
            &mock_server,
            &[
                "routines",
                "update",
                "--id",
                "r2",
                "--input",
                routine_path.to_str().unwrap(),
                "--confirm",
            ],
        ))["id"],
        "r2"
    );
    assert_eq!(
        assert_json_success(run_cli(
            &mock_server,
            &["folders", "create", "--title", "Strength", "--confirm"],
        ))["id"],
        1
    );
    assert_eq!(
        assert_json_success(run_cli(
            &mock_server,
            &[
                "webhooks",
                "create",
                "--url",
                "https://example.com/hevy",
                "--confirm",
            ],
        ))["webhook"]["id"],
        "wh2"
    );
    assert_eq!(
        assert_json_success(run_cli(&mock_server, &["webhooks", "delete", "--confirm"]))["status"],
        "success"
    );

    let mut child = Command::new(env!("CARGO_BIN_EXE_hevy-cli"))
        .args(["templates", "create", "--input", "-", "--confirm"])
        .env("HEVY_API_KEY", "test_key")
        .env("HEVY_BASE_URL", mock_server.uri())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn hevy-cli");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(serde_json::to_string(&template_input).unwrap().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(assert_json_success(output)["exercise_template"]["id"], "e2");

    let _ = std::fs::remove_file(workout_path);
    let _ = std::fs::remove_file(routine_path);
}

#[tokio::test]
async fn test_cli_exports_preserve_full_notes() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/workouts"))
        .and(query_param("page", "1"))
        .and(query_param("pageSize", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1,
            "page_count": 1,
            "workouts": [workout_json("w1")]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/workouts/w1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(workout_json("w1")))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/routines/r1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(routine_json("r1")))
        .mount(&mock_server)
        .await;

    let export = assert_json_success(run_cli(
        &mock_server,
        &["export", "workouts", "--weeks", "520", "--full"],
    ));
    assert_eq!(export["workoutLogs"][0]["description"], "Session note");
    assert_eq!(
        export["workoutLogs"][0]["exercises"][0]["notes"],
        "Exercise note"
    );

    let bundle = assert_json_success(run_cli(
        &mock_server,
        &[
            "export",
            "routine-bundle",
            "--routine-id",
            "r1",
            "--weeks",
            "520",
        ],
    ));
    assert_eq!(bundle["routineBundle"]["id"], "r1");
    assert_eq!(bundle["workoutLogs"][0]["description"], "Session note");
}
