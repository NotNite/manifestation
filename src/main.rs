use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(version)]
pub struct Args {
    /// Path to manifestation.toml
    toml_path: Option<PathBuf>,

    #[clap(long, short, action)]
    /// Enable copying the mod to another dir. Defaults to your GDWeave/mods/ directory.
    copy: bool,

    #[clap(long)]
    /// Optional destination for manifestation to copy to
    copy_path: Option<PathBuf>,
}

pub mod config;
pub mod execute;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(toml_path) = args.toml_path {
        execute::process(toml_path.as_ref(), args.copy, args.copy_path)?;
    } else {
        config::config()?;
    }

    Ok(())
}
