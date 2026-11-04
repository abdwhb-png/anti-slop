//! Generate to stdout, or compare an existing local artifact without rewriting it.
use std::{io::Write, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let mut generated = serde_json::to_string_pretty(&anti_slop::config::schema_document())
        .expect("generated schema is JSON");
    generated.push('\n');
    if args.is_empty() {
        return if std::io::stdout()
            .lock()
            .write_all(generated.as_bytes())
            .is_ok()
        {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(3)
        };
    }
    if args[0] != "--check" || args.len() > 2 {
        eprintln!("Usage: config_schema [--check [schema-file]]");
        return ExitCode::from(2);
    }
    let path = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("schemas/anti-slop.schema.json"));
    match std::fs::read(path) {
        Ok(bytes) if bytes == generated.as_bytes() => ExitCode::SUCCESS,
        Ok(_) => {
            eprintln!(
                "Schema differs from the authoritative Rust configuration definition; regenerate it."
            );
            ExitCode::from(1)
        }
        Err(_) => {
            eprintln!("Could not read the schema artifact.");
            ExitCode::from(3)
        }
    }
}
