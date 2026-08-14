use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use url::Url;
use uuid::Uuid;

use crate::{
    domain::{AnalyzePaperRequest, PaperSummaryResponse, SummaryEvidence, SummarySections},
    errors::AppError,
    ports::{PaperRepository, PaperSourceReader, PaperSummarizer},
};

#[derive(Debug, Clone)]
pub struct UrlPaperSourceReader {
    client: Client,
}

impl UrlPaperSourceReader {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl PaperSourceReader for UrlPaperSourceReader {
    async fn fetch_text(&self, url: &str) -> Result<String, AppError> {
        let parsed_url = Url::parse(url)
            .map_err(|err| AppError::Validation(format!("invalid URL: {err}")))?;

        if !matches!(parsed_url.scheme(), "http" | "https") {
            return Err(AppError::Validation(
                "source URL must use http or https".to_string(),
            ));
        }

        let response = self
            .client
            .get(url)
            .header("User-Agent", "paperlens/1.0")
            .send()
            .await
            .map_err(|err| AppError::PaperSource(format!("failed to fetch URL: {err}")))?;

        if !response.status().is_success() {
            return Err(AppError::PaperSource(format!(
                "failed to fetch URL: HTTP {}",
                response.status()
            )));
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();

        let lower_url = url.to_ascii_lowercase();
        if lower_url.ends_with(".pdf") || content_type.contains("application/pdf") {
            let bytes = response
                .bytes()
                .await
                .map_err(|err| AppError::PaperSource(format!("failed to read PDF bytes: {err}")))?;

            let text = pdf_extract::extract_text_from_mem(bytes.as_ref()).map_err(|err| {
                AppError::PaperSource(format!("failed to extract paper text from PDF: {err}"))
            })?;

            if text.trim().is_empty() {
                return Err(AppError::PaperSource(
                    "PDF URL did not yield readable paper text.".to_string(),
                ));
            }

            return Ok(text);
        }

        let html = response
            .text()
            .await
            .map_err(|err| AppError::PaperSource(format!("failed to read body: {err}")))?;

        let document = Html::parse_document(&html);
        let selector = Selector::parse("body")
            .map_err(|err| AppError::Parse(format!("failed to configure HTML selector: {err}")))?;

        let text = document
            .select(&selector)
            .next()
            .map(|node| node.text().collect::<Vec<_>>().join(" "))
            .unwrap_or_default();

        if text.trim().is_empty() {
            return Err(AppError::PaperSource(
                "URL did not yield readable paper text.".to_string(),
            ));
        }

        Ok(text)
    }
}

#[derive(Debug, Clone)]
pub struct OpenAiPaperSummarizer {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAiPaperSummarizer {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }
}

#[async_trait]
impl PaperSummarizer for OpenAiPaperSummarizer {
    async fn summarize(&self, source_url: &str, text: &str) -> Result<PaperSummaryResponse, AppError> {
        let prompt = format!(
            r#"
You are summarizing a research paper.

Return valid JSON only with the following shape:
{{
  "title": "paper title",
  "summary": {{
    "background": "...",
    "issue": "...",
    "cause": "...",
    "proposal": "...",
    "result": "...",
    "discussion": "..."
  }},
  "evidence": {{
    "background": ["..."],
    "issue": ["..."],
    "cause": ["..."],
    "proposal": ["..."],
    "result": ["..."],
    "discussion": ["..."]
  }}
}}

Rules:
- The JSON keys must remain exactly as shown above.
- The values for title, summary.* and evidence.* must be written in Japanese.
- The six summary fields must be in Japanese: background, issue, cause, proposal, result, and discussion.
- Each summary sentence should be concise, informative, and natural in Japanese.
- Each evidence array should contain 1 to 3 short Japanese phrases or quotations that support the corresponding summary.
- If the paper title is originally in English, translate it to natural Japanese for the title field.
- Do not output markdown fences, explanations, or English text in the values.
- Output valid JSON only.

Source URL: {source_url}

Paper text:
{text}
"#
        );

        let payload = serde_json::json!({
            "model": self.model,
            "input": [{
                "role": "user",
                "content": [{
                    "type": "input_text",
                    "text": prompt
                }]
            }],
            "text": {
                "format": {
                    "type": "json_object"
                }
            }
        });

        let response = self
            .client
            .post("https://api.openai.com/v1/responses")
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|err| AppError::Llm(format!("openai responses request failed: {err}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown OpenAI error".to_string());
            return Err(AppError::Llm(format!(
                "OpenAI API returned HTTP {status}: {message}"
            )));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|err| AppError::Llm(format!("reading OpenAI response failed: {err}")))?;

        let response_text = body
            .get("output")
            .and_then(Value::as_array)
            .and_then(|items| {
                items.iter().find_map(|item| {
                    item.get("content")
                        .and_then(Value::as_array)
                        .and_then(|content| {
                            content.iter().find_map(|entry| entry.get("text").and_then(Value::as_str))
                        })
                })
            })
            .or_else(|| body.get("output_text").and_then(Value::as_str))
            .or_else(|| body.get("content").and_then(Value::as_str))
            .ok_or_else(|| AppError::Parse("LLM response did not contain JSON output".to_string()))?;

        let parsed: Value = serde_json::from_str(response_text)
            .map_err(|err| AppError::Parse(format!("LLM returned invalid JSON: {err}")))?;

        let summary_obj = parsed
            .get("summary")
            .and_then(Value::as_object)
            .ok_or_else(|| AppError::Parse("LLM JSON missing summary object".to_string()))?;

        let evidence_obj = parsed
            .get("evidence")
            .and_then(Value::as_object)
            .ok_or_else(|| AppError::Parse("LLM JSON missing evidence object".to_string()))?;

        let summary = SummarySections {
            background: string_value(summary_obj.get("background")).unwrap_or_default(),
            issue: string_value(summary_obj.get("issue")).unwrap_or_default(),
            cause: string_value(summary_obj.get("cause")).unwrap_or_default(),
            proposal: string_value(summary_obj.get("proposal")).unwrap_or_default(),
            result: string_value(summary_obj.get("result")).unwrap_or_default(),
            discussion: string_value(summary_obj.get("discussion")).unwrap_or_default(),
        };

        let evidence = SummaryEvidence {
            background: string_array_value(evidence_obj.get("background")),
            issue: string_array_value(evidence_obj.get("issue")),
            cause: string_array_value(evidence_obj.get("cause")),
            proposal: string_array_value(evidence_obj.get("proposal")),
            result: string_array_value(evidence_obj.get("result")),
            discussion: string_array_value(evidence_obj.get("discussion")),
        };

        Ok(PaperSummaryResponse {
            paper_id: Uuid::new_v4().to_string(),
            title: string_value(parsed.get("title")).unwrap_or_else(|| "Untitled paper".to_string()),
            source_url: source_url.to_string(),
            summary,
            evidence,
        })
    }
}

fn string_value(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(value)) => Some(value.clone()),
        Some(Value::Number(value)) => Some(value.to_string()),
        Some(Value::Bool(value)) => Some(value.to_string()),
        Some(Value::Array(items)) => {
            let values = items
                .iter()
                .filter_map(|item| string_value(Some(item)))
                .collect::<Vec<_>>();
            if values.is_empty() {
                None
            } else {
                Some(values.join(" "))
            }
        }
        _ => None,
    }
}

