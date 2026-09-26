use anyhow::Result;
use clap::Parser;
use knowme_builder::{Cli, execute};

fn main() -> Result<()> {
    execute(Cli::parse())
}
