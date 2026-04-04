use std::{any::TypeId, collections::HashSet};

use crate::providers::*;

// Checks that parsing is successful for each providers standard JSON response
#[test]
fn test_json_mapping() {
    struct TestCase {
        name: &'static str,
        provider: Box<dyn Provider>,
        input: &'static str
    }

    let cases: Vec<TestCase> = vec![
        TestCase {
            name: "Binance",
            provider: Box::new(Binance),
            input: r#"{
                "symbol": "BTCUSDT",
                "priceChange": "864.60000000",
                "priceChangePercent": "1.311",
                "weightedAvgPrice": "66199.41293310",
                "prevClosePrice": "65949.39000000",
                "lastPrice": "66814.00000000",
                "lastQty": "0.09147000",
                "bidPrice": "66814.00000000",
                "bidQty": "0.40852000",
                "askPrice": "66814.01000000",
                "askQty": "2.16545000",
                "openPrice": "65949.40000000",
                "highPrice": "67163.99000000",
                "lowPrice": "65548.25000000",
                "volume": "18313.12533000",
                "quoteVolume": "1212318145.81622020",
                "openTime": 1774621542003,
                "closeTime": 1774707942003,
                "firstId": 6158704736,
                "lastId": 6161434531,
                "count": 2729796
            }"#
        },
        TestCase {
            name: "Bitstamp",
            provider: Box::new(Bitstamp),
            input: r#"{
                "last": "2211.00",
                "high": "2811.00",
                "low": "2188.97",
                "vwap": "2189.80",
                "volume": "213.26801100",
                "bid": "2188.97",
                "ask": "2211.00",
                "timestamp": "1643640186",
                "open": "2211.00",
                "open_24": "2211.00",
                "percent_change_24": "13.57",
                "side": "0",
                "market_type": "SPOT",
                "mark_price": "2812.00",
                "index_price": "2814.00",
                "open_interest": "10.1",
                "open_interest_value": "10234.00"
            }"#
        },
        TestCase {
            name: "Bybit",
            provider: Box::new(Bybit),
            input: r#"{
                "retCode": 0,
                "retMsg": "OK",
                "result": {
                    "category": "spot",
                    "list": [
                        {
                            "symbol": "BTCUSDT",
                            "bid1Price": "66782.2",
                            "bid1Size": "0.463827",
                            "ask1Price": "66782.3",
                            "ask1Size": "1.215596",
                            "lastPrice": "66782.2",
                            "prevPrice24h": "66787.8",
                            "price24hPcnt": "-0.0001",
                            "highPrice24h": "67384.1",
                            "lowPrice24h": "66281.9",
                            "turnover24h": "344828352.73575856",
                            "volume24h": "5159.246799"
                        }
                    ]
                },
                "retExtInfo": {},
                "time": 1775239921132
            }"#
        },
        TestCase {
            name: "Coinbase Exchange",
            provider: Box::new(CoinbaseExchange),
            input: r#"{
                "trade_id": 86326522,
                "price": "6268.48",
                "size": "0.00698254",
                "time": "2020-03-20T00:22:57.833Z",
                "bid": "6265.15",
                "ask": "6267.71",
                "volume": "53602.03940154",
                "rfq_volume": "123.122",
                "conversions_volume": "0.00"
            }"#
        },
        TestCase {
            name: "Gate",
            provider: Box::new(Gate),
            input: r#"[
                {
                    "currency_pair": "BTC_USDT",
                    "last": "66958.7",
                    "lowest_ask": "66959",
                    "lowest_size": "9.160064",
                    "highest_bid": "66958.9",
                    "highest_size": "2.875639",
                    "change_percentage": "0.18",
                    "base_volume": "5039.281754",
                    "quote_volume": "336709375.7657289",
                    "high_24h": "67352.7",
                    "low_24h": "66284"
                }
            ]"#
        },
        TestCase {
            name: "Kraken",
            provider: Box::new(Kraken),
            input: r#"{
                "error": [],
                "result": {
                    "XXBTZUSD": {
                        "a": [
                            "66740.70000",
                            "1",
                            "1.000"
                        ],
                        "b": [
                            "66740.60000",
                            "2",
                            "2.000"
                        ],
                        "c": [
                            "66740.60000",
                            "0.00306352"
                        ],
                        "v": [
                            "835.60648728",
                            "1738.28564104"
                        ],
                        "p": [
                            "66398.23711",
                            "66191.08579"
                        ],
                        "t": [
                            22907,
                            44874
                        ],
                        "l": [
                            "65889.00000",
                            "65519.70000"
                        ],
                        "h": [
                            "67101.00000",
                            "67101.00000"
                        ],
                        "o": "66360.40000"
                    }
                }
            }"#
        },
        TestCase {
            name: "OKX",
            provider: Box::new(OKX),
            input: r#"{
                "code": "0",
                "msg": "",
                "data": [
                    {
                        "instType": "SPOT",
                        "instId": "BTC-USDT",
                        "last": "66256.8",
                        "lastSz": "0.0000152",
                        "askPx": "66260.3",
                        "askSz": "0.22152927",
                        "bidPx": "66260.2",
                        "bidSz": "0.43439376",
                        "open24h": "66724.9",
                        "high24h": "67124.6",
                        "low24h": "66159.7",
                        "volCcy24h": "204987308.1880515",
                        "vol24h": "3074.77151423",
                        "ts": "1774805577108",
                        "sodUtc0": "66369",
                        "sodUtc8": "66526.2"
                    }
                ]
            }"#
        }
    ];

    let covered: HashSet<TypeId> = cases
        .iter()
        .map(|case| case.provider.type_id())
        .collect();

    let missing: Vec<&str> = ALL_PROVIDERS
        .iter()
        .filter(|p| !covered.contains(&p.type_id()))
        .map(|p| p.name())
        .collect();

    if !missing.is_empty() {
        panic!("\n\x1b[31m[!] ERROR: Missing test cases for: {:?}\x1b[0m\n", missing);
    }

    for case in cases {
        if let Err(e) = case.provider.parse_response(case.input) {
            panic!("\n\x1b[31m[!] ERROR: Case = {}\n\t{}\n\x1b[0m\n", case.name, e)
        }
    }
}