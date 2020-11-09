use crate::contract::transcode::Transcoder;
use crate::primitives;
use substrate_subxt::{contracts::*, system::System, ClientBuilder, Error, IndracoreNodeRuntime};

pub struct Instantiate {
    pub name: String,
    pub args: Vec<String>,
    pub metadata: String,
    pub signer: primitives::Sr25519,
    pub endowment: u128,
    pub gas_limit: u64,
    pub code_hash: <IndracoreNodeRuntime as System>::Hash,
}

impl Instantiate {
    pub fn instantiate(&self) -> Result<InstantiatedEvent<IndracoreNodeRuntime>, Error> {
        let metadata = match super::load_metadata(&self.metadata) {
            Ok(m) => m,
            Err(_) => return Err(Error::Other("loading metadata failed".into())),
        };

        let transcoder = Transcoder::new(metadata);
        let data = match transcoder.encode(&self.name, &self.args) {
            Ok(m) => m,
            Err(_) => return Err(Error::Other("encode metadata error".into())),
        };
        async_std::task::block_on(async move {
            let client = match ClientBuilder::<IndracoreNodeRuntime>::new()
                .set_url(primitives::url())
                .build()
                .await
            {
                Ok(cli) => cli,
                Err(e) => return Err(e),
            };
            let result = client
                .instantiate_and_watch(
                    &self.signer,
                    self.endowment,
                    self.gas_limit,
                    &self.code_hash,
                    &data,
                )
                .await?;

            let instantiated = result
                .instantiated()?
                .ok_or_else(|| Error::Other("Failed to find a Instantiated event".into()))?;

            Ok(instantiated)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::contract::instantiate::Instantiate;
    use crate::keyring::parse_code_hash;
    use sp_keyring::AccountKeyring;
    use substrate_subxt::{sp_core::sr25519::Pair, IndracoreNodeRuntime, PairSigner};

    #[test]
    fn test_instantiated() {
        let pair = AccountKeyring::Alice.pair();
        let signer = PairSigner::<IndracoreNodeRuntime, Pair>::new(pair);
        let code_hash =
            parse_code_hash("0x40f8c7c624d1d8fbd0873a381c63a0858b4d75315bd8ca62e0111068bbf138e3");
        let metadata =
            "/data/project/indracore-api/indracore_api/src/contract/test/erc20.json".to_string();
        let mut args: Vec<String> = Vec::new();
        args.push("1_000_000_000".to_string());

        let inst = Instantiate {
            name: "new".to_string(),
            args: args.clone(),
            metadata,
            signer,
            code_hash: code_hash.unwrap(),
            endowment: 1_000_000_000_000_000,
            gas_limit: 500_000_000_000,
        };

        assert!(inst.instantiate().is_ok())
    }
}