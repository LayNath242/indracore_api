pub mod call;
pub mod deploy;
pub mod instantiate;
mod transcode;

use anyhow::Result;
use std::{fs::File, path::PathBuf};

pub fn load_metadata(dir: &str) -> Result<ink_metadata::InkProject> {
    let mut path: PathBuf = PathBuf::new();
    path.push(dir);
    let metadata = serde_json::from_reader(File::open(path)?)?;
    Ok(metadata)
}
