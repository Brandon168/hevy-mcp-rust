use hevy_mcp::types::{ExerciseTemplate, Routine, SetType, Workout};

#[test]
fn test_deserialize_workout() {
    let json_data = r#"{
        "id": "w_123",
        "title": "Leg Day",
        "description": "Heavy squats",
        "start_time": "2024-01-01T10:00:00Z",
        "end_time": "2024-01-01T11:00:00Z",
        "updated_at": "2024-01-01T11:00:00Z",
        "created_at": "2024-01-01T10:00:00Z",
        "exercises": [
            {
                "index": 0,
                "title": "Barbell Squat",
                "exercise_template_id": "t_123",
                "sets": [
                    {
                        "index": 0,
                        "type": "warmup",
                        "weight_kg": 60.0,
                        "reps": 10
                    },
                    {
                        "index": 1,
                        "type": "normal",
                        "weight_kg": 100.0,
                        "reps": 5
                    }
                ]
            }
        ]
    }"#;

    let workout: Workout = serde_json::from_str(json_data).unwrap();
    assert_eq!(workout.id, "w_123");
    assert_eq!(workout.title, "Leg Day");
    assert_eq!(workout.exercises.len(), 1);

    let ex = &workout.exercises[0];
    assert_eq!(ex.title, "Barbell Squat");
    assert_eq!(ex.sets.len(), 2);

    // Check set types (snake_case deserialization)
    assert!(matches!(ex.sets[0].set_type, SetType::Warmup));
    assert!(matches!(ex.sets[1].set_type, SetType::Normal));
}

#[test]
fn test_deserialize_exercise_template() {
    let json_data = r#"{
        "id": "t_456",
        "title": "Bench Press",
        "type": "weight_reps",
        "primary_muscle_group": "chest",
        "secondary_muscle_groups": ["triceps", "shoulders"],
        "is_custom": false
    }"#;

    let template: ExerciseTemplate = serde_json::from_str(json_data).unwrap();
    assert_eq!(template.id, "t_456");
    assert_eq!(template.template_type, "weight_reps");
    assert_eq!(template.primary_muscle_group, "chest");
    assert_eq!(template.secondary_muscle_groups.len(), 2);
}

#[test]
fn test_deserialize_routine() {
    let json_data = r#"{
        "id": "r_789",
        "title": "Push Day Routine",
        "folder_id": 1,
        "updated_at": "2024-02-01T10:00:00Z",
        "created_at": "2024-02-01T09:00:00Z",
        "exercises": []
    }"#;

    let routine: Routine = serde_json::from_str(json_data).unwrap();
    assert_eq!(routine.id, "r_789");
    assert_eq!(routine.folder_id, Some(1));
    assert_eq!(routine.exercises.len(), 0);
}
