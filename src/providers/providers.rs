use worker::Result;

pub const ALL_PROVIDERS: &[&dyn Provider] = &[
    &Binance,
    &Bitstamp,
    &Bybit,
    &CoinbaseExchange,
    &Gate,
    &Kraken,
    &OKX,
];

pub trait Provider: 'static {
    fn url(&self, symbol: &str) -> String;
    fn parse_response(&self, body: &str) -> Result<f64>;

    // Test helpers
    #[cfg(test)]
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    #[cfg(test)]
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

pub struct Binance;
pub struct Bitstamp;
pub struct Bybit;
pub struct CoinbaseExchange;
pub struct Gate;
pub struct Kraken;
pub struct OKX;

impl Provider for Binance {
    fn url(&self, symbol: &str) -> String {
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
                .ok_or_else(|| format!("Failed to parse field {} in response from {}", key, self.url("<symbol>")))
        };

        let ask = extract("askPrice")?;
        let bid = extract("bidPrice")?;
        let mid = (ask + bid) / 2.0;

        Ok(mid)
    }
}

impl Provider for Bitstamp {
    fn url(&self, symbol: &str) -> String {
        format!(
            "https://www.bitstamp.net/api/v2/ticker/{}USD/",
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
                .ok_or_else(|| format!("Failed to parse field {} in response from {}", key, self.url("<symbol>")))
        };

        let ask = extract("ask")?;
        let bid = extract("bid")?;
        let mid = (ask + bid) / 2.0;

        Ok(mid)
    }
}

impl Provider for Bybit {
    fn url(&self, symbol: &str) -> String {
        // Only supports USDT
        format!(
            "https://api.bybit.com/v5/market/tickers?category=spot&symbol={}USDT",
            symbol.to_uppercase()
        )
    }

    fn parse_response(&self, body: &str) -> Result<f64> {
        let json: serde_json::Value = serde_json::from_str(body)?;

        let ticker_data = json
            .get("result")
            .and_then(|v| v.get("list"))
            .and_then(|v| v.get(0))
            .ok_or("Failed to locate ticker data in Bybit response")?;

        let extract = |key: &str| {
            ticker_data
                .get(key)
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .ok_or_else(|| format!("Failed to parse field {} in response from {}", key, self.url("<symbol>")))
        };

        let ask = extract("ask1Price")?;
        let bid = extract("bid1Price")?;
        let mid = (ask + bid) / 2.0;

        Ok(mid)
    }
}

impl Provider for CoinbaseExchange {
    fn url(&self, symbol: &str) -> String {
        format!(
            "https://api.exchange.coinbase.com/products/{}-USD/ticker",
            // Coinase Exchange only accepts capitalised symbols.
            symbol.to_uppercase()
        )
    }

    fn parse_response(&self, body: &str) -> Result<f64> {
        let json: serde_json::Value = serde_json::from_str(body)?;

        let extract = |key: &str| {
            json
                .get(key)
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .ok_or_else(|| format!("Failed to parse field {} in response from {}", key, self.url("<symbol>")))
        };

        let ask = extract("ask")?;
        let bid = extract("bid")?;
        let mid = (ask + bid) / 2.0;

        Ok(mid)
    }
}

impl Provider for Gate {
    fn url(&self, symbol: &str) -> String {
        // Only supports USDT
        format!(
            "https://api.gateio.ws/api/v4/spot/tickers?currency_pair={}_USDT",
            symbol.to_uppercase()
        )
    }

    fn parse_response(&self, body: &str) -> Result<f64> {
        let json: serde_json::Value = serde_json::from_str(body)?;

        let ticker_data = json
            .get(0)
            .ok_or("Failed to locate ticker data in Gate response")?;

        let extract = |key: &str| {
            ticker_data
                .get(key)
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .ok_or_else(|| format!("Failed to parse field {} in response from {}", key, self.url("<symbol>")))
        };

        let ask = extract("lowest_ask")?;
        let bid = extract("highest_bid")?;
        let mid = (ask + bid) / 2.0;

        Ok(mid)
    }
}

impl Provider for Kraken {
    fn url(&self, symbol: &str) -> String {
        format!(
            "https://api.kraken.com/0/public/Ticker?pair={}USD",
            symbol
        )
    }

    fn parse_response(&self, body: &str) -> Result<f64> {
        let json: serde_json::Value = serde_json::from_str(body)?;

        let ticker_data = json
            .get("result")
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.values().next())
            .ok_or("Failed to locate ticker data in Kraken response")?;

        let extract = |key: &str, index: usize| {
            ticker_data
                .get(key)
                .and_then(|v| v.get(index))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .ok_or_else(|| format!("Failed to parse field {} at index {} in response from {}", key, index, self.url("<symbol>")))
        };

        let ask = extract("a", 0)?;
        let bid = extract("b", 0)?;
        let mid = (ask + bid) / 2.0;

        Ok(mid)
    }
}

impl Provider for OKX {
    fn url(&self, symbol: &str) -> String {
        format!(
            "https://eea.okx.com/api/v5/market/ticker?instId={}-USDT",
            symbol.to_uppercase()
        )
    }

    fn parse_response(&self, body: &str) -> Result<f64> {
        let json: serde_json::Value = serde_json::from_str(body)?;

        let ticker_data = json
            .get("data")
            .and_then(|v| v.get(0))
            .ok_or("Failed to locate ticker data in OKX response")?;

        let extract = |key: &str| {
            ticker_data
                .get(key)
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .ok_or_else(|| format!("Failed to parse field {} in response from {}", key, self.url("<symbol>")))
        };

        let ask = extract("askPx")?;
        let bid = extract("bidPx")?;
        let mid = (ask + bid) / 2.0;

        Ok(mid)
    }
}