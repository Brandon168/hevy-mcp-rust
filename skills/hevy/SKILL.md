---
name: hevy
description: Use when working with live Hevy fitness data through the hevy-cli binary, including workouts, routines, routine folders, exercise templates, exercise history, exports, and webhooks.
---

# Hevy

Use `hevy-cli` for live Hevy account operations. This skill is for command-driven Hevy API access, not general fitness advice unless live account data is needed.

## Requirements

- `HEVY_API_KEY` must be set in the agent process environment, or pass `--hevy-api-key`.
- Prefer `bin/hevy-cli` relative to this skill directory when present. Fall back to `hevy-cli` from `PATH`.
- Keep stdout as JSON for downstream use. Treat stderr as diagnostics. Use `--output pretty` only for human inspection.
- Do not print or expose API keys, bearer tokens, `.env` contents, or command output that contains secrets.
- Use `auth test` first when the user asks whether Hevy access is working.

## Command Selection

- Access check: `hevy-cli auth test`
- Recent workout summaries: `hevy-cli workouts list --page 1 --page-size 10`
- Specific workout details: `hevy-cli workouts get --id <workout_id>`
- Full workout history window: `hevy-cli export workouts --weeks <n> --full`
- Routine context plus recent logs: `hevy-cli export routine-bundle --routine-id <routine_id> --weeks <n>`
- Exercise history: `hevy-cli exercises history --template-id <template_id> --start-date <yyyy-mm-dd> --end-date <yyyy-mm-dd>`
- Webhooks: use only when the user explicitly asks about webhook subscriptions.

## Read Commands

```bash
hevy-cli auth test
hevy-cli workouts list --page 1 --page-size 10
hevy-cli workouts get --id <workout_id>
hevy-cli workouts count
hevy-cli workouts events --since <iso_timestamp> --page 1 --page-size 10
hevy-cli routines list --page 1 --page-size 10
hevy-cli routines get --id <routine_id>
hevy-cli folders list --page 1 --page-size 10
hevy-cli folders get --id <folder_id>
hevy-cli templates list --page 1 --page-size 100
hevy-cli templates get --id <template_id>
hevy-cli exercises history --template-id <template_id> --start-date 2026-01-01 --end-date 2026-01-31
hevy-cli webhooks get
```

## Export Commands

Use exports when the user asks for workout history, full details, routine bundles, or files/artifacts that combine logs with routine context.

```bash
hevy-cli export workouts --weeks 3 --full
hevy-cli export routine-bundle --routine-id <routine_id> --weeks 3
```

`export workouts --full` fetches each workout by ID so session `description` and per-exercise `exercises[].notes` are preserved in `workoutLogs`.

## Write Safety

Write commands require `--confirm`. Do not add it until the user has clearly asked to mutate Hevy data. For vague requests, inspect first and summarize the intended mutation before writing.

```bash
hevy-cli workouts create --input workout.json --confirm
hevy-cli workouts update --id <workout_id> --input workout.json --confirm
hevy-cli routines create --input routine.json --confirm
hevy-cli routines update --id <routine_id> --input routine.json --confirm
hevy-cli folders create --title "New Folder" --confirm
hevy-cli templates create --input template.json --confirm
hevy-cli webhooks create --url https://example.com/hevy --confirm
hevy-cli webhooks delete --confirm
```

`--input -` reads JSON from stdin. JSON payloads may be either the raw object or the Hevy API wrapper object, such as `{ "workout": ... }`, `{ "routine": ... }`, or `{ "exercise_template": ... }`.

## Agent Workflow

1. Resolve the binary: use `bin/hevy-cli` if it exists, otherwise `hevy-cli`.
2. For live data requests, run the smallest read command that answers the question.
3. For "full details" workout requests, use `workouts get` for specific IDs or `export workouts --weeks <n> --full` for date windows.
4. For routine bundle requests, list routines if needed, then use `export routine-bundle --routine-id <id> --weeks <n>`.
5. For mutations, prefer `--input -` when generating JSON dynamically, run the matching write command with `--confirm`, then read back the changed resource when practical.
