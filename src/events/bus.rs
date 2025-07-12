//! Event bus implementation

use crate::events::{Event, EventError, EventPriority, EventWithMetadata, Result};
use log::{debug, error, warn};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Duration, Instant};

/// Maximum number of events in the queue
const MAX_QUEUE_SIZE: usize = 1000;

/// Event bus for inter-module communication
pub struct EventBus {
    /// Event queue with priority ordering
    queue: Arc<Mutex<VecDeque<EventWithMetadata>>>,
    /// Event handlers
    handlers: Arc<RwLock<Vec<Box<dyn EventHandler + Send + Sync>>>>,
    /// Bus statistics
    stats: Arc<RwLock<EventBusStats>>,
}

/// Event handler trait
pub trait EventHandler {
    /// Handle an event
    fn handle(&self, event: &Event) -> Result<()>;
    
    /// Get handler name for debugging
    fn name(&self) -> &str;
}

/// Event bus statistics
#[derive(Debug, Default, Clone)]
pub struct EventBusStats {
    pub events_emitted: u64,
    pub events_processed: u64,
    pub events_dropped: u64,
    pub queue_size: usize,
    pub max_queue_size: usize,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            handlers: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(EventBusStats::default())),
        }
    }

    /// Emit an event to the bus
    pub async fn emit(&self, event: Event) -> Result<()> {
        self.emit_with_priority(event, EventPriority::Normal).await
    }

    /// Emit an event with specific priority
    pub async fn emit_with_priority(&self, event: Event, priority: EventPriority) -> Result<()> {
        let event_with_metadata = EventWithMetadata::new(event, "system".to_string())
            .with_priority(priority);

        let mut queue = self.queue.lock().await;
        
        // Check queue size limit
        if queue.len() >= MAX_QUEUE_SIZE {
            // Drop lowest priority event if queue is full
            if let Some(dropped_event) = self.drop_lowest_priority_event(&mut queue) {
                warn!("Event queue full, dropped event: {}", dropped_event.event);
                self.increment_dropped_count().await;
            } else {
                return Err(EventError::BusFull);
            }
        }

        // Insert event in priority order
        let insert_pos = queue
            .iter()
            .position(|e| e.priority < event_with_metadata.priority)
            .unwrap_or(queue.len());
        
        queue.insert(insert_pos, event_with_metadata);
        
        // Update statistics
        self.increment_emitted_count().await;
        self.update_queue_size(queue.len()).await;

        debug!("Event emitted, queue size: {}", queue.len());
        Ok(())
    }

    /// Poll for the next event
    pub async fn poll_event(&self) -> Option<Event> {
        let mut queue = self.queue.lock().await;
        
        if let Some(event_with_metadata) = queue.pop_front() {
            self.increment_processed_count().await;
            self.update_queue_size(queue.len()).await;
            
            debug!("Event polled: {}", event_with_metadata.event);
            Some(event_with_metadata.event)
        } else {
            None
        }
    }

    /// Register an event handler
    pub async fn register_handler(&self, handler: Box<dyn EventHandler + Send + Sync>) -> Result<()> {
        let mut handlers = self.handlers.write().await;
        debug!("Registering event handler: {}", handler.name());
        handlers.push(handler);
        Ok(())
    }

    /// Process events with registered handlers
    pub async fn process_events(&self) -> Result<()> {
        while let Some(event) = self.poll_event().await {
            let handlers = self.handlers.read().await;
            
            for handler in handlers.iter() {
                if let Err(e) = handler.handle(&event) {
                    error!("Handler '{}' failed to process event {}: {}", 
                           handler.name(), event, e);
                }
            }
        }
        
        Ok(())
    }

    /// Get current queue size
    pub async fn queue_size(&self) -> usize {
        self.queue.lock().await.len()
    }

    /// Get event bus statistics
    pub async fn get_stats(&self) -> EventBusStats {
        self.stats.read().await.clone()
    }

    /// Clear all events from the queue
    pub async fn clear(&self) -> usize {
        let mut queue = self.queue.lock().await;
        let cleared_count = queue.len();
        queue.clear();
        self.update_queue_size(0).await;
        cleared_count
    }

    /// Drop the lowest priority event from the queue
    fn drop_lowest_priority_event(&self, queue: &mut VecDeque<EventWithMetadata>) -> Option<EventWithMetadata> {
        if queue.is_empty() {
            return None;
        }

        // Find the event with the lowest priority
        let mut lowest_priority = EventPriority::Critical;
        let mut lowest_index = 0;

        for (index, event) in queue.iter().enumerate() {
            if event.priority <= lowest_priority {
                lowest_priority = event.priority;
                lowest_index = index;
            }
        }

        queue.remove(lowest_index)
    }

    async fn increment_emitted_count(&self) {
        let mut stats = self.stats.write().await;
        stats.events_emitted += 1;
    }

    async fn increment_processed_count(&self) {
        let mut stats = self.stats.write().await;
        stats.events_processed += 1;
    }

    async fn increment_dropped_count(&self) {
        let mut stats = self.stats.write().await;
        stats.events_dropped += 1;
    }

    async fn update_queue_size(&self, size: usize) {
        let mut stats = self.stats.write().await;
        stats.queue_size = size;
        if size > stats.max_queue_size {
            stats.max_queue_size = size;
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHandler {
        name: String,
    }

    impl EventHandler for TestHandler {
        fn handle(&self, _event: &Event) -> Result<()> {
            Ok(())
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_event_bus_creation() {
        let bus = EventBus::new();
        assert_eq!(bus.queue_size().await, 0);
    }

    #[tokio::test]
    async fn test_event_emission_and_polling() {
        let bus = EventBus::new();
        let event = Event::Shutdown;

        bus.emit(event.clone()).await.unwrap();
        assert_eq!(bus.queue_size().await, 1);

        let polled_event = bus.poll_event().await;
        assert!(polled_event.is_some());
        assert_eq!(bus.queue_size().await, 0);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let bus = EventBus::new();

        // Emit events with different priorities
        bus.emit_with_priority(Event::Shutdown, EventPriority::Low).await.unwrap();
        bus.emit_with_priority(Event::Error { error: "test".to_string() }, EventPriority::Critical).await.unwrap();
        bus.emit_with_priority(Event::Heartbeat { device_id: "test".to_string(), timestamp: 0 }, EventPriority::Normal).await.unwrap();

        // Events should be polled in priority order (Critical, Normal, Low)
        let first = bus.poll_event().await.unwrap();
        assert!(matches!(first, Event::Error { .. }));

        let second = bus.poll_event().await.unwrap();
        assert!(matches!(second, Event::Heartbeat { .. }));

        let third = bus.poll_event().await.unwrap();
        assert!(matches!(third, Event::Shutdown));
    }
}
