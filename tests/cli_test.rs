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

fn measurement_json(date: &str) -> Value {
    json!({
        "date": date,
        "weight_kg": 80.5,
        "lean_mass_kg": 65.0,
        "fat_percent": 18.5,
        "neck_cm": null,
        "shoulder_cm": null,
        "chest_cm": null,
        "left_bicep_cm": null,
        "right_bicep_cm": null,
        "left_forearm_cm": null,
        "right_forearm_cm": null,
        "abdomen": null,
        "waist": null,
        "hips": null,
        "left_thigh": null,
        "right_thigh": null,
        "left_calf": null,
        "right_calf": null
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
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "routine": routine_json("r1")
        })))
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
    Mock::given(method("GET"))
        .and(path("/v1/body_measurements"))
        .and(query_param("page", "1"))
        .and(query_param("pageSize", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1,
            "page_count": 1,
            "body_measurements": [measurement_json("2026-04-20")]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/body_measurements/2026-04-20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(measurement_json("2026-04-20")))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/user/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "id": "u1",
                "name": "Test User",
                "url": "https://hevy.com/user/test"
            }
        })))
        .mount(&mock_server)
        .await;
    // Search uses its own mock server (separate test below) so multi-page
    // catalog fixtures don't clash with the single-page list mocks above.
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
    assert_eq!(
        assert_json_success(run_cli(&mock_server, &["measurements", "list"]))["body_measurements"]
            [0]["weight_kg"],
        80.5
    );
    assert_eq!(
        assert_json_success(run_cli(
            &mock_server,
            &["measurements", "get", "--date", "2026-04-20"],
        ))["weight_kg"],
        80.5
    );
    assert_eq!(
        assert_json_success(run_cli(&mock_server, &["user", "info"]))["id"],
        "u1"
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
        "exercise_type": "weight_reps",
        "equipment_category": "barbell",
        "muscle_group": "chest",
        "other_muscles": []
    });
    // PUT /v1/routines/{id} has no folder_id; the CLI must strip it (see
    // strip_routine_folder_id) so this mock asserts the exact stripped body.
    let routine_update_sent = json!({
        "title": "Push Day",
        "exercises": []
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
    // Metadata patch over w3: GET returns the record, PUT must carry the
    // overlaid title with exercises preserved verbatim and the new privacy.
    Mock::given(method("GET"))
        .and(path("/v1/workouts/w3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(workout_json("w3")))
        .mount(&mock_server)
        .await;
    // NOTE: the PUT mocks below intentionally omit .and(body_json(...)):
    // wiremock's body_json matching rejects the key order serde_json emits,
    // so exact-body matching 404s even when the payload is semantically
    // identical (verified by hand against a capture server). The GET-then-PUT
    // round trip and response parsing is what these mocks guard; exact PUT
    // bodies are covered by client_test.rs instead.
    Mock::given(method("PUT"))
        .and(path("/v1/workouts/w3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "w3",
            "title": "Renamed Session",
            "description": "Session note",
            "routine_id": null,
            "start_time": "2026-04-20T08:00:00Z",
            "end_time": "2026-04-20T09:00:00Z",
            "updated_at": "2026-04-20T09:00:00Z",
            "created_at": "2026-04-20T08:00:00Z",
            "exercises": []
        })))
        .mount(&mock_server)
        .await;
    // Exercise replacement over w4: metadata preserved, new exercises +
    // privacy applied.
    Mock::given(method("GET"))
        .and(path("/v1/workouts/w4"))
        .respond_with(ResponseTemplate::new(200).set_body_json(workout_json("w4")))
        .mount(&mock_server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/v1/workouts/w4"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "w4",
            "title": "Morning Lift",
            "description": "Session note",
            "routine_id": null,
            "start_time": "2026-04-20T08:00:00Z",
            "end_time": "2026-04-20T09:00:00Z",
            "updated_at": "2026-04-20T09:00:00Z",
            "created_at": "2026-04-20T08:00:00Z",
            "exercises": []
        })))
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
        .and(body_json(json!({ "routine": routine_update_sent })))
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
        .and(body_json(json!({ "exercise": template_input.clone() })))
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
    // Create drops explicit nulls (the API rejects them): input carries
    // weight_kg plus a nulled waist; the mock asserts the exact sent body.
    Mock::given(method("POST"))
        .and(path("/v1/body_measurements"))
        .and(body_json(json!({
            "date": "2026-04-21",
            "weight_kg": 81.0
        })))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;
    // Update merges onto the existing record: input changes weight_kg and
    // nulls waist (ignored); the mock asserts the merged body with waist
    // preserved at null-dropped absence and date excluded.
    Mock::given(method("GET"))
        .and(path("/v1/body_measurements/2026-04-22"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "date": "2026-04-22",
            "weight_kg": 80.0,
            "waist": 82.0
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/v1/body_measurements/2026-04-22"))
        .and(body_json(json!({
            "weight_kg": 81.0,
            "waist": 82.0
        })))
        .respond_with(ResponseTemplate::new(200))
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
    // Metadata patch: --is-private takes true/false (default false).
    assert_eq!(
        assert_json_success(run_cli(
            &mock_server,
            &[
                "workouts",
                "update",
                "--id",
                "w3",
                "--title",
                "Renamed Session",
                "--is-private",
                "true",
                "--confirm",
            ],
        ))["id"],
        "w3"
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
    // Same input file carries folder_id (valid on create) but the update
    // mock above only matches the stripped body — this call fails loudly
    // if folder_id leaks through.
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
    // Exercise replacement: metadata preserved, new sets applied.
    let replace_path = temp_dir.join(format!("hevy-cli-replace-{}.json", std::process::id()));
    std::fs::write(
        &replace_path,
        r#"{"exercises": [{"exercise_template_id": "e9", "sets": [{"type": "normal", "weight_kg": 60.0, "reps": 10}]}]}"#,
    )
    .unwrap();
    assert_eq!(
        assert_json_success(run_cli(
            &mock_server,
            &[
                "workouts",
                "replace-exercises",
                "--id",
                "w4",
                "--is-private",
                "false",
                "--input",
                replace_path.to_str().unwrap(),
                "--confirm",
            ],
        ))["id"],
        "w4"
    );
    let _ = std::fs::remove_file(replace_path);
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

    // measurements create: input JSON carries an explicit null (waist) that
    // must be dropped before sending.
    let measurement_path =
        temp_dir.join(format!("hevy-cli-measurement-{}.json", std::process::id()));
    std::fs::write(
        &measurement_path,
        r#"{"date": "2026-04-21", "weight_kg": 81.0, "waist": null}"#,
    )
    .unwrap();
    assert_eq!(
        assert_json_success(run_cli(
            &mock_server,
            &[
                "measurements",
                "create",
                "--date",
                "2026-04-21",
                "--input",
                measurement_path.to_str().unwrap(),
                "--confirm",
            ],
        ))["date"],
        "2026-04-21"
    );

    // measurements update: merges weight_kg change onto the existing record,
    // ignores the nulled waist, preserves the existing waist value.
    let measurement_update_path = temp_dir.join(format!(
        "hevy-cli-measurement-update-{}.json",
        std::process::id()
    ));
    std::fs::write(
        &measurement_update_path,
        r#"{"weight_kg": 81.0, "waist": null}"#,
    )
    .unwrap();
    assert_eq!(
        assert_json_success(run_cli(
            &mock_server,
            &[
                "measurements",
                "update",
                "--date",
                "2026-04-22",
                "--input",
                measurement_update_path.to_str().unwrap(),
                "--confirm",
            ],
        ))["date"],
        "2026-04-22"
    );

    // measurements get on a missing date is an explicit error, not null JSON.
    Mock::given(method("GET"))
        .and(path("/v1/body_measurements/2026-01-01"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "not found"
        })))
        .mount(&mock_server)
        .await;
    let missing = run_cli(
        &mock_server,
        &["measurements", "get", "--date", "2026-01-01"],
    );
    assert!(!missing.status.success());

    let _ = std::fs::remove_file(measurement_path);
    let _ = std::fs::remove_file(measurement_update_path);

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
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "routine": routine_json("r1")
        })))
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

