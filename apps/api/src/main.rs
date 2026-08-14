mod app;
mod config;
mod domain;
mod errors;
mod ports;
mod repository;
mod services;

use app::{create_app, AppState};
use config::Settings;
use repository::InMemoryPaperRepository;
use services::{OpenAiPaperSummarizer, PaperAnalysisService, UrlPaperSourceReader};

#[tokio::main]
async fn main() {
    let settings = Settings::load().expect("failed to load backend settings");

    let analysis_service = PaperAnalysisService::new(
        InMemoryPaperRepository::default(),
        UrlPaperSourceReader::new(),
        OpenAiPaperSummarizer::new(
            settings.openai_api_key.clone(),
            settings.openai_model.clone(),
        ),
    );

    let app = create_app(AppState::new(analysis_service));
    let addr = format!("0.0.0.0:{}", settings.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind the API port");

    println!("paperlens API listening on http://{}", addr);
    axum::serve(listener, app)
        .await
        .expect("paperlens API server failed");
}
