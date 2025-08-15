use std::env;

/// Application configuration loaded from environment variables.
///
/// - PORT: u16, defaults to 8080
/// - DATABASE_URL: String, defaults to "sqlite:data/todos.db"
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub port: u16,
    pub database_url: String,
}

impl AppConfig {
    /// Load configuration from the environment and optional .env file.
    ///
    /// Order of resolution:
    /// 1. .env (if present)
    /// 2. Process environment
    /// 3. Defaults
    pub fn from_env() -> Self {
        // Best-effort load of .env; ignore if missing.
        let _ = dotenvy::dotenv();

        let port = env::var("PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(8080);

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data/todos.db".to_string());

        Self { port, database_url }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn unset_var(key: &str) {
        // remove_var panics if key is empty, which we never provide
        env::remove_var(key);
    }

    #[test]
    fn loads_defaults_when_env_not_set() {
        // Ensure clean slate for variables we care about
        unset_var("PORT");
        unset_var("DATABASE_URL");

        let cfg = AppConfig::from_env();
        assert_eq!(cfg.port, 8080);
        assert_eq!(cfg.database_url, "sqlite:data/todos.db");
    }

    #[test]
    fn parses_port_from_env() {
        unset_var("PORT");
        env::set_var("PORT", "9090");
        let cfg = AppConfig::from_env();
        assert_eq!(cfg.port, 9090);
    }

    #[test]
    fn invalid_port_falls_back_to_default() {
        env::set_var("PORT", "not-a-number");
        let cfg = AppConfig::from_env();
        assert_eq!(cfg.port, 8080);
    }

    #[test]
    fn sets_database_url_from_env() {
        env::set_var("DATABASE_URL", "sqlite::memory:?cache=shared");
        let cfg = AppConfig::from_env();
        assert_eq!(cfg.database_url, "sqlite::memory:?cache=shared");
    }
}