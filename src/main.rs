pub mod config;
pub mod execute;

fn main() -> anyhow::Result<()> {
    if let Some(config_path) = std::env::args().nth(1) {
        execute::process(&std::path::Path::new(&config_path))?;
    } else {
        config::config()?;
    }

    Ok(())
}
