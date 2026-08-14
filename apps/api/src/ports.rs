use async_trait::async_trait;

use crate::{domain::PaperSummaryResponse, errors::AppError};

#[async_trait]
pub trait PaperSourceReader: Send + Sync {
    async fn fetch_text(&self, url: &str) -> Result<String, AppError>;
}

#[async_trait]
pub trait PaperSummarizer: Send + Sync {
    async fn summarize(&self, source_url: &str, text: &str) -> Result<PaperSummaryResponse, AppError>;
}

pub trait PaperRepository: Send + Sync {
    fn save(&self, summary: PaperSummaryResponse) -> Result<(), AppError>;
    fn list(&self) -> Result<Vec<PaperSummaryResponse>, AppError>;
    fn get(&self, paper_id: &str) -> Result<Option<PaperSummaryResponse>, AppError>;
}
