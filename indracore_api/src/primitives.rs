use std::env;
use substrate_subxt::{
    sp_core::{ed25519, sr25519},
    IndracoreNodeRuntime, PairSigner,
};

pub type Sr25519 = PairSigner<IndracoreNodeRuntime, sr25519::Pair>;
pub type Ed25519 = PairSigner<IndracoreNodeRuntime, ed25519::Pair>;
pub type Client = substrate_subxt::Client<IndracoreNodeRuntime>;
pub type IndracoreId = pallet_indices::address::Address<sp_core::crypto::AccountId32, u32>;

pub fn url() -> String {
    dotenv::dotenv().expect("!!! Failed to read .env file");
    let url = env::var("RPC");
    url.unwrap_or("ws://127.0.0.1:9944".to_string())
}
