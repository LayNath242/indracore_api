use crate::primitives;
use substrate_subxt::{
    sp_core::{ed25519, sr25519, Pair as TraitPair},
    sp_runtime,
    system::System,
    Error, IndracoreNodeRuntime, PairSigner,
};

use std::str::FromStr;

pub fn indracoreid(pubkey: &str) -> Result<primitives::IndracoreId, Error> {
    let id = sp_runtime::AccountId32::from_str(pubkey);
    match id {
        Ok(id) => Ok(pallet_indices::address::Address::from(id)),
        Err(e) => return Err(Error::Other(e.into())),
    }
}

pub fn accounid32(pubkey: &str) -> Result<sp_core::crypto::AccountId32, Error> {
    let id = sp_runtime::AccountId32::from_str(pubkey);
    match id {
        Ok(id) => Ok(id),
        Err(e) => return Err(Error::Other(e.into())),
    }
}

#[derive(PartialEq)]
pub struct Sr25519 {
    pub suri: String,
}

impl Sr25519 {
    pub fn pair(&self, pass: Option<&str>) -> Result<primitives::Sr25519, Error> {
        let pair = sr25519::Pair::from_string(&self.suri, pass);
        match pair {
            Ok(p) => Ok(PairSigner::<IndracoreNodeRuntime, sr25519::Pair>::new(p)),
            Err(e) => return Err(Error::Other(format!("{:?}", e))),
        }
    }

    pub fn to_accountid(&self) -> Result<sp_core::crypto::AccountId32, Error> {
        let pair = sr25519::Pair::from_string(&self.suri, None);
        match pair {
            Ok(data) => Ok(sp_core::crypto::AccountId32::from(data.public())),
            Err(e) => return Err(Error::Other(format!("{:?}", e))),
        }
    }
}

#[derive(PartialEq)]
pub struct Ed25519 {
    pub suri: String,
}

impl Ed25519 {
    pub fn pair(&self, pass: Option<&str>) -> Result<primitives::Ed25519, Error> {
        let pair = ed25519::Pair::from_string(&self.suri, pass);
        match pair {
            Ok(p) => Ok(PairSigner::<IndracoreNodeRuntime, ed25519::Pair>::new(p)),
            Err(e) => Err(Error::SecretString(e)),
        }
    }

    pub fn to_accountid(&self) -> Result<sp_core::crypto::AccountId32, Error> {
        let pair = ed25519::Pair::from_string(&self.suri, None);
        match pair {
            Ok(data) => Ok(sp_core::crypto::AccountId32::from(data.public())),
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

#[cfg(test)]
mod test {
    use crate::keyring::{accounid32, indracoreid, parse_code_hash, Ed25519, Sr25519};
    #[test]
    fn test_sr25519() {
        let sig = Sr25519 {
            suri: "0x0d782a1f150ff7eadd1a4fa0ec3e0a46d77ba89c86ac5d4ce6ddfdc9d54e5beb".into(),
        };
        assert!(sig.pair(None).is_ok());
        assert!(sig.to_accountid().is_ok())
    }
    #[test]
    fn test_ed25519() {
        let sig = Ed25519 {
            suri: "0x0d782a1f150ff7eadd1a4fa0ec3e0a46d77ba89c86ac5d4ce6ddfdc9d54e5beb".into(),
        };

        assert!(sig.pair(None).is_ok());
        assert!(sig.to_accountid().is_ok())
    }

    #[test]
    fn parse_code_hash_works() {
        // with 0x prefix
        assert!(parse_code_hash(
            "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
        )
        .is_ok());
        // without 0x prefix
        assert!(
            parse_code_hash("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
                .is_ok()
        )
    }

    #[test]
    fn test_id() {
        let pubkey = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

        assert!(indracoreid(pubkey).is_ok());
        assert!(accounid32(pubkey).is_ok())
    }
}
