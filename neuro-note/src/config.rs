use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub backend_url: String,
    pub voice_service_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3007".to_string())
                .parse()
                .unwrap_or(3007),
            backend_url: env::var("BACKEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            voice_service_url: env::var("VOICE_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8100".to_string()),
        }
    }
}
