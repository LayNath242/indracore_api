mod transcode;

use anyhow::Result;
use std::{fs::File, path::PathBuf};

pub fn load_metadata() -> Result<ink_metadata::InkProject> {
    let mut path: PathBuf = PathBuf::new();
    path.push("/data/project/cr/template/erc20/target/metadata.json");
    let metadata = serde_json::from_reader(File::open(path)?)?;
    Ok(metadata)
}
