use worker::Result;

pub trait Provider {
    fn url(&self, symbol: &str, fiat: &str) -> String;
    fn parse_response(&self, body: &str) -> Result<serde_json::Value>;
}

pub struct Kraken;

impl Provider for Kraken {
    fn url(&self, symbol: &str, fiat: &str) -> String {
        format!(
            "https://api.kraken.com/0/public/Ticker?pair={}{}",
            symbol, fiat
        )
    }

    fn parse_response(&self, body: &str) -> Result<serde_json::Value> {
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
                .cloned()
                .ok_or_else(|| format!("Failed to parse field {} at index {}", key, index))
        };

        let _ask = extract("a", 0)?;
        let _bid = extract("b", 0)?;
        let _last = extract("c", 0)?;
        let vwap = extract("p", 1)?;
        let _vol = extract("v", 1)?;

        Ok(vwap)
    }
}
