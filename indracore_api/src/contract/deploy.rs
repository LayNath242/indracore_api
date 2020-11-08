
use std::path::PathBuf;
use primitives;

pub struct ConractDeploy {
    pub wasm_path: PathBuf,
    pub signer: primitives::Signer,
}