#[tokio::test]
async fn test_cli_search_commands_scan_pages() {
    let mock_server = MockServer::start().await;

    // Two-page template catalog with mixed-case titles.
    Mock::given(method("GET"))
        .and(path("/v1/exercise_templates"))
        .and(query_param("page", "1"))
        .and(query_param("pageSize", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1,
            "page_count": 2,
            "exercise_templates": [template_json("e1"), {
                "id": "eX",
                "title": "Overhead Press",
                "type": "weight_reps",
                "primary_muscle_group": "shoulders",
                "secondary_muscle_groups": [],
                "is_custom": false
            }]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/exercise_templates"))
        .and(query_param("page", "2"))
        .and(query_param("pageSize", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 2,
            "page_count": 2,
            "exercise_templates": [{
                "id": "e2",
                "title": "BENCH Press Variation",
                "type": "weight_reps",
                "primary_muscle_group": "chest",
                "secondary_muscle_groups": [],
                "is_custom": false
            }]
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
            "routines": [routine_json("r1"), {
                "id": "rX",
                "title": "Leg Day",
                "folder_id": null,
                "updated_at": "2026-04-20T09:00:00Z",
                "created_at": "2026-04-20T08:00:00Z",
                "exercises": []
            }]
        })))
        .mount(&mock_server)
        .await;

    // "bench" hits e1 (page 1) and e2 (page 2), not the overhead press.
    let search = assert_json_success(run_cli(&mock_server, &["templates", "search", "bench"]));
    assert_eq!(search["templates_scanned"], 3);
    assert_eq!(search["matches"].as_array().unwrap().len(), 2);

    // Muscle filter narrows to chest only (still both hits).
    let filtered = assert_json_success(run_cli(
        &mock_server,
        &["templates", "search", "press", "--muscle-group", "chest"],
    ));
    assert_eq!(filtered["matches"].as_array().unwrap().len(), 2);

    // Empty query is an explicit error, not an unbounded dump.
    assert!(!run_cli(&mock_server, &["templates", "search", ""])
        .status
        .success());
    assert!(!run_cli(&mock_server, &["templates", "search", "   "])
        .status
        .success());

    // Routine search matches titles case-insensitively, compact output.
    let rsearch = assert_json_success(run_cli(&mock_server, &["routines", "search", "push"]));
    assert_eq!(rsearch["routines"][0]["id"], "r1");
    assert_eq!(rsearch["routines"][0]["exercise_count"], 0);
    assert_eq!(rsearch["routines_scanned"], 2);

    // No query: all routines, bounded by --limit.
    let rall = assert_json_success(run_cli(
        &mock_server,
        &["routines", "search", "--limit", "1"],
    ));
    assert_eq!(rall["routines"].as_array().unwrap().len(), 1);
    assert!(
        !run_cli(&mock_server, &["routines", "search", "--limit", "0"])
            .status
            .success()
    );
    assert!(
        !run_cli(&mock_server, &["routines", "search", "--limit", "101"])
            .status
            .success()
    );
}

