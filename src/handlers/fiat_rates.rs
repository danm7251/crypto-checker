use futures::future::try_join_all;
use serde::Deserialize;
use worker::{Method::Get, *};

use crate::errors::AppError;

const FIAT_RATES_URL: &'static str = "https://api.frankfurter.dev/v2/rates?base=USD";

#[derive(Debug, Deserialize)]
struct FiatRate {
    quote: String,
    rate: f64
}

pub async fn sync_fiat_rates(env: &Env) -> Result<(), AppError> {
    // Fetch and parse upstream data
    let request = Request::new(FIAT_RATES_URL, Get)?;
    let mut response = Fetch::Request(request).send().await?;
    let body = response.text().await?;
    let rates: Vec<FiatRate> = serde_json::from_str(&body)?;

    // Write all pairs in parallel to KV store
    let store = env.kv("FIAT_RATES")?;
    try_join_all(
        rates.iter().map(|r| put(&store, &r.quote, r.rate))
    ).await?;

    Ok(())
}

async fn put(store: &KvStore, key: &str, val: f64) -> Result<(), AppError> {
    store.put(key, val)?.execute().await?;
    Ok(())
}