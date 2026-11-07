//! The `git-prolly` command-line interface: the Dolt-shaped `db` surface over
//! this crate's versioned database.

use anyhow::Result;
use clap::{Parser, Subcommand};

use git_database::cli::db::{self, DbCommand};
use git_database::cli::{ClassifiedContext, ExitClass, OutputFormat};

#[derive(Parser)]
#[command(
    name = "git-prolly",
    about = "A versioned key/value database on ordinary Git objects",
    version,
    arg_required_else_help = true
)]
struct Cli {
    /// Output format, honored by every command.
    #[arg(long, global = true, value_enum)]
    format: Option<OutputFormat>,
    /// Shorthand for `--format json`.
    #[arg(long, global = true, hide = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Versioned key/value database (Dolt-shaped porcelain).
    #[command(name = "db")]
    Db {
        #[command(subcommand)]
        command: DbCommand,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        if RUN_FORMAT.get().is_some_and(|format| format.machine()) {
            // Machine consumers parse stdout; give failures the same
            // envelope success paths use, classified like the exit code.
            println!(
                "{{\"status\":\"error\",\"code\":\"{}\"}}",
                class_slug(exit_class_of(&error))
            );
        }
        std::process::exit(error_exit_code(&error));
    }
}

/// The output format of the in-flight invocation, for the error reporter.
static RUN_FORMAT: std::sync::OnceLock<OutputFormat> = std::sync::OnceLock::new();

fn run() -> Result<()> {
    let cli = Cli::parse();
    let output = OutputFormat::from_cli(cli.format, cli.json);
    let _ = RUN_FORMAT.set(output);
    let repo = gix::discover(".").map_err(|_| anyhow::anyhow!("not inside a git repository"))?;
    match cli.command {
        Command::Db { command } => db::run(&repo, command, output),
    }
}

/// The stable machine-facing slug for an exit class.
fn class_slug(class: ExitClass) -> &'static str {
    match class {
        ExitClass::Cas => "cas",
        ExitClass::NotFound => "not_found",
        ExitClass::Schema => "schema",
        ExitClass::Invalid => "invalid",
        ExitClass::Other => "error",
    }
}

/// The stable machine-facing class of a failure, read from the
/// [`ClassifiedContext`] the database CLI attaches to its errors.
fn exit_class_of(error: &anyhow::Error) -> ExitClass {
    let mut class = ExitClass::Other;
    for cause in error.chain() {
        if let Some(context) = cause.downcast_ref::<ClassifiedContext>() {
            class = class.prefer(context.class);
        }
    }
    class
}

fn error_exit_code(error: &anyhow::Error) -> i32 {
    exit_class_of(error).code()
}
