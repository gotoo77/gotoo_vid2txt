use super::Acquisition;
use crate::{domain::Source, Result};
use std::path::{Path, PathBuf};

pub struct LocalAcquisition;

impl Acquisition for LocalAcquisition {
    fn acquire(&self, source: &Source, _scratch: &Path) -> Result<PathBuf> {
        let Source::Local(path) = source else {
            return Err("local adapter received an URL".into());
        };
        if !path.is_file() {
            return Err(format!("local media not found: {}", path.display()).into());
        }
        Ok(path.canonicalize()?)
    }
}
