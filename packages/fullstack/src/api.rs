use dioxus::prelude::*;

// POST /api/login  (args are sent as a JSON body)
#[post("/api/login")]
pub async fn login(username: String, password: String) -> Result<String, ServerFnError> {
    // This body only runs on the server.
    // TODO: replace with a real lookup (argon2 hash check against CockroachDB).
    if username == "admin" && password == "secret" {
        Ok(format!("token-for-{username}"))
    } else {
        Err(ServerFnError::new("Invalid credentials"))
    }
}

// GET /api/whoami?token=...  (args listed after `?` become query params)
#[get("/api/whoami?token")]
pub async fn whoami(token: String) -> Result<String, ServerFnError> {
    token
        .strip_prefix("token-for-")
        .map(str::to_owned)
        .ok_or_else(|| ServerFnError::new("Bad token"))
}