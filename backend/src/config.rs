use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub db_max_connections: u32,
    pub auth0_domain: String,
    pub auth0_audience: String,
    pub auth0_issuer: String,
    pub auth_bypass: bool,
    pub allowed_origins: String,
    pub openai_api_url: String,
    pub openai_api_key: Option<String>,
    pub openai_model: String,
    pub github_token: Option<String>,
    pub feedback_repo: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let default_domain = "dev.local";
        let default_audience = "https://babelbye.local";
        let default_issuer = "https://dev.local/";
        let mut settings = config::Config::builder()
            .add_source(config::Environment::default().separator("__"));
        settings = settings.set_default("db_max_connections", 5)?;
        settings = settings.set_default("auth_bypass", false)?;
        settings = settings.set_default("allowed_origins", "*")?;
        settings = settings.set_default("auth0_domain", default_domain)?;
        settings = settings.set_default("auth0_audience", default_audience)?;
        settings = settings.set_default("auth0_issuer", default_issuer)?;
        settings = settings.set_default("openai_api_url", "https://api.openai.com/v1")?;
        settings = settings.set_default("openai_model", "gpt-5.2")?;
        let config: Config = settings.build()?.try_deserialize()?;

        if !config.auth_bypass
            && (config.auth0_domain == default_domain
                || config.auth0_audience == default_audience
                || config.auth0_issuer == default_issuer)
        {
            return Err(config::ConfigError::Message(
                "Auth0 settings must be provided when AUTH_BYPASS=false".to_string(),
            ));
        }

        Ok(config)
    }
}
