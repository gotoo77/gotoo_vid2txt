//! Local acquisition and URL adapters. Their output always remains a local file.
mod local;
mod ytdlp;

pub use local::LocalAcquisition;
pub use ytdlp::YtDlp;

use crate::{domain::Source, Result};
use std::path::{Path, PathBuf};

pub trait Acquisition: Send + Sync {
    fn acquire(&self, source: &Source, scratch: &Path) -> Result<PathBuf>;
}

/// Route sources to two independent acquisition adapters.
pub struct RoutedAcquisition {
    pub local: LocalAcquisition,
    pub remote: YtDlp,
}

impl Acquisition for RoutedAcquisition {
    fn acquire(&self, source: &Source, scratch: &Path) -> Result<PathBuf> {
        match source {
            Source::Local(_) => self.local.acquire(source, scratch),
            Source::Url(_) => self.remote.acquire(source, scratch),
        }
    }
}
