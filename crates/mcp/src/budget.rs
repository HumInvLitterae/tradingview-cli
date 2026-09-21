//! Persist counters before dispatch; the surrounding Admission lock owns access.

use crate::{Failure, Result, admission::now_ms};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub(crate) enum RequestClass {
    Metadata,
    Registration,
    Exchange,
    Refresh,
    Protocol,
    Tool,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Counts {
    pub started_ms: u64,
    pub metadata: u32,
    pub registration: u32,
    pub exchange: u32,
    pub refresh: u32,
    pub protocol: u32,
    pub tools: u32,
    #[serde(default)]
    pub saved_generation: u32,
    // Accept old private proof records; their assistant-imposed limits are retired.
    #[serde(default, rename = "read_only_window", skip_serializing)]
    _legacy_read_only_window: bool,
}

pub(crate) struct Budget {
    path: PathBuf,
    counts: Counts,
}

impl Budget {
    pub fn open(directory: &Path, start: bool) -> Result<Self> {
        let path = directory.join("proof-budget.json");
        // A login may resume after public discovery failed. Existing counters
        // and the original window are always retained, including exhausted ones.
        let create = start && !path.exists();
        let counts = if create {
            Counts {
                started_ms: now_ms()?,
                ..Default::default()
            }
        } else {
            crate::admission::read_private_json(&path, 4096)?
        };
        let value = Self { path, counts };
        if create {
            value.save()?;
        }
        Ok(value)
    }

    fn save(&self) -> Result<()> {
        crate::admission::write_private_json(&self.path, &self.counts)
    }

    pub fn charge(&mut self, class: RequestClass) -> Result<()> {
        let now = now_ms()?;
        if now < self.counts.started_ms {
            return Err(Failure::Clock);
        }
        let mut next = self.counts.clone();
        let counter = match class {
            RequestClass::Metadata => &mut next.metadata,
            RequestClass::Registration => &mut next.registration,
            RequestClass::Exchange => &mut next.exchange,
            RequestClass::Refresh => &mut next.refresh,
            RequestClass::Protocol => &mut next.protocol,
            RequestClass::Tool => {
                next.tools = next.tools.checked_add(1).ok_or(Failure::LocalState)?;
                &mut next.protocol
            }
        };
        *counter = counter.checked_add(1).ok_or(Failure::LocalState)?;
        self.counts = next;
        self.save()
    }

    pub fn credential_saved(&mut self) -> Result<()> {
        self.counts.saved_generation = self.counts.exchange + self.counts.refresh;
        self.save()
    }

    pub fn credential_continuity(&self) -> Result<()> {
        if self.counts.exchange + self.counts.refresh != self.counts.saved_generation {
            return Err(Failure::AuthRequired);
        }
        Ok(())
    }

    pub fn counts(&self) -> Counts {
        self.counts.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::time::{Duration, Instant};
    #[tokio::test]
    async fn request_counts_and_original_start_survive_resume() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("state");
        let _guard =
            crate::admission::Admission::acquire(&dir, Instant::now() + Duration::from_secs(10))
                .await
                .unwrap();
        let mut first = Budget::open(&dir, true).unwrap();
        let start = first.counts().started_ms;
        for _ in 0..13 {
            first.charge(RequestClass::Protocol).unwrap();
        }
        first.charge(RequestClass::Refresh).unwrap();
        assert_eq!(
            first.credential_continuity().unwrap_err(),
            Failure::AuthRequired
        );
        first.credential_saved().unwrap();
        drop(first);
        let mut resumed = Budget::open(&dir, true).unwrap();
        assert_eq!(resumed.counts().started_ms, start);
        assert_eq!(resumed.counts().protocol, 13);
        resumed.credential_continuity().unwrap();
        resumed.charge(RequestClass::Tool).unwrap();
        assert_eq!(resumed.counts().protocol, 14);
        assert_eq!(resumed.counts().tools, 1);
    }
}
