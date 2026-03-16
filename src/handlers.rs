use std::collections::HashMap;

use worker::*;

const SUPPORTED_FIAT: &[&str] = &["USD"];

pub async fn price(req: &Request) -> Result<Response> {
    // Extract query parameters from URL
    let params = query_params(req)?;

    // Validate query parameters
    let _coin = match params.get("coin") {
        Some(coin) => coin,
        None => return Response::error("Missing required parameter: coin", 400),
    };

    let _currency = match params.get("currency") {
        Some(currency) => {
            if !SUPPORTED_FIAT.contains(&currency.to_uppercase().as_str()) {
                return Response::error("Unsupported currency", 400);
            }
            currency
        }
        None => return Response::error("Missing required parameter: currency", 400),
    };

    Response::error("Under development", 503)
}

// Parses a Request into a HashMap of query parameters
fn query_params(req: &Request) -> Result<HashMap<String, String>> {
    let url = req.url()?;
    Ok(url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect())
}
