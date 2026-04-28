use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration, Utc};
use clap::{Args, Parser, Subcommand, ValueEnum};
use hevy_mcp::client::HevyClient;
use serde::Serialize;
use serde_json::{json, Value};
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hevy-cli", about = "Command-line client for the Hevy API")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format. JSON is the stable agent-facing contract.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json, global = true)]
    output: OutputFormat,

    /// Hevy API key. Defaults to HEVY_API_KEY.
    #[arg(long, env = "HEVY_API_KEY", hide_env_values = true, global = true)]
    hevy_api_key: Option<String>,

    /// Optional Hevy base URL, primarily for mock testing.
    #[arg(
        long,
        env = "HEVY_BASE_URL",
        hide_env_values = true,
        default_value = "https://api.hevyapp.com",
        global = true
    )]
    hevy_base_url: String,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
    Pretty,
}

#[derive(Subcommand)]
enum Commands {
    Workouts(WorkoutsCommand),
    Routines(RoutinesCommand),
    Folders(FoldersCommand),
    Templates(TemplatesCommand),
    Exercises(ExercisesCommand),
    Webhooks(WebhooksCommand),
    Export(ExportCommand),
    Auth(AuthCommand),
}

#[derive(Args)]
struct WorkoutsCommand {
    #[command(subcommand)]
    command: WorkoutsSubcommand,
}

#[derive(Subcommand)]
enum WorkoutsSubcommand {
    List(PageArgs),
    Get(IdArgs),
    Count,
    Events(EventsArgs),
    Create(InputConfirmArgs),
    Update(UpdateInputConfirmArgs),
}

#[derive(Args)]
struct RoutinesCommand {
    #[command(subcommand)]
    command: RoutinesSubcommand,
}

#[derive(Subcommand)]
enum RoutinesSubcommand {
    List(PageArgs),
    Get(IdArgs),
    Create(InputConfirmArgs),
    Update(UpdateInputConfirmArgs),
}

#[derive(Args)]
struct FoldersCommand {
    #[command(subcommand)]
    command: FoldersSubcommand,
}

#[derive(Subcommand)]
enum FoldersSubcommand {
    List(PageArgs),
    Get(IdArgs),
    Create(FolderCreateArgs),
}

#[derive(Args)]
struct TemplatesCommand {
    #[command(subcommand)]
    command: TemplatesSubcommand,
}

#[derive(Subcommand)]
enum TemplatesSubcommand {
    List(TemplatePageArgs),
    Get(IdArgs),
    Create(InputConfirmArgs),
}

#[derive(Args)]
struct ExercisesCommand {
    #[command(subcommand)]
    command: ExercisesSubcommand,
}

#[derive(Subcommand)]
enum ExercisesSubcommand {
    History(ExerciseHistoryArgs),
}

#[derive(Args)]
struct WebhooksCommand {
    #[command(subcommand)]
    command: WebhooksSubcommand,
}

#[derive(Subcommand)]
enum WebhooksSubcommand {
    Get,
    Create(WebhookCreateArgs),
    Delete(ConfirmArgs),
}

#[derive(Args)]
struct ExportCommand {
    #[command(subcommand)]
    command: ExportSubcommand,
}

#[derive(Subcommand)]
enum ExportSubcommand {
    Workouts(ExportWorkoutsArgs),
    RoutineBundle(ExportRoutineBundleArgs),
}

#[derive(Args)]
struct AuthCommand {
    #[command(subcommand)]
    command: AuthSubcommand,
}

#[derive(Subcommand)]
enum AuthSubcommand {
    Test,
}

#[derive(Args)]
struct PageArgs {
    #[arg(long, default_value_t = 1)]
    page: u32,
    #[arg(long, default_value_t = 10)]
    page_size: u32,
}

#[derive(Args)]
struct TemplatePageArgs {
    #[arg(long, default_value_t = 1)]
    page: u32,
    #[arg(long, default_value_t = 100)]
    page_size: u32,
}

#[derive(Args)]
struct IdArgs {
    #[arg(long)]
    id: String,
}

#[derive(Args)]
struct EventsArgs {
    #[arg(long)]
    since: String,
    #[arg(long, default_value_t = 1)]
    page: u32,
    #[arg(long, default_value_t = 10)]
    page_size: u32,
}

#[derive(Args)]
struct InputConfirmArgs {
    /// JSON file path, or '-' to read JSON from stdin.
    #[arg(long)]
    input: String,
    #[arg(long)]
    confirm: bool,
}

#[derive(Args)]
struct UpdateInputConfirmArgs {
    #[arg(long)]
    id: String,
    /// JSON file path, or '-' to read JSON from stdin.
    #[arg(long)]
    input: String,
    #[arg(long)]
    confirm: bool,
}

