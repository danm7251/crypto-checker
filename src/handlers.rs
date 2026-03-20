use std::collections::HashMap;
use worker::*;

use crate::providers::{self, Provider};

const SUPPORTED_FIAT: &[&str] = &["USD"];

pub async fn price(req: &Request) -> Result<Response> {
    // Extract query parameters from URL
    let params = query_params(req)?;

    // Validate query parameters
    let coin = match params.get("coin") {
        Some(coin) => coin,
        None => return Response::error("Missing required parameter: coin", 400),
    };

    let fiat = match params.get("currency") {
        Some(fiat) => {
            if !SUPPORTED_FIAT.contains(&fiat.to_uppercase().as_str()) {
                return Response::error("Unsupported currency", 400);
            }
            fiat
        }
        None => return Response::error("Missing required parameter: currency", 400),
    };

    let ticker_data = providers::Kraken.fetch_data(coin, fiat).await?;

    Response::from_json(&serde_json::json!({
        "last": ticker_data.last,
        "mid": ticker_data.mid,
        "vwap": ticker_data.vwap,
        "vol": ticker_data.vol
    }))
}

// Parses a Request into a HashMap of query parameters
fn query_params(req: &Request) -> Result<HashMap<String, String>> {
    let url = req.url()?;
    Ok(url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect())
}
