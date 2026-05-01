use core::time;
use std::{collections::HashMap, os::raw, time::Duration};
use futures::future::{Either, join_all, select};
use worker::{web_sys::console::time, *};

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

    // Try and fetch responses from upstream data sources in parallel.
    let raw_results: Vec<Result<ResponseData>> = parallel_fetch(ALL_PROVIDERS, coin).await;

    // Discard failed responses.
    let results: Vec<ResponseData> = raw_results.into_iter().filter_map(|r| r.ok()).collect();

    // Extract prices, don't consume `results` as it's needed later in debug mode.
    let prices: Vec<f64> = results.iter().map(|r| r.price).collect();

    // Check if environment is in debug mode.
    let debug = env.var("DEBUG").map(|v| v.to_string() == "true").unwrap_or(false);

    match calculate_result(&prices) {
        Ok((avg_price, sources)) => {
            let mut json = serde_json::json!({
                "average_price": avg_price,
                "sources": sources,
            });

            if debug {
                // Extract API name-latency pairs from response data.
                let timings: serde_json::Map<String, serde_json::Value> = results
                    .iter()
                    .map(|r| (r.name.to_string(), serde_json::json!(format!("{}ms", r.elapsed_ms))))
                    .collect();

                // Append it to final response.
                json["debug"] = serde_json::Value::Object(timings);
            }

            Response::from_json(&json)
        },
        Err(e) => {
            Response::error(format!("{}", e), 503)
        }
    }
}

fn calculate_result(prices: &[f64]) -> Result<(f64, u8)> {
    let mut total_price = 0.0;
    let mut sources = 0;

    for price in prices {
        total_price += price;
        sources += 1;
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
        .map(|&p| timeout(p, symbol, 300)); 

    join_all(futures).await
}

async fn timeout(provider: &dyn Provider, symbol: &str, timeout_ms: u64) -> Result<ResponseData> {
    let fetch = Box::pin(fetch_response(provider, symbol));
    let timeout = Box::pin(worker::Delay::from(Duration::from_millis(timeout_ms)));

    match select(fetch, timeout).await {
        Either::Left((response, _)) => response,
        Either::Right(_) => Err(format!("{} timed out after {}ms", provider.name(), timeout_ms).into()),
    }
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
