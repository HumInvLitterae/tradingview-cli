//! Internal authenticated TradingView MCP client. No stable external Rust API.

mod admission;
mod auth;
mod browser;
mod budget;
mod client;
mod credentials;
mod error;
mod http;
mod proof;
mod sse;
mod tools;
mod transport;
mod watchlist;
#[cfg(windows)]
mod windows_state;

pub use client::{Client, Operation};
pub use credentials::credential_worker;
pub use error::Failure;
pub use proof::{ProofOperation, run_proof, run_proof_with_worker};

pub type Result<T> = std::result::Result<T, Failure>;

/// Exercise local admission state only; no credentials or provider I/O.
/// This is development-harness plumbing, not a stable public CLI contract.
#[doc(hidden)]
pub async fn check_local_admission(directory: &std::path::Path) -> Result<()> {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
    let mut admission = admission::Admission::acquire(directory, deadline).await?;
    admission.before_tool(deadline).await?;
    admission.cooldown(1)?;
    drop(admission);
    let mut admission = admission::Admission::acquire(directory, deadline).await?;
    match admission.before_tool(deadline).await {
        Err(Failure::RateLimited) => Ok(()),
        _ => Err(Failure::LocalState),
    }
}

/// Exercise the bounded SSE decoder and its SDK event compatibility locally.
#[doc(hidden)]
pub async fn check_local_sse() -> Result<()> {
    sse::check_decoder().await
}

#[cfg(test)]
mod fixtures;
