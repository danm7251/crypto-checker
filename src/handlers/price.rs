use std::collections::HashMap;
use futures::future::join_all;
use worker::*;

use crate::providers::{ALL_PROVIDERS, Provider};

const MIN_SOURCES: u8 = 2;
const SUPPORTED_FIAT: &[&str] = &["USD"];

pub async fn price(req: &Request, env: &Env) -> Result<Response> {
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

    let raw_results: Vec<Result<ResponseData>> = parallel_fetch(ALL_PROVIDERS, coin).await;

    let debug = env.var("DEBUG").map(|v| v.to_string() == "true").unwrap_or(false);

    match calculate_result(&raw_results) {
        Ok((avg_price, sources)) => {
            let mut json = serde_json::json!({
                "average_price": avg_price,
                "sources": sources,
            });

            if debug {
                let timings: serde_json::Map<String, serde_json::Value> = raw_results
                    .iter()
                    .filter_map(|r| r.as_ref().ok())
                    .map(|r| (r.name.to_string(), serde_json::json!(format!("{}ms", r.elapsed_ms))))
                    .collect();

                json["debug"] = serde_json::Value::Object(timings);
            }

            Response::from_json(&json)
        },
        Err(e) => {
            Response::error(format!("{}", e), 503)
        }
    }
}

fn calculate_result(results: &[Result<ResponseData>]) -> Result<(f64, u8)> {
    let mut total_price = 0.0;
    let mut sources = 0;

    for result in results {
        if let Ok(response_data) = result {
            total_price += response_data.price;
            sources += 1;
        }
    }

    if sources <= MIN_SOURCES {
        return Err("Insufficient sources".into());
    }

    let avg_price = total_price / sources as f64;

    Ok((avg_price, sources))
}

async fn parallel_fetch(providers: &[&dyn Provider], symbol: &str) -> Vec<Result<ResponseData>> {
    let futures = providers
        .iter()
        .map(|&p| fetch_response(p, symbol)); 

    join_all(futures).await
}

#[derive(Debug)]
struct ResponseData {
    name: &'static str,
    price: f64,
    elapsed_ms: u64,   
}

async fn fetch_response(provider: &dyn Provider, symbol: &str) -> Result<ResponseData> {
    let uri = provider.url(symbol);

    let headers = Headers::new();
    headers.set("Accept", "application/json")?;

    let mut init = RequestInit::new();
    init.with_headers(headers);

    let request = Request::new_with_init(&uri, &init)?;
    let start_time = worker::Date::now().as_millis();
    let mut response = Fetch::Request(request).send().await?;
    let elapsed_ms = worker::Date::now().as_millis() - start_time;

    let body = response.text().await?;
    let price = provider.parse_response(&body)?;

    Ok(ResponseData {
        name: provider.name(),
        price,
        elapsed_ms
    })
}

// Parses a Request into a HashMap of query parameters
fn query_params(req: &Request) -> Result<HashMap<String, String>> {
    let url = req.url()?;
    Ok(url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect())
}
