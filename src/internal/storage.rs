//! Event storage implementations for the FunnelMob SDK.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::FunnelMobError;
use crate::internal::event::Event;

/// Trait for event storage implementations.
pub trait EventStorage: Send + Sync {
    /// Appends events to storage.
    fn append(&self, events: &[Event]) -> Result<(), FunnelMobError>;

    /// Reads all events from storage.
    fn read_all(&self) -> Result<Vec<Event>, FunnelMobError>;

    /// Removes events from storage by their IDs.
    fn remove(&self, event_ids: &[uuid::Uuid]) -> Result<(), FunnelMobError>;

    /// Clears all events from storage.
    fn clear(&self) -> Result<(), FunnelMobError>;

    /// Returns the number of stored events.
    fn count(&self) -> Result<usize, FunnelMobError>;
}

/// In-memory storage for testing purposes.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    events: Mutex<Vec<Event>>,
}

impl MemoryStorage {
    /// Creates a new in-memory storage.
    pub fn new() -> Self {
        Self::default()
    }
}

impl EventStorage for MemoryStorage {
    fn append(&self, events: &[Event]) -> Result<(), FunnelMobError> {
        let mut storage = self.events.lock().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock storage: {}", e))
        })?;
        storage.extend(events.iter().cloned());
        Ok(())
    }

    fn read_all(&self) -> Result<Vec<Event>, FunnelMobError> {
        let storage = self.events.lock().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock storage: {}", e))
        })?;
        Ok(storage.clone())
    }

    fn remove(&self, event_ids: &[uuid::Uuid]) -> Result<(), FunnelMobError> {
        let mut storage = self.events.lock().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock storage: {}", e))
        })?;
        storage.retain(|e| !event_ids.contains(&e.event_id));
        Ok(())
    }

    fn clear(&self) -> Result<(), FunnelMobError> {
        let mut storage = self.events.lock().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock storage: {}", e))
        })?;
        storage.clear();
        Ok(())
    }

    fn count(&self) -> Result<usize, FunnelMobError> {
        let storage = self.events.lock().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock storage: {}", e))
        })?;
        Ok(storage.len())
    }
}

/// File-based storage using JSON Lines format.
///
/// Events are stored one per line in a JSON file, allowing for efficient
/// append operations and crash recovery.
pub struct FileStorage {
    path: PathBuf,
    mutex: Mutex<()>,
}

impl FileStorage {
    /// Creates a new file storage at the given path.
    pub fn new(path: PathBuf) -> Result<Self, FunnelMobError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                FunnelMobError::Configuration(format!(
                    "Failed to create storage directory: {}",
                    e
                ))
            })?;
        }

        Ok(Self {
            path,
            mutex: Mutex::new(()),
        })
    }

    /// Creates a file storage in the default data directory.
    ///
    /// On Linux: `~/.local/share/funnelmob/events.jsonl`
    /// On macOS: `~/Library/Application Support/funnelmob/events.jsonl`
    /// On Windows: `%APPDATA%\funnelmob\events.jsonl`
    pub fn default_path(app_id: &str) -> Result<PathBuf, FunnelMobError> {
        let data_dir = dirs::data_dir().ok_or_else(|| {
            FunnelMobError::Configuration("Could not determine data directory".to_string())
        })?;

        // Sanitize app_id for use in path
        let safe_app_id: String = app_id
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '.' || c == '_' { c } else { '_' })
            .collect();

        Ok(data_dir.join("funnelmob").join(safe_app_id).join("events.jsonl"))
    }
}

