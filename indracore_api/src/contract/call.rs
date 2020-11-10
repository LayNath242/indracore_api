use crate::contract::transcode::Transcoder;
use crate::primitives;
use substrate_subxt::{
    balances::Balances, contracts::*, system::System, ClientBuilder, Error, ExtrinsicSuccess,
    IndracoreNodeRuntime,
};

pub struct ContarctCall {
    pub name: String,
    pub args: Vec<String>,
    pub metadata: String,
    pub signer: primitives::Sr25519,
    pub value: <IndracoreNodeRuntime as Balances>::Balance,
    pub gas_limit: u64,
    pub contract: <IndracoreNodeRuntime as System>::Address,
}

impl ContarctCall {
    async fn call(&self, data: Vec<u8>) -> Result<ExtrinsicSuccess<IndracoreNodeRuntime>, Error> {
        let client = match ClientBuilder::<IndracoreNodeRuntime>::new()
            .set_url(primitives::url())
            .build()
            .await
        {
            Ok(cli) => cli,
            Err(e) => return Err(e),
        };
        let extrinsic_success = client
            .call_and_watch(
                &self.signer,
                &self.contract,
                self.value,
                self.gas_limit,
                &data,
            )
            .await?;
        Ok(extrinsic_success)
    }

    pub fn run(&self) -> Result<ExtrinsicSuccess<IndracoreNodeRuntime>, Error> {
        let metadata = match super::load_metadata(&self.metadata) {
            Ok(m) => m,
            Err(e) => return Err(Error::Other(format!("{:?}", e))),
        };

        let transcoder = Transcoder::new(metadata);
        let data = match transcoder.encode(&self.name, &self.args) {
            Ok(m) => m,
            Err(e) => return Err(Error::Other(format!("{:?}", e))),
        };
        let result = async_std::task::block_on(self.call(data))?;

        Ok(result)
    }
}
