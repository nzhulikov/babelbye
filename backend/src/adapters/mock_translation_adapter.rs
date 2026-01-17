use crate::ports::TranslationPort;
use async_trait::async_trait;

pub struct MockTranslationAdapter;

impl MockTranslationAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TranslationPort for MockTranslationAdapter {
    async fn translate(&self, text: &str, target_locale: &str) -> anyhow::Result<String> {
        Ok(format!("[{}] {}", target_locale, text))
    }
}
