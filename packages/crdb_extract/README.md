# crdb_extract

A tiny Rust library that takes **one query string** and returns the result from
CockroachDB as **JSON built by the database itself**. The connection url is read
from the `DATABASE_URL` environment variable.

CockroachDB speaks the Postgres wire protocol, so this crate is a thin layer over
[`sqlx`](https://crates.io/crates/sqlx), with [`backon`](https://crates.io/crates/backon)
for retries. There is no per-column decoding: the query is wrapped as

```sql
SELECT coalesce(jsonb_agg(row_to_json(t)), '[]'::JSONB) FROM (<your query>) AS t
```

so every column type CockroachDB supports arrives as ordinary JSON.

## API

| Function | Description |
|---|---|
| `Extractor::from_env()` | Connect using `DATABASE_URL` |
| `Extractor::connect(url)` | Connect using an explicit url |
| `extractor.fetch_json(query)` | Run `query`, return a `serde_json::Value` array of row objects |

`query` is either:

- a **bare table name** (`"users"` or `"app.users"`; letters, digits and underscores only), which returns the whole table, or
- a **`SELECT` statement**, which returns its rows.

An empty result is `[]`. Serialization failures (SQLSTATE `40001`) are retried automatically with exponential backoff (up to 3 times).

## Usage

```rust
use crdb_extract::Extractor;

#[tokio::main]
async fn main() -> Result<(), crdb_extract::Error> {
    // export DATABASE_URL='postgresql://user:pw@host:26257/mydb?sslmode=verify-full'
    let db = Extractor::from_env().await?;

    let whole_table = db.fetch_json("users").await?;
    let adults = db
        .fetch_json("SELECT id, name FROM users WHERE age >= 18 ORDER BY id")
        .await?;

    println!("{whole_table}\n{adults}");
    Ok(())
}
```

## Notes

- **Requirements:** Rust 1.94 or newer (sqlx 0.9).
- **Trusted SQL only.** The query string is run as-is. Never build it from untrusted input. Under sqlx 0.9 the crate has to assert this with `AssertSqlSafe`; that assertion is only fully checked for bare table names. A single token that is not a plain table name (for example `generate_series(1,2)`) is rejected.
- **Use a `SELECT`-only database role.** This is the only real protection. Every connection defaults to `default_transaction_read_only = on` and a 30 second `statement_timeout`, but that only guards against accidents: a query can call `set_config(...)` to switch both off for the rest of that pooled connection (CockroachDB supports session-scoped `set_config`). Do not share one `Extractor` between trusted and untrusted queries.
- **Unique column names.** Each row becomes a JSON object, so alias expressions with `AS`.
- **Whole result in memory.** The database returns one JSON document, so very large tables will use a lot of memory on both sides. Limit with `LIMIT`/`WHERE` or page through the table.
- **Row order.** An `ORDER BY` inside your query is expected to carry through the wrapper; verify on your cluster version if strict order matters.
- **TLS and auth** come from the url (`sslmode`, `sslrootcert`, ...). `sqlx` defaults to `sslmode=prefer`, which does not verify the server; use `sslmode=verify-full` with `sslrootcert` outside local development.
- **Pool timeout:** acquiring a connection gives up after 5 seconds, both when connecting and when every pooled connection is busy.
- **Errors** are plain `sqlx::Error` (re-exported as `crdb_extract::Error`). A missing or unreadable `DATABASE_URL`, and any connection failure, are reported as `Error::Configuration` with a message like `could not connect to user@host:26257/db: <cause>`. The password is never included. These messages name the host and user, so log them rather than returning them to external clients. When the pool only reports a timeout, the crate makes one direct connection attempt to find the underlying cause (refused, TLS, auth, no response).

## Tests

```sh
cargo test                              # unit tests only (offline); live tests show as ignored
DATABASE_URL=... cargo test -- --ignored   # runs the live tests against your cluster
```