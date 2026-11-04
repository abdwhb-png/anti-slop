use anti_slop::{config, output};
use clap::{
    Parser, Subcommand, ValueEnum,
    error::{ContextKind, ContextValue, ErrorKind},
};
use std::{ffi::OsString, io::Write, path::PathBuf};

const USAGE: &str = "anti-slop config explain --workspace <dir> --path <relative-path> [--config <file>] [--format toon|json]";

fn invalid_value_message(error: &clap::Error) -> &'static str {
    // Inspect only the known argument identity; never print the rejected value.
    match error.get(ContextKind::InvalidArg) {
        Some(ContextValue::String(arg)) if arg.starts_with("--workspace ") => {
            "--workspace requires a directory value"
        }
        Some(ContextValue::String(arg)) if arg.starts_with("--path ") => {
            "--path requires a relative path value"
        }
        Some(ContextValue::String(arg)) if arg.starts_with("--config ") => {
            "--config requires a file path value"
        }
        Some(ContextValue::String(arg)) if arg.starts_with("--format ") => {
            "use --format toon or --format json"
        }
        _ => "a flag value is missing or invalid; supply the values shown in the usage",
    }
}

#[derive(Parser)]
#[command(
    version,
    about = "Inspect local configuration. Analysis is not implemented yet.",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Inspect effective settings without reading target source or writing files.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}
#[derive(Subcommand)]
enum ConfigCommand {
    /// Explain policies and winning origins for one workspace-relative path.
    #[command(
        after_help = "Examples:\n  anti-slop config explain --workspace . --path src/lib.rs\n  anti-slop config explain --workspace . --path future.ts --config policy.jsonc --format json"
    )]
    Explain {
        /// Existing workspace directory (required).
        #[arg(long)]
        workspace: PathBuf,
        /// Nonempty workspace-relative path; the target need not exist.
        #[arg(long)]
        path: PathBuf,
        /// File relative to workspace or absolute; missing is an error.
        #[arg(long)]
        config: Option<PathBuf>,
        /// Explicit encoding, including when stdout is redirected.
        #[arg(long, value_enum, default_value = "toon")]
        format: FormatArg,
    },
}
#[derive(Clone, Copy, ValueEnum)]
enum FormatArg {
    Toon,
    Json,
}
impl From<FormatArg> for output::Format {
    fn from(value: FormatArg) -> Self {
        match value {
            FormatArg::Toon => Self::Toon,
            FormatArg::Json => Self::Json,
        }
    }
}

// Argument failures retain a valid selected format without echoing rejected
// values. Unknown, missing or ambiguous formats use the TOON default.
fn error_format(args: &[OsString]) -> output::Format {
    let mut selection = None;
    let mut index = 1;
    while index < args.len() {
        let arg = args[index].to_str().unwrap_or("");
        let value = if arg == "--format" {
            index += 1;
            Some(args.get(index).and_then(|v| v.to_str()).unwrap_or(""))
        } else {
            arg.strip_prefix("--format=")
        };
        if let Some(value) = value {
            if selection.is_some() {
                return output::Format::Toon;
            }
            selection = Some(match value {
                "toon" => output::Format::Toon,
                "json" => output::Format::Json,
                _ => return output::Format::Toon,
            });
        }
        index += 1;
    }
    selection.unwrap_or_default()
}
fn emit(view: &output::Inspection<'_>, format: output::Format, code: i32) -> i32 {
    let text = match output::encode(view, format) {
        Ok(text) => text,
        Err(_) => {
            eprintln!("Could not encode configuration inspection.");
            return 3;
        }
    };
    match std::io::stdout().lock().write_all(text.as_bytes()) {
        Ok(()) => code,
        Err(_) => {
            eprintln!("Could not write configuration inspection.");
            3
        }
    }
}
pub(crate) fn run(args: Vec<OsString>) -> i32 {
    let cli = match Cli::try_parse_from(&args) {
        Ok(cli) => cli,
        Err(error) => {
            if error.exit_code() == 0 || args.get(1).is_none_or(|arg| arg != "config") {
                let code = error.exit_code();
                let _ = error.print();
                return code;
            }
            let message = match error.kind() {
                ErrorKind::MissingRequiredArgument => "--workspace and --path are required",
                ErrorKind::InvalidValue => invalid_value_message(&error),
                _ => {
                    "invalid or duplicate arguments; valid flags: --workspace, --path, --config, --format, --help"
                }
            };
            let error = config::ConfigError {
                code: config::ErrorCode::InvalidInput,
                message: message.into(),
                location: None,
            };
            return emit(
                &output::Inspection::Error {
                    error: &error,
                    help: USAGE,
                },
                error_format(&args),
                2,
            );
        }
    };
    let Command::Config {
        command:
            ConfigCommand::Explain {
                workspace,
                path,
                config,
                format,
            },
    } = cli.command;
    let format = format.into();
    match config::resolve(&workspace, &path, config.as_deref()) {
        Ok(snapshot) => emit(
            &output::Inspection::Config {
                config: snapshot.view(),
            },
            format,
            0,
        ),
        Err(error) => emit(
            &output::Inspection::Error {
                error: &error,
                help: "Correct the indicated path or configuration entry, then repeat config explain. Use config explain --help for the supported local contract.",
            },
            format,
            error.exit_code(),
        ),
    }
}
