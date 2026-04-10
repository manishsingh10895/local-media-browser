use std::{path::PathBuf, sync::Arc};

use tokio::sync::{Mutex, RwLock};

use crate::{
    config::FrontendMode,
    models::{HealthResponse, IndexJob, IndexPhase, MediaIndex},
};

#[derive(Clone)]
pub struct AppState {
    pub media_root: Arc<PathBuf>,
    pub thumbnail_root: Arc<PathBuf>,
    pub index: Arc<RwLock<MediaIndex>>,
    pub status: Arc<RwLock<IndexStatusState>>,
    pub index_run_lock: Arc<Mutex<()>>,
    pub frontend_mode: FrontendMode,
}

#[derive(Clone)]
pub struct IndexStatusState {
    pub phase: IndexPhase,
    pub current_job: Option<IndexJob>,
    pub indexed_media_count: usize,
    pub last_completed_ms: Option<u64>,
    pub last_error: Option<String>,
    pub started_at_ms: Option<u64>,
}

impl Default for IndexStatusState {
    fn default() -> Self {
        Self {
            phase: IndexPhase::Starting,
            current_job: None,
            indexed_media_count: 0,
            last_completed_ms: None,
            last_error: None,
            started_at_ms: None,
        }
    }
}

impl AppState {
    pub async fn health_response(&self) -> HealthResponse {
        let status = self.status.read().await;
        status.as_response()
    }

    pub async fn initial_index_ready(&self) -> bool {
        self.status.read().await.has_completed_initial_index()
    }

    pub async fn current_status(&self) -> HealthResponse {
        self.status.read().await.as_response()
    }
}

impl IndexStatusState {
    pub fn mark_indexing(&mut self, job: IndexJob, started_at_ms: u64) {
        self.phase = IndexPhase::Indexing;
        self.current_job = Some(job);
        self.started_at_ms = Some(started_at_ms);
        self.last_error = None;
    }

    pub fn mark_ready(&mut self, indexed_media_count: usize, completed_at_ms: u64) {
        self.phase = IndexPhase::Ready;
        self.current_job = None;
        self.indexed_media_count = indexed_media_count;
        self.last_completed_ms = Some(completed_at_ms);
        self.last_error = None;
        self.started_at_ms = None;
    }

    pub fn mark_error(&mut self, job: IndexJob, error: String) {
        self.phase = IndexPhase::Error;
        self.current_job = Some(job);
        self.last_error = Some(error);
        self.started_at_ms = None;
    }

    pub fn has_completed_initial_index(&self) -> bool {
        self.last_completed_ms.is_some() || matches!(self.phase, IndexPhase::Ready)
    }

    pub fn as_response(&self) -> HealthResponse {
        HealthResponse {
            status: match self.phase {
                IndexPhase::Error => "error",
                IndexPhase::Ready => "ready",
                IndexPhase::Starting | IndexPhase::Indexing => "indexing",
            },
            phase: self.phase,
            is_ready: self.has_completed_initial_index(),
            is_indexing: matches!(self.phase, IndexPhase::Indexing),
            current_job: self.current_job,
            indexed_media_count: self.indexed_media_count,
            last_completed_ms: self.last_completed_ms,
            last_error: self.last_error.clone(),
            started_at_ms: self.started_at_ms,
        }
    }
}
