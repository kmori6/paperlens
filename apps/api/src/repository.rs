use std::{collections::HashMap, sync::Mutex};

use crate::{domain::PaperSummaryResponse, errors::AppError, ports::PaperRepository};

#[derive(Default)]
pub struct InMemoryPaperRepository {
    items: Mutex<HashMap<String, PaperSummaryResponse>>,
}

impl PaperRepository for InMemoryPaperRepository {
    fn save(&self, summary: PaperSummaryResponse) -> Result<(), AppError> {
        let mut items = self
            .items
            .lock()
            .map_err(|err| AppError::Storage(format!("failed to lock repository: {err}")))?;

        items.insert(summary.paper_id.clone(), summary);
        Ok(())
    }

    fn list(&self) -> Result<Vec<PaperSummaryResponse>, AppError> {
        let items = self
            .items
            .lock()
            .map_err(|err| AppError::Storage(format!("failed to lock repository: {err}")))?;

        Ok(items.values().cloned().collect())
    }

    fn get(&self, paper_id: &str) -> Result<Option<PaperSummaryResponse>, AppError> {
        let items = self
            .items
            .lock()
            .map_err(|err| AppError::Storage(format!("failed to lock repository: {err}")))?;

        Ok(items.get(paper_id).cloned())
    }
}
