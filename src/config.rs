#[derive(Clone, Debug)]
pub struct Config {
    pub database_url:      String,
    pub jwt_secret:        String,
    pub jwt_expiry_hours:  u64,
    pub port:              u16,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Config {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| "DATABASE_URL must be set")?,
            jwt_secret: std::env::var("JWT_SECRET")
                .map_err(|_| "JWT_SECRET must be set")?,
            jwt_expiry_hours: std::env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".into())
                .parse()
                .unwrap_or(24),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .unwrap_or(3000),
        })
    }
}
