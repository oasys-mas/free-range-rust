// core/src/logging.rs

use serde_json::Value;
use std::sync::mpsc::Sender;

/// Trait for non-blocking environment logging.
pub trait Logger: Send + Sync {
    /// Enqueue a log event (non-blocking).
    fn log_event(&self, event: Value);

    /// Flush and shut down the logger (blocking).
    fn shutdown(&self);
}

/// Example: CSV Logger implementation skeleton.
pub struct CsvLogger {
    sender: Sender<Value>,
    // ... background worker handle, file path, etc.
}

impl Logger for CsvLogger {
    fn log_event(&self, event: Value) {
        // Send event to background thread (non-blocking)
        let _ = self.sender.send(event);
    }

    fn shutdown(&self) {
        // Signal background thread to flush and exit
        // (implementation omitted)
    }
}