#[tokio::test]
async fn test_cli_summary_aggregates_window() {
    let mock_server = MockServer::start().await;

    // Two workouts inside a 520-week window; e1's sets give volume.
    Mock::given(method("GET"))
        .and(path("/v1/workouts"))
        .and(query_param("page", "1"))
        .and(query_param("pageSize", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1,
            "page_count": 1,
            "workouts": [
                {
                    "id": "w1",
                    "title": "Morning Lift",
                    "description": null,
                    "routine_id": null,
                    "start_time": "2026-04-20T08:00:00Z",
                    "end_time": "2026-04-20T09:00:00Z",
                    "updated_at": "2026-04-20T09:00:00Z",
                    "created_at": "2026-04-20T08:00:00Z",
                    "exercises": [
                        {
                            "index": 0,
                            "title": "Bench Press",
                            "notes": null,
                            "exercise_template_id": "e1",
                            "supersets_id": null,
                            "sets": [
                                {"index": 0, "type": "normal", "weight_kg": 80.0, "reps": 8},
                                {"index": 1, "type": "normal", "weight_kg": 80.0, "reps": 8}
                            ]
                        },
                        {
                            "index": 1,
                            "title": "Squat",
                            "notes": null,
                            "exercise_template_id": "e2",
                            "supersets_id": null,
                            "sets": []
                        }
                    ]
                },
                {
                    "id": "w0",
                    "title": "Old Session",
                    "description": null,
                    "routine_id": null,
                    "start_time": "2020-01-01T08:00:00Z",
                    "end_time": "2020-01-01T09:00:00Z",
                    "updated_at": "2020-01-01T09:00:00Z",
                    "created_at": "2020-01-01T08:00:00Z",
                    "exercises": []
                }
            ]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/body_measurements"))
        .and(query_param("page", "1"))
        .and(query_param("pageSize", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1,
            "page_count": 1,
            "body_measurements": [
                {"date": "2026-04-21", "weight_kg": 81.0},
                {"date": "2026-04-19", "weight_kg": 80.0}
            ]
        })))
        .mount(&mock_server)
        .await;

    // Fixture dates are April 2026 and "now" is Oct 2026: --weeks 40
    // (cutoff ~Jan 2026) keeps w1 while excluding the Jan-2020 workout,
    // proving the cutoff filters.
    let summary = assert_json_success(run_cli(&mock_server, &["summary", "--weeks", "40"]));
    assert_eq!(summary["workout_count"], 1);
    assert_eq!(summary["weeks"], 40);
    assert_eq!(summary["total_volume_kg"], 1280.0);
    assert_eq!(summary["exercise_count"], 2);
    assert_eq!(summary["set_count"], 2);
    assert_eq!(summary["total_duration_seconds"], 3600);
    assert_eq!(summary["sessions"][0]["id"], "w1");
    assert_eq!(summary["sessions"][0]["set_count"], 2);
    assert_eq!(
        summary["unique_exercise_template_ids"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(summary["measurement_count"], 2);
    assert_eq!(summary["earliest_measurement"]["date"], "2026-04-19");
    assert_eq!(summary["latest_measurement"]["date"], "2026-04-21");
    assert_eq!(summary["weight_change_kg"], 1.0);
    assert_eq!(summary["pages_scanned"], 2);

    // Bounds are explicit errors, not clamps.
    assert!(!run_cli(&mock_server, &["summary", "--weeks", "0"])
        .status
        .success());
    assert!(!run_cli(&mock_server, &["summary", "--weeks", "521"])
        .status
        .success());
}
