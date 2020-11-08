use crate::primitives;
use std::{fs, io::Read, path::PathBuf};

pub struct ContractDeploy {
    pub wasm_path: PathBuf,
    pub signer: primitives::Signer,
}
use substrate_subxt::{contracts::*, ClientBuilder, Error, IndracoreNodeRuntime};

impl ContractDeploy {
    pub fn load_contract(&self) -> Result<Vec<u8>, Error> {
        let contract_wasm_path = self.wasm_path.clone();
        let mut data: Vec<u8> = Vec::new();

        let file = fs::File::open(&contract_wasm_path);
        let mut file = match file {
            Ok(f) => f,
            Err(_) => return Err(Error::Other("File not exit".into())),
        };
        match file.read_to_end(&mut data) {
            Ok(_) => Ok(data),
            Err(_) => Err(Error::Other("File Cannot be read".into())),
        }
    }

    ///put contract code to indracoe chain
    pub fn exec(&self) -> Result<sp_core::H256, Error> {
        let code = self.load_contract().unwrap();

        async_std::task::block_on(async move {
            let client = match ClientBuilder::<IndracoreNodeRuntime>::new()
                .set_url("ws://127.0.0.1:9944")
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
