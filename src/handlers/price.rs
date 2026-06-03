use std::{collections::HashMap, time::Duration};
use futures::future::{Either, join_all, select};
use worker::*;

use crate::{errors::AppError, providers::{ALL_PROVIDERS, Provider}};

const MIN_SOURCES: u8 = 2;
const SUPPORTED_FIAT: &[&str] = &["USD"];

pub async fn price(req: &Request, env: &Env) -> Result<Response, AppError> {
    // Extract environment variables
    let debug= env.var("DEBUG").map(|v| v.to_string() == "true").unwrap_or(false);
    let timeout_ms = env.var("TIMEOUT_MS").ok().and_then(|v| v.to_string().parse::<u64>().ok());

    // Extract query parameters from URL
    let params = query_params(req)?;

    // Validate query parameters
    let coin = match params.get("coin") {
        Some(coin) => coin,
        None => return Err(AppError::RequiredParameter { param: "coin" }),
    };

    // [!] Currently unnecessary
    let _fiat = match params.get("currency") {
        Some(fiat) => {
            let fiat_upper = fiat.to_uppercase();
            if !SUPPORTED_FIAT.contains(&fiat_upper.as_str()) {
                return Err(AppError::UnsupportedValue { param: "currency", value: fiat_upper, supported: SUPPORTED_FIAT });
            }
            fiat
        }
        None => return Err(AppError::RequiredParameter { param: "currency" }),
    };

    // Try and fetch responses from upstream data sources in parallel.
    // TODO: Distinguish non-timeout errors and log them. Currently all errors are ignored.
    let raw_results: Vec<Result<ResponseData, AppError>> = parallel_fetch(coin, timeout_ms).await;

    // Discard failed responses.
    let results: Vec<ResponseData> = raw_results.into_iter().filter_map(|r| r.ok()).collect();

    // Extract prices, don't consume `results` as it's needed later in debug mode.
    let prices: Vec<f64> = results.iter().map(|r| r.price).collect();

    let (avg_price, sources) = calculate_result(&prices)?;

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

    Ok(Response::from_json(&json)?)
}

fn calculate_result(prices: &[f64]) -> Result<(f64, u8), AppError> {
    let mut total_price = 0.0;
    let mut sources = 0;

    for price in prices {
        total_price += price;
        sources += 1;
    }

    if sources < MIN_SOURCES {
        return Err(AppError::InsufficientSources);
    }

    let avg_price = total_price / sources as f64;

    Ok((avg_price, sources))
}

async fn parallel_fetch(symbol: &str, timeout_ms: Option<u64>) -> Vec<Result<ResponseData, AppError>> {
    match timeout_ms {
        Some(t) => {
            join_all(
                ALL_PROVIDERS.iter().map(|&p| timeout(p, symbol, t))
            ).await
        },
        None => {
            join_all(
                ALL_PROVIDERS.iter().map(|&p| fetch_response(p, symbol))
            ).await
        }
    }
}

async fn timeout(provider: &dyn Provider, symbol: &str, timeout_ms: u64) -> Result<ResponseData, AppError> {
    let fetch = Box::pin(fetch_response(provider, symbol));
    let timeout = Box::pin(worker::Delay::from(Duration::from_millis(timeout_ms)));

    match select(fetch, timeout).await {
        Either::Left((response, _)) => response,
        Either::Right(_) => {
            let msg = format!("{} timed out after {}ms", provider.name(), timeout_ms);
            return Err(AppError::Internal { error: msg })
        }
    }
}

#[derive(Debug)]
struct ResponseData {
    name: &'static str,
    price: f64,
    elapsed_ms: u64,   
}

async fn fetch_response(provider: &dyn Provider, symbol: &str) -> Result<ResponseData, AppError> {
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
fn query_params(req: &Request) -> Result<HashMap<String, String>, AppError> {
    let url = req.url()?;
    Ok(url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect())
}
