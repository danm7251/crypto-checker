use worker::Result;

#[derive(Debug)]
pub struct TickerData {
    pub last: f64,
    pub mid: f64,
    pub vwap: f64,
    pub vol: f64
}

pub trait Provider {
    fn url(&self, symbol: &str, fiat: &str) -> String;
    fn parse_response(&self, body: &str) -> Result<TickerData>;
}

pub struct Kraken;

impl Provider for Kraken {
    fn url(&self, symbol: &str, fiat: &str) -> String {
        format!(
            "https://api.kraken.com/0/public/Ticker?pair={}{}",
            symbol, fiat
        )
    }

    fn parse_response(&self, body: &str) -> Result<TickerData> {
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
                .ok_or_else(|| format!("Failed to parse field {} at index {}", key, index))
        };

        let ask = extract("a", 0)?;
        let bid = extract("b", 0)?;
        let last = extract("c", 0)?;
        let vwap = extract("p", 1)?;
        let vol = extract("v", 1)?;

        let mid = (ask + bid) / 2.0;

        Ok(TickerData { last, mid, vwap, vol })
    }
}
