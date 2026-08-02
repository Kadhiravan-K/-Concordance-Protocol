use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolRecord {
    pub id: String,
    pub payload: Vec<u8>,
}

/// In-memory protocol recorder used for replay and deterministic debugging.
/// Real deployments should persist records to durable storage.
#[derive(Default, Clone)]
pub struct ProtocolRecorder {
    inner: Arc<Mutex<Vec<ProtocolRecord>>>,
}

impl ProtocolRecorder {
    pub fn new() -> Self { Self { inner: Arc::new(Mutex::new(Vec::new())) } }

    pub fn record(&self, rec: ProtocolRecord) {
        let mut guard = self.inner.lock().unwrap();
        guard.push(rec);
    }

    pub fn dump(&self) -> Vec<ProtocolRecord> {
        let guard = self.inner.lock().unwrap();
        guard.clone()
    }
}
