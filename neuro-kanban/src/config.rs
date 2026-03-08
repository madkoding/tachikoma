use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub backend_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3006".to_string())
                .parse()
                .expect("PORT must be a number"),
            backend_url: env::var("BACKEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        }
    }
}