impl EventStorage for FileStorage {
    fn append(&self, events: &[Event]) -> Result<(), FunnelMobError> {
        let _lock = self.mutex.lock().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock storage: {}", e))
        })?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| {
                FunnelMobError::Configuration(format!("Failed to open storage file: {}", e))
            })?;

        for event in events {
            let json = serde_json::to_string(event).map_err(|e| {
                FunnelMobError::Configuration(format!("Failed to serialize event: {}", e))
            })?;
            writeln!(file, "{}", json).map_err(|e| {
                FunnelMobError::Configuration(format!("Failed to write event: {}", e))
            })?;
        }

        file.flush().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to flush storage file: {}", e))
        })?;

        Ok(())
    }

    fn read_all(&self) -> Result<Vec<Event>, FunnelMobError> {
        let _lock = self.mutex.lock().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock storage: {}", e))
        })?;

        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path).map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to open storage file: {}", e))
        })?;

        let reader = BufReader::new(file);
        let mut events = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| {
                FunnelMobError::Configuration(format!("Failed to read line: {}", e))
            })?;

            if line.trim().is_empty() {
                continue;
            }

            let event: Event = serde_json::from_str(&line).map_err(|e| {
                FunnelMobError::Configuration(format!("Failed to parse event: {}", e))
            })?;
            events.push(event);
        }

        Ok(events)
    }

    fn remove(&self, event_ids: &[uuid::Uuid]) -> Result<(), FunnelMobError> {
        let _lock = self.mutex.lock().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock storage: {}", e))
        })?;

        if !self.path.exists() {
            return Ok(());
        }

        // Read all events, filter, and rewrite
        let events: Vec<Event> = {
            let file = File::open(&self.path).map_err(|e| {
                FunnelMobError::Configuration(format!("Failed to open storage file: {}", e))
            })?;
            let reader = BufReader::new(file);
            let mut events = Vec::new();

            for line in reader.lines() {
                let line = line.map_err(|e| {
                    FunnelMobError::Configuration(format!("Failed to read line: {}", e))
                })?;

                if line.trim().is_empty() {
                    continue;
                }

                let event: Event = serde_json::from_str(&line).map_err(|e| {
                    FunnelMobError::Configuration(format!("Failed to parse event: {}", e))
                })?;

                if !event_ids.contains(&event.event_id) {
                    events.push(event);
                }
            }
            events
        };

        // Rewrite file with remaining events
        let mut file = File::create(&self.path).map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to create storage file: {}", e))
        })?;

        for event in &events {
            let json = serde_json::to_string(event).map_err(|e| {
                FunnelMobError::Configuration(format!("Failed to serialize event: {}", e))
            })?;
            writeln!(file, "{}", json).map_err(|e| {
                FunnelMobError::Configuration(format!("Failed to write event: {}", e))
            })?;
        }

        Ok(())
    }

    fn clear(&self) -> Result<(), FunnelMobError> {
        let _lock = self.mutex.lock().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock storage: {}", e))
        })?;

        if self.path.exists() {
            File::create(&self.path).map_err(|e| {
                FunnelMobError::Configuration(format!("Failed to clear storage file: {}", e))
            })?;
        }

        Ok(())
    }

    fn count(&self) -> Result<usize, FunnelMobError> {
        let _lock = self.mutex.lock().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock storage: {}", e))
        })?;

        if !self.path.exists() {
            return Ok(0);
        }

        let file = File::open(&self.path).map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to open storage file: {}", e))
        })?;

        let reader = BufReader::new(file);
        let count = reader
            .lines()
            .map_while(Result::ok)
            .filter(|line| !line.trim().is_empty())
            .count();

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_memory_storage_append_and_read() {
        let storage = MemoryStorage::new();

        let events = vec![Event::new("test1"), Event::new("test2")];
        storage.append(&events).unwrap();

        let stored = storage.read_all().unwrap();
        assert_eq!(stored.len(), 2);
        assert_eq!(stored[0].event_name, "test1");
        assert_eq!(stored[1].event_name, "test2");
    }

    #[test]
    fn test_memory_storage_remove() {
        let storage = MemoryStorage::new();

        let events = vec![Event::new("test1"), Event::new("test2"), Event::new("test3")];
        let id_to_remove = events[1].event_id;
        storage.append(&events).unwrap();

        storage.remove(&[id_to_remove]).unwrap();

        let stored = storage.read_all().unwrap();
        assert_eq!(stored.len(), 2);
        assert!(stored.iter().all(|e| e.event_id != id_to_remove));
    }

    #[test]
    fn test_memory_storage_clear() {
        let storage = MemoryStorage::new();

        let events = vec![Event::new("test1"), Event::new("test2")];
        storage.append(&events).unwrap();
        assert_eq!(storage.count().unwrap(), 2);

        storage.clear().unwrap();
        assert_eq!(storage.count().unwrap(), 0);
    }

    #[test]
    fn test_file_storage_append_and_read() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("events.jsonl");
        let storage = FileStorage::new(path).unwrap();

        let events = vec![Event::new("test1"), Event::new("test2")];
        storage.append(&events).unwrap();

        let stored = storage.read_all().unwrap();
        assert_eq!(stored.len(), 2);
        assert_eq!(stored[0].event_name, "test1");
        assert_eq!(stored[1].event_name, "test2");
    }

    #[test]
    fn test_file_storage_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("events.jsonl");

        // Write with one instance
        {
            let storage = FileStorage::new(path.clone()).unwrap();
            let events = vec![Event::new("persistent")];
            storage.append(&events).unwrap();
        }

        // Read with new instance
        {
            let storage = FileStorage::new(path).unwrap();
            let stored = storage.read_all().unwrap();
            assert_eq!(stored.len(), 1);
            assert_eq!(stored[0].event_name, "persistent");
        }
    }

    #[test]
    fn test_file_storage_remove() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("events.jsonl");
        let storage = FileStorage::new(path).unwrap();

        let events = vec![Event::new("test1"), Event::new("test2"), Event::new("test3")];
        let id_to_remove = events[1].event_id;
        storage.append(&events).unwrap();

        storage.remove(&[id_to_remove]).unwrap();

        let stored = storage.read_all().unwrap();
        assert_eq!(stored.len(), 2);
        assert!(stored.iter().all(|e| e.event_id != id_to_remove));
    }

    #[test]
    fn test_file_storage_clear() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("events.jsonl");
        let storage = FileStorage::new(path).unwrap();

        let events = vec![Event::new("test1"), Event::new("test2")];
        storage.append(&events).unwrap();
        assert_eq!(storage.count().unwrap(), 2);

        storage.clear().unwrap();
        assert_eq!(storage.count().unwrap(), 0);
    }

    #[test]
    fn test_file_storage_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("events.jsonl");
        let storage = FileStorage::new(path).unwrap();

        // Reading from non-existent file should return empty vec
        let stored = storage.read_all().unwrap();
        assert!(stored.is_empty());
        assert_eq!(storage.count().unwrap(), 0);
    }

    #[test]
    fn test_default_path() {
        let path = FileStorage::default_path("com.example.app").unwrap();
        assert!(path.to_string_lossy().contains("funnelmob"));
        assert!(path.to_string_lossy().contains("com.example.app"));
    }
}
