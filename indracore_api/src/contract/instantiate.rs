use crate::primitives;
use substrate_subxt::{system::System, IndracoreNodeRuntime};

pub struct Instantiate {
    pub name: String,
    pub args: Vec<String>,
    pub signer: primitives::Sr25519,
    pub endowment: u128,
    pub gas_limit: u64,
    pub code_hash: <IndracoreNodeRuntime as System>::Hash,
}
