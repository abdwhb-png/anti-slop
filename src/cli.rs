use clap::Parser;

#[derive(Parser)]
#[command(
    version,
    about = "Anti-Slop scaffold. Analysis is not implemented yet.",
    arg_required_else_help = true
)]
pub(crate) struct Cli {}
