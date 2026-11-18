// Server settings read from environment variables at startup.

use std::env;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
    pub admin_name: String,
    pub admin_email: String,
    pub admin_password: String,
    pub static_dir: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: required("DATABASE_URL"),

            jwt_secret: required("JWT_SECRET"),

            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),

            admin_name: env::var("ADMIN_NAME")
                .unwrap_or_else(|_| "Admin".into()),

            admin_email: env::var("ADMIN_EMAIL")
                .unwrap_or_else(|_| "admin@crm.local".into()),

            admin_password: env::var("ADMIN_PASSWORD")
                .unwrap_or_else(|_| "admin12345".into()),

            static_dir: env::var("STATIC_DIR")
                .ok()
                .filter(|d| !d.is_empty()),
        }
    }
}

fn required(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("{} is required", name))
}


