use crate::job::checkpoint;
use anyhow::Context as _;

pub fn cancel_job(job_id: &str) -> anyhow::Result<()> {
    let cp = checkpoint::read(job_id).with_context(|| format!("job {job_id} not found"))?;

    #[cfg(unix)]
    if let Some(pid) = cp.pid {
        use nix::sys::signal::{self, Signal};
        use nix::unistd::Pid;
        let _ = signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
    }

    checkpoint::update_status(job_id, "cancelled")?;
    Ok(())
}
