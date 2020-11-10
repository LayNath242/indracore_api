use crate::primitives;
use substrate_subxt::{balances::*, ClientBuilder, Error, IndracoreNodeRuntime};

pub async fn total_issuance() -> Result<u128, Error> {
    let client = match ClientBuilder::<IndracoreNodeRuntime>::new()
        .set_url(primitives::url())
        .build()
        .await
    {
        Ok(cli) => cli,
        Err(e) => return Err(e),
    };
    let total = match client.total_issuance(None).await {
        Ok(total) => total,
        Err(e) => return Err(e),
    };
    Ok(total)
}
