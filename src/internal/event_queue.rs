//! Thread-safe event queue for the FunnelMob SDK.

use std::sync::{Arc, RwLock};

use crate::error::FunnelMobError;
use crate::internal::event::Event;
use crate::internal::storage::EventStorage;

/// Thread-safe event queue with optional persistence.
///
/// The queue maintains events in memory for fast access and optionally
/// persists them to storage for crash recovery.
pub struct EventQueue {
    /// In-memory queue of events.
    events: RwLock<Vec<Event>>,

    /// Optional persistent storage.
    storage: Option<Arc<dyn EventStorage>>,

    /// Maximum number of events to keep in the queue.
    max_size: usize,
}

impl EventQueue {
    /// Creates a new event queue without persistence.
    pub fn new(max_size: usize) -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            storage: None,
            max_size,
        }
    }

    /// Creates a new event queue with persistent storage.
    ///
    /// Events will be loaded from storage on creation and persisted
    /// when added or removed.
    pub fn with_storage(max_size: usize, storage: Arc<dyn EventStorage>) -> Result<Self, FunnelMobError> {
        // Load existing events from storage
        let existing_events = storage.read_all()?;

        Ok(Self {
            events: RwLock::new(existing_events),
            storage: Some(storage),
            max_size,
        })
    }

    /// Adds an event to the queue.
    ///
    /// If the queue is at max capacity, the oldest events will be dropped.
    pub fn enqueue(&self, event: Event) -> Result<(), FunnelMobError> {
        self.enqueue_batch(vec![event])
    }

    /// Adds multiple events to the queue.
    ///
    /// If the queue would exceed max capacity, the oldest events will be dropped.
    pub fn enqueue_batch(&self, new_events: Vec<Event>) -> Result<(), FunnelMobError> {
        let mut events = self.events.write().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock queue: {}", e))
        })?;

        // Persist new events if storage is available
        if let Some(ref storage) = self.storage {
            storage.append(&new_events)?;
        }

        events.extend(new_events);

        // Trim to max size if needed (drop oldest)
        if events.len() > self.max_size {
            let to_remove = events.len() - self.max_size;
            let removed_ids: Vec<_> = events.drain(..to_remove).map(|e| e.event_id).collect();

            // Remove from storage too
            if let Some(ref storage) = self.storage {
                storage.remove(&removed_ids)?;
            }
        }

        Ok(())
    }

    /// Takes up to `count` events from the front of the queue.
    ///
    /// Events are removed from the queue but NOT from storage yet.
    /// Call `confirm_sent` after successfully sending to remove from storage.
    pub fn take(&self, count: usize) -> Result<Vec<Event>, FunnelMobError> {
        let mut events = self.events.write().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock queue: {}", e))
        })?;

        let take_count = count.min(events.len());
        let taken: Vec<Event> = events.drain(..take_count).collect();

        Ok(taken)
    }

    /// Confirms that events were successfully sent and removes them from storage.
    pub fn confirm_sent(&self, event_ids: &[uuid::Uuid]) -> Result<(), FunnelMobError> {
        if let Some(ref storage) = self.storage {
            storage.remove(event_ids)?;
        }
        Ok(())
    }

    /// Re-queues events that failed to send (puts them back at the front).
    pub fn requeue(&self, events: Vec<Event>) -> Result<(), FunnelMobError> {
        let mut queue = self.events.write().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock queue: {}", e))
        })?;

        // Put failed events back at the front
        let mut new_queue = events;
        new_queue.append(&mut *queue);
        *queue = new_queue;

        // Trim if needed (remove from the end since we put failed events at front)
        if queue.len() > self.max_size {
            let to_remove = queue.len() - self.max_size;
            let start_idx = queue.len() - to_remove;
            let removed_ids: Vec<_> = queue.drain(start_idx..).map(|e| e.event_id).collect();

            if let Some(ref storage) = self.storage {
                storage.remove(&removed_ids)?;
            }
        }

        Ok(())
    }

    /// Returns the number of events in the queue.
    #[allow(dead_code)]
    pub fn len(&self) -> Result<usize, FunnelMobError> {
        let events = self.events.read().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock queue: {}", e))
        })?;
        Ok(events.len())
    }

    /// Returns true if the queue is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> Result<bool, FunnelMobError> {
        Ok(self.len()? == 0)
    }

    /// Clears all events from the queue and storage.
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<(), FunnelMobError> {
        let mut events = self.events.write().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock queue: {}", e))
        })?;
        events.clear();

        if let Some(ref storage) = self.storage {
            storage.clear()?;
        }

        Ok(())
    }

    /// Peeks at up to `count` events without removing them.
    #[allow(dead_code)]
    pub fn peek(&self, count: usize) -> Result<Vec<Event>, FunnelMobError> {
        let events = self.events.read().map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to lock queue: {}", e))
        })?;

        let peek_count = count.min(events.len());
        Ok(events[..peek_count].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::storage::MemoryStorage;

    #[test]
    fn test_enqueue_and_take() {
        let queue = EventQueue::new(100);

        queue.enqueue(Event::new("test1")).unwrap();
        queue.enqueue(Event::new("test2")).unwrap();

        assert_eq!(queue.len().unwrap(), 2);

        let taken = queue.take(1).unwrap();
        assert_eq!(taken.len(), 1);
        assert_eq!(taken[0].event_name, "test1");

        assert_eq!(queue.len().unwrap(), 1);
    }

    #[test]
    fn test_enqueue_batch() {
        let queue = EventQueue::new(100);

        let events = vec![Event::new("test1"), Event::new("test2"), Event::new("test3")];
        queue.enqueue_batch(events).unwrap();

        assert_eq!(queue.len().unwrap(), 3);
    }

    #[test]
    fn test_max_size_enforcement() {
        let queue = EventQueue::new(3);

        for i in 0..5 {
            queue.enqueue(Event::new(format!("test{}", i))).unwrap();
        }

        // Should only have 3 events (oldest dropped)
        assert_eq!(queue.len().unwrap(), 3);

        // Should have the newest events
        let events = queue.take(3).unwrap();
        assert_eq!(events[0].event_name, "test2");
        assert_eq!(events[1].event_name, "test3");
        assert_eq!(events[2].event_name, "test4");
    }

    #[test]
    fn test_requeue() {
        let queue = EventQueue::new(100);

        queue.enqueue(Event::new("test1")).unwrap();
        queue.enqueue(Event::new("test2")).unwrap();

        // Take all events
        let taken = queue.take(2).unwrap();
        assert!(queue.is_empty().unwrap());

        // Requeue (simulating failed send)
        queue.requeue(taken).unwrap();

        // Should be back in the queue
        assert_eq!(queue.len().unwrap(), 2);
        let events = queue.peek(2).unwrap();
        assert_eq!(events[0].event_name, "test1");
    }

    #[test]
    fn test_peek() {
        let queue = EventQueue::new(100);

        queue.enqueue(Event::new("test1")).unwrap();
        queue.enqueue(Event::new("test2")).unwrap();

        let peeked = queue.peek(1).unwrap();
        assert_eq!(peeked.len(), 1);
        assert_eq!(peeked[0].event_name, "test1");

        // Queue should still have both events
        assert_eq!(queue.len().unwrap(), 2);
    }

    #[test]
    fn test_clear() {
        let queue = EventQueue::new(100);

        queue.enqueue(Event::new("test1")).unwrap();
        queue.enqueue(Event::new("test2")).unwrap();

        queue.clear().unwrap();
        assert!(queue.is_empty().unwrap());
    }

    #[test]
    fn test_with_storage() {
        let storage = Arc::new(MemoryStorage::new());
        let queue = EventQueue::with_storage(100, storage.clone()).unwrap();

        queue.enqueue(Event::new("test1")).unwrap();
        queue.enqueue(Event::new("test2")).unwrap();

        // Events should be in storage
        assert_eq!(storage.count().unwrap(), 2);
    }

    #[test]
    fn test_confirm_sent_removes_from_storage() {
        let storage = Arc::new(MemoryStorage::new());
        let queue = EventQueue::with_storage(100, storage.clone()).unwrap();

        queue.enqueue(Event::new("test1")).unwrap();
        queue.enqueue(Event::new("test2")).unwrap();

        let taken = queue.take(2).unwrap();
        let ids: Vec<_> = taken.iter().map(|e| e.event_id).collect();

        // Still in storage
        assert_eq!(storage.count().unwrap(), 2);

        // Confirm sent
        queue.confirm_sent(&ids).unwrap();

        // Now removed from storage
        assert_eq!(storage.count().unwrap(), 0);
    }

    #[test]
    fn test_storage_persistence() {
        let storage = Arc::new(MemoryStorage::new());

        // Add events with first queue
        {
            let queue = EventQueue::with_storage(100, storage.clone()).unwrap();
            queue.enqueue(Event::new("persistent")).unwrap();
        }

        // New queue should load from storage
        {
            let queue = EventQueue::with_storage(100, storage).unwrap();
            assert_eq!(queue.len().unwrap(), 1);
            let events = queue.peek(1).unwrap();
            assert_eq!(events[0].event_name, "persistent");
        }
    }

    #[test]
    fn test_take_more_than_available() {
        let queue = EventQueue::new(100);

        queue.enqueue(Event::new("test1")).unwrap();

        let taken = queue.take(10).unwrap();
        assert_eq!(taken.len(), 1);
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let queue = Arc::new(EventQueue::new(1000));
        let mut handles = vec![];

        // Spawn multiple threads to enqueue
        for i in 0..10 {
            let q = Arc::clone(&queue);
            handles.push(thread::spawn(move || {
                for j in 0..10 {
                    q.enqueue(Event::new(format!("thread{}_{}", i, j))).unwrap();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Should have all 100 events
        assert_eq!(queue.len().unwrap(), 100);
    }
}
