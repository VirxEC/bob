use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod build;
mod buildinfo;
mod config;
mod pack;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Doc comment
#[derive(Subcommand)]
#[command()]
enum Command {
    /// Build based on a bob.toml
    Build(BuildCommand),

    /// Package a bob output dir
    Pack(PathCommand),
}

#[derive(Parser, Debug)]
struct BuildCommand {
    config_path: PathBuf,
    #[arg(short, long, default_value = "./bob_build")]
    /// By default, bob will reuse already-built projects if the project hash matches
    out_dir: PathBuf,
}

#[derive(Parser, Debug)]
struct PathCommand {
    #[arg(default_value = "./bob_build")]
    build_dir: PathBuf,
    #[arg(short, long)]
    old_buildinfo: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();
    match cli.command {
        Command::Build(x) => build::build(x),
        Command::Pack(x) => pack::pack(x),
    }
}
