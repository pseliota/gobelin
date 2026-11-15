# Gobelin

Tools and experiments for studying financial markets.

This product uses the FRED® API but is not endorsed or certified by the Federal Reserve Bank of St. Louis.

## Credentials

Values are read from the process environment first, then from `.env` in the current directory. Blank values count as missing.

| Credential | Variables, in order of preference |
|---|---|
| Alpaca key | `ALPACA_KEY`, `ALPACA_API_KEY`, `APCA_API_KEY_ID` |
| Alpaca secret | `ALPACA_SECRET`, `ALPACA_SECRET_KEY`, `ALPACA_API_SECRET`, `APCA_API_SECRET_KEY` |
| FRED key | `FRED_API_KEY` |

## Behaviour notes

The source carries no comments, so the facts a caller cannot read from a signature are kept here.

### All clients

- Requests time out after 20 seconds and never follow redirects.
- Errors never contain response bodies, URLs or credential values.
- Request models are sent as supplied. Dates, timeframes and trading rules are validated by the server, not locally.
- Response models keep wire representations: timestamps and dates stay strings, and unknown fields are ignored.

### Alpaca trading

- The client targets the paper trading API only.
- Every request is sent exactly once. Nothing is retried.
- A transport failure or timeout while submitting an order does not prove the order was rejected. Reconcile by `client_order_id` before submitting again.
- Replacing an order is a request: acceptance does not guarantee the original order was replaced.
- Cancelling an order is a request: HTTP 204 acknowledges it and says nothing about the final order state.
- Cancelling all orders returns one result per order. Each must be checked; the overall response succeeding does not mean every cancellation did.
- Listing orders returns one page: at most 500, and 50 when no limit is given.
- Decimal amounts are strings, exactly as Alpaca sends them.
- Calendar open and close times use `HH:MM` and session times use `HHMM`, both in Eastern time.

### Alpaca market data

- Prices are `f64`, because the API sends JSON numbers.
- The `limit` of a bars request counts bars across all symbols, so one page of a multi-symbol request can omit symbols. `stock_bars_pages` follows every page.
- The configured feed is sent with every request. Without one, Alpaca chooses according to the subscription.

### FRED

- The API key travels in the URL query string over HTTPS. Never log a request URL.
- Observation values are strings. `"."` marks a missing value, and `numeric_value` returns `None` for it.
- Observations and search results return one page. `next_offset` gives the offset of the next page, or `None` when there is none. The server's default observation limit is 100000.
- FRED revises data. Without `realtime_start` and `realtime_end` the latest revision is returned, which can differ from what was known at an earlier date.
- Series whose notes contain "Copyright" belong to third parties and are for personal use only.

### Retries

- Only the market data and FRED clients retry, and only GET requests.
- Transport errors, HTTP 429 and HTTP 5xx are retried with a delay that doubles from `base_delay` up to `max_delay`. `max_attempts` counts the first attempt; zero behaves as one.
- When Alpaca reports when its rate limit resets, the client waits until then, or gives up if that is longer than `max_delay`.
- A call can block for `max_attempts` timeouts plus the waits between them: up to 120 seconds with the default policy.

## Checks

```
cargo fmt --check
cargo clippy --all-targets --locked
cargo test --locked
ast-grep scan
```

`ast-grep scan` fails on any comment or docstring in Rust source.
