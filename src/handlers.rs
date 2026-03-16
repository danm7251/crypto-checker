use std::collections::HashMap;

use worker::*;

const SUPPORTED_FIAT: &[&str] = &["USD"];
const UPSTREAM_PROVIDER: &str = "https://api.kraken.com/0/public/Ticker";

pub async fn price(req: &Request) -> Result<Response> {
    // Extract query parameters from URL
    let params = query_params(req)?;

    // Validate query parameters
    let coin = match params.get("coin") {
        Some(coin) => coin,
        None => return Response::error("Missing required parameter: coin", 400),
    };

    let currency = match params.get("currency") {
        Some(currency) => {
            if !SUPPORTED_FIAT.contains(&currency.to_uppercase().as_str()) {
                return Response::error("Unsupported currency", 400);
            }
            currency
        }
        None => return Response::error("Missing required parameter: currency", 400),
    };

    // Construct upstream request
    let uri = format!("{}?pair={}{}", UPSTREAM_PROVIDER, &coin, &currency);

    let headers = Headers::new();
    headers.set("Accept", "application/json")?;

    let mut init = RequestInit::new();
    init.with_headers(headers);

    let _request = Request::new_with_init(&uri, &init)?;

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
