use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    domain::AnalyzePaperRequest,
    errors::AppError,
    repository::InMemoryPaperRepository,
    services::{OpenAiPaperSummarizer, PaperAnalysisService, UrlPaperSourceReader},
};

#[derive(Clone)]
pub struct AppState {
    pub analysis_service: Arc<
        PaperAnalysisService<InMemoryPaperRepository, OpenAiPaperSummarizer, UrlPaperSourceReader>,
    >,
}

impl AppState {
    pub fn new(
        analysis_service: PaperAnalysisService<InMemoryPaperRepository, OpenAiPaperSummarizer, UrlPaperSourceReader>,
    ) -> Self {
        Self {
            analysis_service: Arc::new(analysis_service),
        }
    }
}

pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_check))
        .route("/api/papers/analyze", post(analyze_paper))
        .route("/api/papers", get(list_papers))
        .route("/api/papers/{paper_id}", get(get_paper))
        .with_state(state)
        .layer(cors)
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

async fn list_papers(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let papers = state.analysis_service.list()?;
    Ok((StatusCode::OK, Json(papers)))
}

async fn analyze_paper(
    State(state): State<AppState>,
    Json(request): Json<AnalyzePaperRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = state.analysis_service.analyze_url(request).await?;
    Ok((StatusCode::OK, Json(response)))
}

async fn get_paper(State(state): State<AppState>, Path(paper_id): Path<String>) -> Result<impl IntoResponse, AppError> {
    match state.analysis_service.get_paper(&paper_id)? {
        Some(paper) => Ok((StatusCode::OK, Json(paper))),
        None => Err(AppError::Validation(format!("paper not found: {paper_id}"))),
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Method, Request},
    };
    use tower::ServiceExt;

    use super::*;
    use crate::{
        config::Settings,
        services::{OpenAiPaperSummarizer, PaperAnalysisService, UrlPaperSourceReader},
    };

    #[tokio::test]
    async fn analyze_route_rejects_empty_url() {
        let settings = Settings::load().unwrap_or(Settings {
            openai_api_key: "test-key".to_string(),
            openai_model: "gpt-5.6-luna".to_string(),
            port: 3000,
        });

        let app = create_app(AppState::new(PaperAnalysisService::new(
            InMemoryPaperRepository::default(),
            UrlPaperSourceReader::new(),
            OpenAiPaperSummarizer::new(settings.openai_api_key, settings.openai_model),
        )));

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/papers/analyze")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"url":""}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
