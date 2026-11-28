//! Shared authenticated local client. The existing daemon remains independent
//! of KBD. Clients never implement an executor or synthesize terminal proof.
pub mod identity;
pub mod uds_client;
mod prepared_request;
pub use prepared_request::{prepare, submit, PrepareRequest, SubmitRequest};
#[cfg(unix)]
pub fn effective_uid()->u32{nix::unistd::Uid::effective().as_raw()}
#[cfg(unix)]
pub fn managed_home()->Result<std::path::PathBuf,BoxError>{prepared_request::installed_home()}
use std::path::Path;
use prometheus_exec_contracts::Digest;
use prometheus_exec_core::ArtifactStore;
use crate::RunResponse;
use hyper::Method;
use uuid::Uuid;
type BoxError = Box<dyn std::error::Error + Send + Sync>;
fn mebibytes(value:u64)->Result<u64,BoxError>{
    value.checked_mul(1024*1024).ok_or_else(|| "artifact budget overflow".into())
}
fn release_pins(artifacts:&ArtifactStore,hashes:&[Digest],reason:&str)->Result<(),String>{
    let failures=hashes.iter().filter_map(|hash|artifacts.unpin(hash,reason).err().map(|e|format!("{hash}: {e}"))).collect::<Vec<_>>();
    if failures.is_empty(){Ok(())}else{Err(failures.join("; "))}
}
async fn fetch_run(socket:&Path,run_id:Uuid)->Result<RunResponse,BoxError>{
    let response=uds_client::request_same_uid(socket,Method::GET,&format!("/api/v2/exec/runs/{run_id}"),vec![]).await?;
    if response.status!=200{return Err(format!("run status refused HTTP {}",response.status).into());}
    Ok(serde_json::from_slice(&response.body)?)
}
