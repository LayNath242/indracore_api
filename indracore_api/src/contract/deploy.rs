use crate::primitives;
use std::{fs, io::Read, path::PathBuf};

pub struct ContractDeploy {
    pub wasm_path: PathBuf,
    pub signer: primitives::Signer,
}
use substrate_subxt::Error;

impl ContractDeploy {
    pub fn load_contract(&self) -> Result<Vec<u8>, Error> {
        let contract_wasm_path = self.wasm_path.clone();
        let mut data: Vec<u8> = Vec::new();

        let file = fs::File::open(&contract_wasm_path);
        let mut file = match file {
            Ok(f) => f,
            Err(_) => return Err(Error::Other("File not exit".to_string())),
        };
        match file.read_to_end(&mut data) {
            Ok(_) => Ok(data),
            Err(_) => Err(Error::Other("File Cannot be read".to_string())),
        }
    }
}
