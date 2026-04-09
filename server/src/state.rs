use std::{path::PathBuf, sync::Arc};

use tokio::sync::RwLock;

use crate::{config::FrontendMode, models::MediaIndex};

#[derive(Clone)]
pub struct AppState {
    pub media_root: Arc<PathBuf>,
    pub thumbnail_root: Arc<PathBuf>,
    pub index: Arc<RwLock<MediaIndex>>,
    pub frontend_mode: FrontendMode,
}
