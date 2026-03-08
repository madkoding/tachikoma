use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub backend_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3009".to_string())
                .parse()
                .unwrap_or(3009),
            backend_url: env::var("BACKEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        }
    }
}
