    use std::collections::HashMap;
    use futures::future::join_all;
    use worker::*;

    use crate::providers::{ALL_PROVIDERS, Provider};

    const SUPPORTED_FIAT: &[&str] = &["USD"];

    pub async fn price(req: &Request) -> Result<Response> {
        // Extract query parameters from URL
        let params = query_params(req)?;

        // Validate query parameters
        let coin = match params.get("coin") {
            Some(coin) => coin,
            None => return Response::error("Missing required parameter: coin", 400),
        };

        // [!] Currently unnecessary
        let _fiat = match params.get("currency") {
            Some(fiat) => {
                if !SUPPORTED_FIAT.contains(&fiat.to_uppercase().as_str()) {
                    return Response::error("Unsupported currency", 400);
                }
                fiat
            }
            None => return Response::error("Missing required parameter: currency", 400),
        };

        let raw_results: Vec<Result<f64>> = serial_fetch(ALL_PROVIDERS, coin).await;

        let (avg_price, sources) = calculate_result(&raw_results);

        Response::from_json(&serde_json::json!({
            "average_price": avg_price,
            "sources": sources,
        }))
    }

fn calculate_result(results: &[Result<f64>]) -> (f64, u8) {
    let mut total_price = 0.0;
    let mut sources = 0;

    for result in results {
        console_log!("{:?}", result);

        if let Ok(price) = result {
            total_price += price;
            sources += 1;
        }
    }

    let avg_price = total_price / sources as f64;

    (avg_price, sources)
}

#[allow(dead_code)]
async fn parallel_fetch(providers: &[&dyn Provider], symbol: &str) -> Vec<Result<f64>> {
    let futures = providers
        .iter()
        .map(|&p| fetch_response(p, symbol)); 

    join_all(futures).await
}

async fn serial_fetch(providers: &[&dyn Provider], symbol: &str) -> Vec<Result<f64>> {
    let mut results = Vec::<Result<f64>>::new();
    
    for provider in providers {
        results.push(fetch_response(*provider, symbol).await);
    }

    results
}

async fn fetch_response(provider: &dyn Provider, symbol: &str) -> Result<f64> {
    let uri = provider.url(symbol);

    let headers = Headers::new();
    headers.set("Accept", "application/json")?;

    let mut init = RequestInit::new();
    init.with_headers(headers);

    let request = Request::new_with_init(&uri, &init)?;

    let mut response = Fetch::Request(request).send().await?;

    let body = response.text().await?;

    provider.parse_response(&body)
}

// Parses a Request into a HashMap of query parameters
fn query_params(req: &Request) -> Result<HashMap<String, String>> {
    let url = req.url()?;
    Ok(url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect())
}
