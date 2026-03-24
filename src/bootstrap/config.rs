use std::ops::Deref;

use crate::config::AppConfig;

#[derive(Debug, Clone)]
pub struct ValidatedConfig {
    environment: String,
    inner: AppConfig,
}

impl ValidatedConfig {
    pub fn load() -> anyhow::Result<Self> {
        AppConfig::init()?;
        Ok(Self {
            environment: AppConfig::current_environment(),
            inner: AppConfig::global().clone(),
        })
    }

    pub fn environment(&self) -> &str {
        &self.environment
    }

    pub fn app_config(&self) -> &AppConfig {
        &self.inner
    }
}

impl Deref for ValidatedConfig {
    type Target = AppConfig;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub fn init() -> anyhow::Result<ValidatedConfig> {
    ValidatedConfig::load()
}
