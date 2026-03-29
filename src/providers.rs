use worker::Result;

pub const ALL_PROVIDERS: &[&dyn Provider] = &[
    &Binance,
    &Bitstamp,
    &CoinbaseExchange,
    &Kraken,
];

pub trait Provider {
    fn url(&self, symbol: &str, fiat: &str) -> String;
    fn parse_response(&self, body: &str) -> Result<f64>;
}

pub struct Binance;
pub struct Bitstamp;
pub struct CoinbaseExchange;
pub struct Kraken;

impl Provider for Binance {
    fn url(&self, symbol: &str, _fiat: &str) -> String {
        format!(
            "https://api.binance.com/api/v3/ticker/24hr?symbol={}USDT",
            symbol
        )
    }

    fn parse_response(&self, body: &str) -> Result<f64> {
        let json: serde_json::Value = serde_json::from_str(body)?;

        let extract = |key: &str| {
            json
                .get(key)
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .ok_or_else(|| format!("Failed to parse field {} in response from {}", key, self.url("<pair>", "")))
        };

        let ask = extract("askPrice")?;
        let bid = extract("bidPrice")?;
        let mid = (ask + bid) / 2.0;

        Ok(mid)
    }
}

impl Provider for Bitstamp {
    fn url(&self, symbol: &str, fiat: &str) -> String {
        format!(
            "https://www.bitstamp.net/api/v2/ticker/{}{}/",
            symbol, fiat
        )
    }

    fn parse_response(&self, body: &str) -> Result<f64> {
        let json: serde_json::Value = serde_json::from_str(body)?;

        let extract = |key: &str| {
            json
                .get(key)
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .ok_or_else(|| format!("Failed to parse field {} in response from {}", key, self.url("<pair>", "")))
        };

        let ask = extract("ask")?;
        let bid = extract("bid")?;
        let mid = (ask + bid) / 2.0;

        Ok(mid)
    }
}

impl Provider for CoinbaseExchange {
    fn url(&self, symbol: &str, fiat: &str) -> String {
        format!(
            "https://api.exchange.coinbase.com/products/{}-{}/ticker",
            // Coinase Exchange only accepts capitalised symbols.
            symbol.to_uppercase(), fiat.to_uppercase()
        )
    }

    fn parse_response(&self, body: &str) -> Result<f64> {
        let json: serde_json::Value = serde_json::from_str(body)?;

        let extract = |key: &str| {
            json
                .get(key)
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .ok_or_else(|| format!("Failed to parse field {} in response from {}", key, self.url("<pair>", "")))
        };

        let ask = extract("ask")?;
        let bid = extract("bid")?;
        let mid = (ask + bid) / 2.0;

        Ok(mid)
    }
}

impl Provider for Kraken {
    fn url(&self, symbol: &str, fiat: &str) -> String {
        format!(
            "https://api.kraken.com/0/public/Ticker?pair={}{}",
            symbol, fiat
        )
    }

    fn parse_response(&self, body: &str) -> Result<f64> {
        let json: serde_json::Value = serde_json::from_str(body)?;

        let ticker_data = json
            .get("result")
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.values().next())
            .ok_or("Failed to locate Ticker data in Kraken response")?;

        let extract = |key: &str, index: usize| {
            ticker_data
                .get(key)
                .and_then(|v| v.get(index))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .ok_or_else(|| format!("Failed to parse field {} at index {} in response from {}", key, index, self.url("<pair>", "")))
        };

        let ask = extract("a", 0)?;
        let bid = extract("b", 0)?;
        let mid = (ask + bid) / 2.0;

        Ok(mid)
    }
}