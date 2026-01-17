use crate::ports::FeedbackPort;
use async_trait::async_trait;

pub struct MockFeedbackAdapter;

impl MockFeedbackAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FeedbackPort for MockFeedbackAdapter {
    async fn create_issue(
        &self,
        _title: &str,
        _body: &str,
    ) -> anyhow::Result<Option<String>> {
        Ok(None)
    }
}
