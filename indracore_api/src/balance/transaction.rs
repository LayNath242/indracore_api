use substrate_subxt::{
    balances::*, sp_core, sp_core::Decode, ClientBuilder, Error, EventSubscription, EventsDecoder,
    IndracoreNodeRuntime,
};

use crate::{keyring::Sr25519, primitives};

pub struct Transaction {
    pub sender: Sr25519,
    pub reciever: primitives::IndracoreId,
    pub amount: u128,
}

impl Transaction {
    pub fn run(&self, pass: Option<&str>) -> Result<sp_core::H256, Error> {
        let sender = match self.sender.pair(pass) {
            Ok(pair) => pair,
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

            let sub = match client.subscribe_events().await {
                Ok(s) => s,
                Err(e) => return Err(e),
            };

            let mut decoder = EventsDecoder::<IndracoreNodeRuntime>::new(client.metadata().clone());
            decoder.with_balances();
            let mut sub = EventSubscription::<IndracoreNodeRuntime>::new(sub, decoder);
            sub.filter_event::<TransferEvent<_>>();

            let hash = match client.transfer(&sender, &self.reciever, self.amount).await {
                Ok(hash) => hash,
                Err(e) => return Err(e),
            };

            let raw = sub.next().await.unwrap().unwrap();
            let event = TransferEvent::<IndracoreNodeRuntime>::decode(&mut &raw.data[..]);
            if let Ok(event) = event {
                println!("{:?}", event)
            } else {
                return Err(Error::Other(
                    "!!! Failed to subscribe to Balances::Transfer Event".into(),
                ));
            }
            Ok(hash)
        })
    }
}
