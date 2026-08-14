use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzePaperRequest {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarySections {
    pub background: String,
    pub issue: String,
    pub cause: String,
    pub proposal: String,
    pub result: String,
    pub discussion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryEvidence {
    pub background: Vec<String>,
    pub issue: Vec<String>,
    pub cause: Vec<String>,
    pub proposal: Vec<String>,
    pub result: Vec<String>,
    pub discussion: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperSummaryResponse {
    pub paper_id: String,
    pub title: String,
    pub source_url: String,
    pub summary: SummarySections,
    pub evidence: SummaryEvidence,
}
