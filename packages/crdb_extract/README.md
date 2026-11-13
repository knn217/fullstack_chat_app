# crdb_extract

A tiny Rust library that takes **one query string** and returns the result from
CockroachDB as **JSON built by the database itself**. The connection url is read
from the `DATABASE_URL` environment variable. It can also run `INSERT`, `UPDATE` and
`DELETE` statements and report how many rows they affected.

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
| `extractor.execute(statement)` | Run an `INSERT`, `UPDATE` or `DELETE` (or any single statement); returns the number of rows affected |

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

    // Modify data: the statement is run as-is, returns rows affected
    let inserted = db.execute("INSERT INTO users (id, name) VALUES (7, 'Ada')").await?;
    let updated = db.execute("UPDATE users SET name = 'Ada L.' WHERE id = 7").await?;
    let deleted = db.execute("DELETE FROM users WHERE id = 7").await?;

    println!("{whole_table}\n{adults}\n{inserted} {updated} {deleted}");
    Ok(())
}
```

## Safety

`fetch_json` and `execute` run the string you give them **exactly as written**. The crate does no
escaping, filtering or parameter binding, and under sqlx 0.9 it has to assert this with
`AssertSqlSafe`. Everything below is your responsibility.

- **Never put untrusted input into a statement.** There are no bind parameters, so interpolating user values is SQL injection. Validate or escape values yourself, or keep statements constant.
- **Use a least-privileged database role, and never `root` or an admin role.** This is the only real protection. Prefer separate roles (and separate `Extractor`s) for reading and for writing, so the reader holds only `SELECT`.
- **Do not treat `fetch_json` as read-only.** CockroachDB accepts statements such as `[INSERT ... RETURNING ...]` there, and a statement can lift the crate's 30 second timeout with `set_config(...)`.
- **Always use a `WHERE` on `UPDATE` and `DELETE`.** `DELETE FROM users` deletes every row. Run the same condition as a `SELECT` first, and check the returned row count. CockroachDB's `sql_safe_updates` setting rejects `UPDATE`/`DELETE` without `WHERE`; consider enabling it for your role.
- **Each call is its own transaction.** There is no way to make two calls atomic. Do a multi-step change in one statement (for example a CTE) or use `sqlx` directly.
- **Make writes idempotent.** Serialization failures (SQLSTATE `40001`) are retried automatically, so a statement can run again after an aborted attempt. An "ambiguous result" error (`40003`) is not retried; check the data before repeating the write. `INSERT ... ON CONFLICT DO NOTHING` and `UPSERT` are safe to repeat.
- **Batch large changes.** Every statement is cancelled and rolled back after 30 seconds. Split big updates and deletes into chunks (for example `DELETE ... WHERE ... LIMIT 1000` in a loop).
- **Use `sslmode=verify-full`.** Writes over an unverified or plaintext connection can be read or tampered with.
- **Take a backup or try it on a copy first** for anything destructive, including `DROP` and `TRUNCATE`.
- **Be careful with logs and errors.** Server errors can echo statement text and data, and connection errors name the host and user. Log them, but do not return them to external clients.

## Notes

- **Requirements:** Rust 1.94 or newer (sqlx 0.9).
- **Unique column names.** Each row becomes a JSON object, so alias expressions with `AS`.
- **Whole result in memory.** The database returns one JSON document, so very large tables will use a lot of memory on both sides. Limit with `LIMIT`/`WHERE` or page through the table.
- **Row order.** An `ORDER BY` inside your query is expected to carry through the wrapper; verify on your cluster version if strict order matters.
- **TLS and auth** come from the url (`sslmode`, `sslrootcert`, ...). `sqlx` defaults to `sslmode=prefer`, which does not verify the server; use `sslmode=verify-full` with `sslrootcert` outside local development.
- **Pool timeout:** acquiring a connection gives up after 5 seconds, both when connecting and when every pooled connection is busy.
- **Errors** are plain `sqlx::Error` (re-exported as `crdb_extract::Error`). A missing or unreadable `DATABASE_URL`, and any connection failure, are reported as `Error::Configuration` with a message like `could not connect to user@host:26257/db: <cause>`. The password is never included. These messages name the host and user, so log them rather than returning them to external clients. When the pool only reports a timeout, the crate makes one direct connection attempt to find the underlying cause (refused, TLS, auth, no response).

## Tests

```sh
cargo test                              # unit tests only (offline); live tests show as ignored
DATABASE_URL=... cargo test -- --ignored   # runs the live tests (one creates and drops a scratch table)
```