use std::{env, path::PathBuf};

use crate::errors::AppError;

#[derive(Debug, Clone)]
pub struct Settings {
    pub openai_api_key: String,
    pub openai_model: String,
    pub port: u16,
}

impl Settings {
    pub fn load() -> Result<Self, AppError> {
        for candidate in [
            PathBuf::from(".env"),
            PathBuf::from("../.env"),
            PathBuf::from("../../.env"),
        ] {
            if candidate.exists() {
                let _ = dotenvy::from_path(&candidate);
            }
        }

        let openai_api_key = env::var("OPENAI_API_KEY").map_err(|_| {
            AppError::Configuration(
                "OPENAI_API_KEY is required. Add it to the backend .env file.".to_string(),
            )
        })?;

        let openai_model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|err| AppError::Configuration(format!("PORT is invalid: {err}")))?;

        Ok(Self {
            openai_api_key,
            openai_model,
            port,
        })
    }
}
