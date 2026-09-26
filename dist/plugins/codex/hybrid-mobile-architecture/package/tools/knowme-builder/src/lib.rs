mod bundle;
mod cli;
mod engine;
mod migrations;
mod model;
mod native;
mod upgrade_journal;

pub use cli::Cli;

use anyhow::Result;

pub fn execute(cli: Cli) -> Result<()> {
    engine::execute(cli)
}
