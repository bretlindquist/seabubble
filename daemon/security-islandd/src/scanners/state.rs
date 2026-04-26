use std::collections::HashSet;
use std::sync::{Arc, RwLock};

#[derive(Debug, Default, Clone)]
pub struct StateTracker {
    pub downloaded_files: Arc<RwLock<HashSet<String>>>,
    pub executed_scripts: Arc<RwLock<HashSet<String>>>,
}

impl StateTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_download(&self, path: String) {
        if let Ok(mut dl) = self.downloaded_files.write() {
            dl.insert(path);
        }
    }

    pub fn is_downloaded(&self, path: &str) -> bool {
        if let Ok(dl) = self.downloaded_files.read() {
            dl.contains(path)
        } else {
            false
        }
    }

    pub fn record_execution(&self, path: String) {
        if let Ok(mut ex) = self.executed_scripts.write() {
            ex.insert(path);
        }
    }
}
