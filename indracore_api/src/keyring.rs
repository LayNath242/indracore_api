use crate::primitives;
use substrate_subxt::{
    sp_core::{sr25519::Pair, Pair as TraitPair},
    Error, IndracoreNodeRuntime, PairSigner,
};

#[derive(Debug)]
pub struct Signer {
    pub mnemonic: String,
}

impl Signer {
    pub fn pair(&self, pass: Option<&str>) -> Result<primitives::Signer, Error> {
        let pair = Pair::from_string(&self.mnemonic, pass);
        match pair {
            Ok(p) => Ok(PairSigner::<IndracoreNodeRuntime, Pair>::new(p)),
            Err(_) => Err(Error::Other("Invalid account".into())),
        }
    }
}
