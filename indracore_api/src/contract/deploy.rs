use crate::primitives;
use std::{fs, io::Read, path::PathBuf};

pub struct ContractDeploy {
    pub wasm_path: PathBuf,
    pub signer: primitives::Sr25519,
}
use substrate_subxt::{contracts::*, ClientBuilder, Error, IndracoreNodeRuntime};

impl ContractDeploy {
    fn load_contract(&self) -> Result<Vec<u8>, Error> {
        let contract_wasm_path = self.wasm_path.clone();
        let mut data: Vec<u8> = Vec::new();

        let file = fs::File::open(&contract_wasm_path);
        let mut file = match file {
            Ok(f) => f,
            Err(e) => return Err(Error::Other(format!("{:?}", e))),
        };
        match file.read_to_end(&mut data) {
            Ok(_) => Ok(data),
            Err(e) => return Err(Error::Other(format!("{:?}", e))),
        }
    }

    ///put contract code to indracoe chain
    pub fn exec(&self) -> Result<sp_core::H256, Error> {
        let code = match self.load_contract() {
            Ok(code) => code,
            Err(e) => return Err(e),
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
            let result = client.put_code_and_watch(&self.signer, &code).await?;
            let code_stored = result
                .code_stored()?
                .ok_or_else(|| Error::Other("Failed to find a CodeStored event".into()))?;
            Ok(code_stored.code_hash)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::contract::deploy::ContractDeploy;
    use crate::keyring::parse_code_hash;
    use sp_keyring::AccountKeyring;
    use std::path::PathBuf;
    use substrate_subxt::{sp_core::sr25519::Pair, IndracoreNodeRuntime, PairSigner};

    #[test]
    fn test_deploy() {
        let mut wasm_path = PathBuf::new();
        wasm_path.push("/data/project/indracore-api/indracore_api/src/contract/test/erc20.wasm");
        let pair = AccountKeyring::Alice.pair();

        let signer = PairSigner::<IndracoreNodeRuntime, Pair>::new(pair);

        let deploy = ContractDeploy { wasm_path, signer };
        let result = deploy.exec().unwrap();

        let code_hash =
            parse_code_hash("0x40f8c7c624d1d8fbd0873a381c63a0858b4d75315bd8ca62e0111068bbf138e3");

        assert_eq!(result, code_hash.unwrap());
    }
}
