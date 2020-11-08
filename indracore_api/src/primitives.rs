use substrate_subxt::{
    sp_core::{sr25519::Pair},
    IndracoreNodeRuntime, PairSigner,
};
pub type Signer = PairSigner<IndracoreNodeRuntime, Pair>;
pub type Client = substrate_subxt::Client<IndracoreNodeRuntime>;