fn string_array_value(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| string_value(Some(item)))
            .collect(),
        Some(other) => string_value(Some(other)).into_iter().collect(),
        None => Vec::new(),
    }
}

#[derive(Debug, Clone)]
pub struct PaperAnalysisService<R, S, SR>
where
    R: PaperRepository,
    S: PaperSummarizer,
    SR: PaperSourceReader,
{
    paper_repository: R,
    paper_source_reader: SR,
    paper_summarizer: S,
}

impl<R, S, SR> PaperAnalysisService<R, S, SR>
where
    R: PaperRepository,
    S: PaperSummarizer,
    SR: PaperSourceReader,
{
    pub fn new(
        paper_repository: R,
        paper_source_reader: SR,
        paper_summarizer: S,
    ) -> Self {
        Self {
            paper_repository,
            paper_source_reader,
            paper_summarizer,
        }
    }

    pub async fn analyze_url(&self, request: AnalyzePaperRequest) -> Result<PaperSummaryResponse, AppError> {
        let url = request.url.trim().to_string();
        if url.is_empty() {
            return Err(AppError::Validation("url must not be empty".to_string()));
        }

        let text = self.paper_source_reader.fetch_text(&url).await?;
        let mut summary = self.paper_summarizer.summarize(&url, &text).await?;

        summary.paper_id = Uuid::new_v4().to_string();
        summary.source_url = url.clone();

        self.paper_repository.save(summary.clone())?;
        Ok(summary)
    }

    pub fn list(&self) -> Result<Vec<PaperSummaryResponse>, AppError> {
        self.paper_repository.list()
    }

    pub fn get_paper(&self, paper_id: &str) -> Result<Option<PaperSummaryResponse>, AppError> {
        self.paper_repository.get(paper_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{SummaryEvidence, SummarySections},
        repository::InMemoryPaperRepository,
    };

    #[derive(Debug, Clone, Default)]
    struct StubSourceReader;

    #[async_trait]
    impl PaperSourceReader for StubSourceReader {
        async fn fetch_text(&self, _url: &str) -> Result<String, AppError> {
            Ok("This paper introduces a new method and demonstrates strong benefits for reasoning tasks.".to_string())
        }
    }

    #[derive(Debug, Clone, Default)]
    struct StubSummarizer;

    #[async_trait]
    impl PaperSummarizer for StubSummarizer {
        async fn summarize(&self, source_url: &str, _text: &str) -> Result<PaperSummaryResponse, AppError> {
            Ok(PaperSummaryResponse {
                paper_id: String::new(),
                title: "A test paper".to_string(),
                source_url: source_url.to_string(),
                summary: SummarySections {
                    background: "Background".to_string(),
                    issue: "Issue".to_string(),
                    cause: "Cause".to_string(),
                    proposal: "Proposal".to_string(),
                    result: "Result".to_string(),
                    discussion: "Discussion".to_string(),
                },
                evidence: SummaryEvidence {
                    background: vec!["Background evidence".to_string()],
                    issue: vec!["Issue evidence".to_string()],
                    cause: vec!["Cause evidence".to_string()],
                    proposal: vec!["Proposal evidence".to_string()],
                    result: vec!["Result evidence".to_string()],
                    discussion: vec!["Discussion evidence".to_string()],
                },
            })
        }
    }

    #[tokio::test]
    async fn service_analyzes_and_stores_summary() {
        let repo = InMemoryPaperRepository::default();
        let service = PaperAnalysisService::new(repo, StubSourceReader, StubSummarizer);

        let result = service
            .analyze_url(AnalyzePaperRequest {
                url: "https://example.com/paper.pdf".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.title, "A test paper");
        assert_eq!(result.source_url, "https://example.com/paper.pdf");
        assert!(!result.paper_id.is_empty());
        assert_eq!(service.list().unwrap().len(), 1);
    }
}
