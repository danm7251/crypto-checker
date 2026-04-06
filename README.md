# crypto-checker

A Cloudflare Worker that returns the average crypto price across multiple exchanges.

For a given symbol, it fetches the bid/ask ticker from 7 exchanges in parallel, computes each mid-price, and returns the mean. At least 2 sources must succeed or the request fails with a 503.

**Exchanges:** Binance, Bitstamp, Bybit, Coinbase Exchange, Gate.io, Kraken, OKX

## API
```
GET /v1/price?coin={symbol}&currency=USD
```
```json
{
  "average_price": 66800.42,
  "sources": 7
}
```

Errors: `400` for bad params, `503` if too few exchanges respond.

## Development

**Prerequisites**
- [Rust](https://rustup.rs/) with the `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
- Node.js (for Wrangler)
- A Cloudflare account is only required to deploy — `wrangler dev` works without one
```bash
cargo test          # unit tests
npx wrangler dev    # local dev server
npx wrangler deploy # deploy (requires Cloudflare account)
```

### TODO:
#### Pre-release
- [ ] - Add a timeout using a fetch builder
- [ ] - Solidify error handling
- [ ] - Integration tests
- [ ] - RapidAPI integration
- [ ] - Critical error notifications

#### Post-release
- [ ] - Refactor Provider trait to be compatible with multiple endpoints
- [ ] - Multiple major currencies
- [ ] - Price caching
- [ ] - Forex rate caching cron job
- [ ] - Upstream health check cron job
