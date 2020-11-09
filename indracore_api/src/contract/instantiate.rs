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
                Ok(c) => c,
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
