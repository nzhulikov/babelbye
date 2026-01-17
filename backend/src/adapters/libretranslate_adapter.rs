use crate::ports::TranslationPort;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct LibreTranslateAdapter {
    base_url: String,
    api_key: Option<String>,
    client: Client,
}

impl LibreTranslateAdapter {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            base_url,
            api_key,
            client: Client::new(),
        }
    }

    fn translate_url(&self) -> String {
        let trimmed = self.base_url.trim_end_matches('/');
        if trimmed.ends_with("/translate") {
            trimmed.to_string()
        } else {
            format!("{}/translate", trimmed)
        }
    }
}

#[derive(Deserialize)]
struct TranslateResponse {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

#[async_trait]
impl TranslationPort for LibreTranslateAdapter {
    async fn translate(&self, text: &str, target_locale: &str) -> anyhow::Result<String> {
        let mut payload = serde_json::json!({
            "q": text,
            "source": "auto",
            "target": target_locale,
            "format": "text"
        });

        if let Some(api_key) = &self.api_key {
            if let Some(map) = payload.as_object_mut() {
                map.insert("api_key".to_string(), serde_json::Value::String(api_key.clone()));
            }
        }

        let response = self
            .client
            .post(self.translate_url())
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        let body: TranslateResponse = response.json().await?;
        Ok(body.translated_text)
    }
}
