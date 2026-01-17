use crate::ports::TranslationPort;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct OpenAiTranslationAdapter {
    base_url: String,
    api_key: String,
    model: String,
    client: Client,
}

impl OpenAiTranslationAdapter {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        Self {
            base_url,
            api_key,
            model,
            client: Client::new(),
        }
    }

    fn completions_url(&self) -> String {
        let trimmed = self.base_url.trim_end_matches('/');
        if trimmed.ends_with("/chat/completions") {
            trimmed.to_string()
        } else {
            format!("{}/chat/completions", trimmed)
        }
    }
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

#[async_trait]
impl TranslationPort for OpenAiTranslationAdapter {
    async fn translate(&self, text: &str, target_locale: &str) -> anyhow::Result<String> {
        let system_prompt = format!(
            "You are a translation engine. Translate the user's text into {target}. \
Return only the translated text without quotes or commentary.",
            target = target_locale
        );

        let payload = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": text}
            ],
            "temperature": 0.2
        });

        let response = self
            .client
            .post(self.completions_url())
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        let body: ChatResponse = response.json().await?;
        let translated = body
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow::anyhow!("empty_translation"))?;
        Ok(translated)
    }
}
