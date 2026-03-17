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

        let price = json
            .get("result") // Retrieves the value at key="result"
            .and_then(|v| v.as_object()) // Converts the value to an object
            .and_then(|obj| obj.values().next()) // Retrieves the first value at the object
            .and_then(|v| v.get("p")) // Retrives the value at key="p"
            .and_then(|v| v.get(1)) // Retrives the value at index 1
            .cloned() // Clones the value to avoid it being dropped after the function since it belongs to `json`
            .ok_or("Failed to parse response from Kraken")?;

        Ok(price)
    }
}
