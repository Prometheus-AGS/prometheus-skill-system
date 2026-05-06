use crate::AppContext;
use anyhow::Result;

pub mod autonomous;
pub mod ci;
pub mod deps;
pub mod enforce;
pub mod format;
pub mod inventory;
pub mod partition;

pub fn run_audit(ctx: &AppContext) -> Result<i32> {
    let mut exit = 0;
    exit = exit.max(run_enforce(ctx)?);
    exit = exit.max(run_format(ctx)?);
    exit = exit.max(run_deps(ctx)?);
    exit = exit.max(run_inventory(ctx)?);
    exit = exit.max(run_partition(ctx)?);
    exit = exit.max(run_ci(ctx)?);
    Ok(exit)
}

pub fn run_enforce(ctx: &AppContext) -> Result<i32> {
    enforce::run(ctx)
}

pub fn run_format(ctx: &AppContext) -> Result<i32> {
    format::run(ctx)
}

pub fn run_deps(ctx: &AppContext) -> Result<i32> {
    deps::run(ctx)
}

pub fn run_inventory(ctx: &AppContext) -> Result<i32> {
    inventory::run(ctx)
}

pub fn run_partition(ctx: &AppContext) -> Result<i32> {
    partition::run(ctx)
}

pub fn run_ci(ctx: &AppContext) -> Result<i32> {
    ci::run(ctx)
}

pub fn run_autonomous(ctx: &AppContext) -> Result<i32> {
    autonomous::run(ctx)
}
