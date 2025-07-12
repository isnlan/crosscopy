//! Performance metrics collection

use crate::utils::{Result, UtilError};
use log::debug;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Performance metrics collector
pub struct PerformanceMetrics {
    timers: Arc<RwLock<HashMap<String, Instant>>>,
    counters: Arc<RwLock<HashMap<String, u64>>>,
    durations: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
    gauges: Arc<RwLock<HashMap<String, f64>>>,
}

impl PerformanceMetrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            timers: Arc::new(RwLock::new(HashMap::new())),
            counters: Arc::new(RwLock::new(HashMap::new())),
            durations: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a timer for a named operation
    pub async fn start_timer(&self, name: &str) {
        let mut timers = self.timers.write().await;
        timers.insert(name.to_string(), Instant::now());
        debug!("Started timer: {}", name);
    }

    /// End a timer and record the duration
    pub async fn end_timer(&self, name: &str) -> Option<Duration> {
        let start_time = {
            let mut timers = self.timers.write().await;
            timers.remove(name)
        };

        if let Some(start_time) = start_time {
            let duration = start_time.elapsed();
            
            {
                let mut durations = self.durations.write().await;
                durations
                    .entry(name.to_string())
                    .or_insert_with(Vec::new)
                    .push(duration);
            }

            debug!("Ended timer: {} ({}ms)", name, duration.as_millis());
            Some(duration)
        } else {
            None
        }
    }

    /// Increment a counter
    pub async fn increment_counter(&self, name: &str) {
        let mut counters = self.counters.write().await;
        *counters.entry(name.to_string()).or_insert(0) += 1;
        debug!("Incremented counter: {}", name);
    }

    /// Add to a counter
    pub async fn add_to_counter(&self, name: &str, value: u64) {
        let mut counters = self.counters.write().await;
        *counters.entry(name.to_string()).or_insert(0) += value;
        debug!("Added {} to counter: {}", value, name);
    }

    /// Set a gauge value
    pub async fn set_gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.write().await;
        gauges.insert(name.to_string(), value);
        debug!("Set gauge: {} = {}", name, value);
    }

    /// Get counter value
    pub async fn get_counter(&self, name: &str) -> u64 {
        let counters = self.counters.read().await;
        counters.get(name).copied().unwrap_or(0)
    }

    /// Get gauge value
    pub async fn get_gauge(&self, name: &str) -> Option<f64> {
        let gauges = self.gauges.read().await;
        gauges.get(name).copied()
    }

    /// Get average duration for a timer
    pub async fn get_average_duration(&self, name: &str) -> Option<Duration> {
        let durations = self.durations.read().await;
        durations.get(name).and_then(|durations| {
            if durations.is_empty() {
                None
            } else {
                let total: Duration = durations.iter().sum();
                Some(total / durations.len() as u32)
            }
        })
    }

    /// Get all metrics as a summary
    pub async fn get_summary(&self) -> MetricsSummary {
        let counters = self.counters.read().await.clone();
        let gauges = self.gauges.read().await.clone();
        
        let mut duration_stats = HashMap::new();
        {
            let durations = self.durations.read().await;
            for (name, durations_vec) in durations.iter() {
                if !durations_vec.is_empty() {
                    let total: Duration = durations_vec.iter().sum();
                    let avg = total / durations_vec.len() as u32;
                    let min = *durations_vec.iter().min().unwrap();
                    let max = *durations_vec.iter().max().unwrap();
                    
                    duration_stats.insert(name.clone(), DurationStats {
                        count: durations_vec.len(),
                        average: avg,
                        min,
                        max,
                        total,
                    });
                }
            }
        }

        MetricsSummary {
            counters,
            gauges,
            duration_stats,
        }
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        let mut timers = self.timers.write().await;
        let mut counters = self.counters.write().await;
        let mut durations = self.durations.write().await;
        let mut gauges = self.gauges.write().await;

        timers.clear();
        counters.clear();
        durations.clear();
        gauges.clear();

        debug!("Reset all metrics");
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Duration statistics
#[derive(Debug, Clone)]
pub struct DurationStats {
    pub count: usize,
    pub average: Duration,
    pub min: Duration,
    pub max: Duration,
    pub total: Duration,
}

/// Metrics summary
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub duration_stats: HashMap<String, DurationStats>,
}

/// Timer guard that automatically ends the timer when dropped
pub struct TimerGuard {
    name: String,
    metrics: Arc<PerformanceMetrics>,
}

impl TimerGuard {
    pub fn new(name: String, metrics: Arc<PerformanceMetrics>) -> Self {
        Self { name, metrics }
    }
}

impl Drop for TimerGuard {
    fn drop(&mut self) {
        let metrics = self.metrics.clone();
        let name = self.name.clone();
        
        tokio::spawn(async move {
            metrics.end_timer(&name).await;
        });
    }
}

impl PerformanceMetrics {
    /// Create a timer guard that automatically ends when dropped
    pub async fn timer_guard(&self, name: &str) -> TimerGuard {
        self.start_timer(name).await;
        TimerGuard::new(name.to_string(), Arc::new(self.clone()))
    }
}

// We need to implement Clone for PerformanceMetrics to use it in TimerGuard
impl Clone for PerformanceMetrics {
    fn clone(&self) -> Self {
        Self {
            timers: self.timers.clone(),
            counters: self.counters.clone(),
            durations: self.durations.clone(),
            gauges: self.gauges.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_counter_operations() {
        let metrics = PerformanceMetrics::new();
        
        metrics.increment_counter("test_counter").await;
        metrics.add_to_counter("test_counter", 5).await;
        
        let value = metrics.get_counter("test_counter").await;
        assert_eq!(value, 6);
    }

    #[tokio::test]
    async fn test_gauge_operations() {
        let metrics = PerformanceMetrics::new();
        
        metrics.set_gauge("test_gauge", 42.5).await;
        
        let value = metrics.get_gauge("test_gauge").await;
        assert_eq!(value, Some(42.5));
    }

    #[tokio::test]
    async fn test_timer_operations() {
        let metrics = PerformanceMetrics::new();
        
        metrics.start_timer("test_timer").await;
        sleep(Duration::from_millis(10)).await;
        let duration = metrics.end_timer("test_timer").await;
        
        assert!(duration.is_some());
        assert!(duration.unwrap() >= Duration::from_millis(10));
        
        let avg = metrics.get_average_duration("test_timer").await;
        assert!(avg.is_some());
    }

    #[tokio::test]
    async fn test_metrics_summary() {
        let metrics = PerformanceMetrics::new();
        
        metrics.increment_counter("counter1").await;
        metrics.set_gauge("gauge1", 100.0).await;
        metrics.start_timer("timer1").await;
        sleep(Duration::from_millis(5)).await;
        metrics.end_timer("timer1").await;
        
        let summary = metrics.get_summary().await;
        
        assert_eq!(summary.counters.get("counter1"), Some(&1));
        assert_eq!(summary.gauges.get("gauge1"), Some(&100.0));
        assert!(summary.duration_stats.contains_key("timer1"));
    }
}
