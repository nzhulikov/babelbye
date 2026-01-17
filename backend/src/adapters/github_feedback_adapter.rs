use crate::ports::FeedbackPort;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};

pub struct GithubFeedbackAdapter {
    repo: String,
    token: String,
}

impl GithubFeedbackAdapter {
    pub fn new(repo: String, token: String) -> Self {
        Self { repo, token }
    }
}

#[async_trait]
impl FeedbackPort for GithubFeedbackAdapter {
    async fn create_issue(
        &self,
        title: &str,
        body: &str,
    ) -> anyhow::Result<Option<String>> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.token))?,
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github+json"));
        headers.insert(USER_AGENT, HeaderValue::from_static("babelbye-feedback"));

        let client = reqwest::Client::new();
        let response = client
            .post(format!("https://api.github.com/repos/{}/issues", self.repo))
            .headers(headers)
            .json(&serde_json::json!({
                "title": title,
                "body": body
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let payload: serde_json::Value = response.json().await?;
        Ok(payload
            .get("html_url")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string()))
    }
}
