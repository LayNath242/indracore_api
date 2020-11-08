use crate::primitives;
use substrate_subxt::{
    sp_core::{sr25519::Pair, Pair as TraitPair},
    system::System,
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

pub fn parse_code_hash(
    input: &str,
) -> Result<<IndracoreNodeRuntime as System>::Hash, hex::FromHexError> {
    let bytes = if input.starts_with("0x") {
        hex::decode(input.trim_start_matches("0x"))?
    } else {
        hex::decode(input)?
    };
    if bytes.len() != 32 {
        return Err(hex::FromHexError::InvalidStringLength);
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr.into())
}

#[test]
fn parse_code_hash_works() {
    // with 0x prefix
    assert!(
        parse_code_hash("0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
            .is_ok()
    );
    // without 0x prefix
    assert!(
        parse_code_hash("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d").is_ok()
    )
}