#[derive(Args)]
struct FolderCreateArgs {
    #[arg(long)]
    title: String,
    #[arg(long)]
    confirm: bool,
}

#[derive(Args)]
struct ExerciseHistoryArgs {
    #[arg(long = "template-id")]
    template_id: String,
    #[arg(long)]
    start_date: Option<String>,
    #[arg(long)]
    end_date: Option<String>,
}

#[derive(Args)]
struct WebhookCreateArgs {
    #[arg(long)]
    url: String,
    #[arg(long)]
    confirm: bool,
}

#[derive(Args)]
struct ConfirmArgs {
    #[arg(long)]
    confirm: bool,
}

#[derive(Args)]
struct ExportWorkoutsArgs {
    #[arg(long)]
    weeks: i64,
    #[arg(long)]
    full: bool,
}

#[derive(Args)]
struct ExportRoutineBundleArgs {
    #[arg(long = "routine-id")]
    routine_id: String,
    #[arg(long)]
    weeks: i64,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    if let Err(err) = run().await {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let api_key = cli
        .hevy_api_key
        .context("HEVY_API_KEY must be set or --hevy-api-key must be provided")?;
    let client = HevyClient::with_base_url(api_key, cli.hevy_base_url)?;
    let value = dispatch(&client, cli.command).await?;
    print_json(&value, cli.output)?;
    Ok(())
}

async fn dispatch(client: &HevyClient, command: Commands) -> Result<Value> {
    match command {
        Commands::Workouts(args) => handle_workouts(client, args.command).await,
        Commands::Routines(args) => handle_routines(client, args.command).await,
        Commands::Folders(args) => handle_folders(client, args.command).await,
        Commands::Templates(args) => handle_templates(client, args.command).await,
        Commands::Exercises(args) => handle_exercises(client, args.command).await,
        Commands::Webhooks(args) => handle_webhooks(client, args.command).await,
        Commands::Export(args) => handle_export(client, args.command).await,
        Commands::Auth(args) => handle_auth(client, args.command).await,
    }
}

async fn handle_workouts(client: &HevyClient, command: WorkoutsSubcommand) -> Result<Value> {
    match command {
        WorkoutsSubcommand::List(args) => {
            to_value(client.get_workouts(args.page, args.page_size).await)
        }
        WorkoutsSubcommand::Get(args) => to_value(client.get_workout(&args.id).await),
        WorkoutsSubcommand::Count => client.get_workout_count().await.map_err(Into::into),
        WorkoutsSubcommand::Events(args) => client
            .get_workout_events(args.page, args.page_size, &args.since)
            .await
            .map_err(Into::into),
        WorkoutsSubcommand::Create(args) => {
            require_confirm(args.confirm)?;
            let payload = read_wrapped_input(&args.input, "workout")?;
            to_value(client.create_workout(payload).await)
        }
        WorkoutsSubcommand::Update(args) => {
            require_confirm(args.confirm)?;
            let payload = read_wrapped_input(&args.input, "workout")?;
            to_value(client.update_workout(&args.id, payload).await)
        }
    }
}

async fn handle_routines(client: &HevyClient, command: RoutinesSubcommand) -> Result<Value> {
    match command {
        RoutinesSubcommand::List(args) => {
            to_value(client.get_routines(args.page, args.page_size).await)
        }
        RoutinesSubcommand::Get(args) => to_value(client.get_routine(&args.id).await),
        RoutinesSubcommand::Create(args) => {
            require_confirm(args.confirm)?;
            let payload = read_wrapped_input(&args.input, "routine")?;
            to_value(client.create_routine(payload).await)
        }
        RoutinesSubcommand::Update(args) => {
            require_confirm(args.confirm)?;
            let payload = read_wrapped_input(&args.input, "routine")?;
            to_value(client.update_routine(&args.id, payload).await)
        }
    }
}

async fn handle_folders(client: &HevyClient, command: FoldersSubcommand) -> Result<Value> {
    match command {
        FoldersSubcommand::List(args) => {
            to_value(client.get_folders(args.page, args.page_size).await)
        }
        FoldersSubcommand::Get(args) => to_value(client.get_folder(&args.id).await),
        FoldersSubcommand::Create(args) => {
            require_confirm(args.confirm)?;
            let payload = json!({ "routine_folder": { "title": args.title } });
            to_value(client.create_folder(payload).await)
        }
    }
}

async fn handle_templates(client: &HevyClient, command: TemplatesSubcommand) -> Result<Value> {
    match command {
        TemplatesSubcommand::List(args) => {
            to_value(client.get_templates(args.page, args.page_size).await)
        }
        TemplatesSubcommand::Get(args) => to_value(client.get_template(&args.id).await),
        TemplatesSubcommand::Create(args) => {
            require_confirm(args.confirm)?;
            let payload = read_wrapped_input(&args.input, "exercise_template")?;
            client
                .create_exercise_template(payload)
                .await
                .map_err(Into::into)
        }
    }
}

async fn handle_exercises(client: &HevyClient, command: ExercisesSubcommand) -> Result<Value> {
    match command {
        ExercisesSubcommand::History(args) => client
            .get_exercise_history(
                &args.template_id,
                args.start_date.as_deref(),
                args.end_date.as_deref(),
            )
            .await
            .map_err(Into::into),
    }
}

async fn handle_webhooks(client: &HevyClient, command: WebhooksSubcommand) -> Result<Value> {
    match command {
        WebhooksSubcommand::Get => client.get_webhook_subscription().await.map_err(Into::into),
        WebhooksSubcommand::Create(args) => {
            require_confirm(args.confirm)?;
            let payload = json!({ "webhook": { "url": args.url } });
            client
                .create_webhook_subscription(payload)
                .await
                .map_err(Into::into)
        }
        WebhooksSubcommand::Delete(args) => {
            require_confirm(args.confirm)?;
            client.delete_webhook_subscription().await?;
            Ok(json!({ "status": "success" }))
        }
    }
}

async fn handle_export(client: &HevyClient, command: ExportSubcommand) -> Result<Value> {
    match command {
        ExportSubcommand::Workouts(args) => {
            let workouts = export_workouts(client, args.weeks, args.full).await?;
            Ok(json!({
                "generated_at": Utc::now().to_rfc3339(),
                "weeks": args.weeks,
                "full": args.full,
                "workoutLogs": workouts,
            }))
        }
        ExportSubcommand::RoutineBundle(args) => {
            let routine = client.get_routine(&args.routine_id).await?;
            let workouts = export_workouts(client, args.weeks, true).await?;
            Ok(json!({
                "generated_at": Utc::now().to_rfc3339(),
                "weeks": args.weeks,
                "routineBundle": routine,
                "workoutLogs": workouts,
            }))
        }
    }
}

async fn handle_auth(client: &HevyClient, command: AuthSubcommand) -> Result<Value> {
    match command {
        AuthSubcommand::Test => {
            let count = client.get_workout_count().await?;
            Ok(json!({ "status": "ok", "workout_count": count }))
        }
    }
}

async fn export_workouts(client: &HevyClient, weeks: i64, full: bool) -> Result<Vec<Value>> {
    if weeks <= 0 {
        bail!("--weeks must be greater than zero");
    }

    let cutoff = Utc::now() - Duration::weeks(weeks);
    let mut page = 1;
    let page_size = 10;
    let mut workouts = Vec::new();

    loop {
        let response = client.get_workouts(page, page_size).await?;
        let mut saw_recent = false;

        for workout in response.workouts {
            if workout_started_after_cutoff(&workout.start_time, cutoff)? {
                saw_recent = true;
                if full {
                    workouts.push(serde_json::to_value(
                        client.get_workout(&workout.id).await?,
                    )?);
                } else {
                    workouts.push(serde_json::to_value(workout)?);
                }
            }
        }

        if page >= response.page_count as u32 || !saw_recent {
            break;
        }
        page += 1;
    }

    Ok(workouts)
}

fn workout_started_after_cutoff(start_time: &str, cutoff: DateTime<Utc>) -> Result<bool> {
    let parsed = DateTime::parse_from_rfc3339(start_time)
        .with_context(|| format!("failed to parse workout start_time as RFC3339: {start_time}"))?;
    Ok(parsed.with_timezone(&Utc) >= cutoff)
}

fn require_confirm(confirm: bool) -> Result<()> {
    if !confirm {
        bail!("write command refused: pass --confirm to execute this Hevy mutation");
    }
    Ok(())
}

fn read_wrapped_input(input: &str, key: &str) -> Result<Value> {
    let value = read_json_input(input)?;
    if value.get(key).is_some() {
        Ok(value)
    } else {
        Ok(json!({ key: value }))
    }
}

fn read_json_input(input: &str) -> Result<Value> {
    let contents = if input == "-" {
        let mut contents = String::new();
        std::io::stdin()
            .read_to_string(&mut contents)
            .context("failed to read JSON from stdin")?;
        contents
    } else {
        let path = PathBuf::from(input);
        std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read JSON input file {}", path.display()))?
    };

    serde_json::from_str(&contents).context("failed to parse JSON input")
}

fn to_value<T: Serialize, E: Into<anyhow::Error>>(
    result: std::result::Result<T, E>,
) -> Result<Value> {
    Ok(serde_json::to_value(result.map_err(Into::into)?)?)
}

fn print_json(value: &Value, output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Json => println!("{}", serde_json::to_string(value)?),
        OutputFormat::Pretty => println!("{}", serde_json::to_string_pretty(value)?),
    }
    Ok(())
}
