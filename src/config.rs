use std::env;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub external_url: String,
}

impl Config {
    pub fn get() -> anyhow::Result<Self> {
        let database_url = req_env_var("DATABASE_URL")?;
        let port = env::var("SQLXUM_PORT").unwrap_or_else(|_| "8080".to_string());
        let port: u16 = port.parse()?;
        let external_url =
            env::var("EXTERNAL_URL").unwrap_or_else(|_| format!("http://localhost:{}", port));
        Ok(Self {
            database_url,
            port,
            external_url,
        })
    }
}

fn req_env_var(name: &str) -> anyhow::Result<String> {
    env::var(name).map_err(|_| anyhow::anyhow!("envvar '{}' not set", name))
}
