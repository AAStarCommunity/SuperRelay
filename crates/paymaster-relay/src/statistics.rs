// crates/paymaster-relay/src/statistics.rs
// This file will contain the implementation for the advanced statistics collection service.

use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
    time::Instant,
};

// A single recorded request
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct RequestRecord {
    method: String,
    status_code: u16,
    response_time: u128, // in milliseconds
}

// Statistics for a single API method
#[derive(Debug, Clone, Default)]
struct MethodStats {
    total_calls: u64,
    successful_calls: u64,
    response_times: Vec<u128>, // list of response times in ms
}

impl MethodStats {
    fn error_rate(&self) -> f64 {
        if self.total_calls == 0 {
            0.0
        } else {
            (self.total_calls - self.successful_calls) as f64 / self.total_calls as f64
        }
    }

    fn avg_response_time(&self) -> f64 {
        if self.response_times.is_empty() {
            0.0
        } else {
            self.response_times.iter().sum::<u128>() as f64 / self.response_times.len() as f64
        }
    }
}

// The main statistics service
#[derive(Debug, Clone)]
pub struct StatisticsService {
    requests: Arc<RwLock<Vec<RequestRecord>>>,
    by_method: Arc<RwLock<BTreeMap<String, MethodStats>>>,
    #[allow(dead_code)]
    start_time: Instant,
}

impl Default for StatisticsService {
    fn default() -> Self {
        Self {
            requests: Arc::new(RwLock::new(Vec::new())),
            by_method: Arc::new(RwLock::new(BTreeMap::new())),
            start_time: Instant::now(),
        }
    }
}

// This will be the structure returned by the aggregation method.
pub struct AggregatedStats {
    pub total_calls: u64,
    pub calls_by_method: BTreeMap<String, u64>,
    pub response_times: BTreeMap<String, f64>, // For now, just avg
    pub error_rates: BTreeMap<String, f64>,
}

impl StatisticsService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&self, method: &str, status_code: u16, response_time: u128) {
        let mut by_method = self.by_method.write().unwrap();
        let method_stats = by_method.entry(method.to_string()).or_default();

        method_stats.total_calls += 1;
        if (200..300).contains(&status_code) {
            method_stats.successful_calls += 1;
        }
        method_stats.response_times.push(response_time);

        // Also keep a log of individual requests for potential future analysis
        // Note: In a production system, this should be capped to prevent memory exhaustion.
        self.requests.write().unwrap().push(RequestRecord {
            method: method.to_string(),
            status_code,
            response_time,
        });
    }

    pub fn aggregate(&self) -> AggregatedStats {
        let by_method = self.by_method.read().unwrap();

        let total_calls = by_method.values().map(|stats| stats.total_calls).sum();

        let calls_by_method = by_method
            .iter()
            .map(|(method, stats)| (method.clone(), stats.total_calls))
            .collect();

        let response_times = by_method
            .iter()
            .map(|(method, stats)| (method.clone(), stats.avg_response_time()))
            .collect();

        let mut error_rates: BTreeMap<String, f64> = by_method
            .iter()
            .map(|(method, stats)| (method.clone(), stats.error_rate()))
            .collect();

        let total_successful: u64 = by_method.values().map(|s| s.successful_calls).sum();
        let overall_error_rate = if total_calls == 0 {
            0.0
        } else {
            (total_calls - total_successful) as f64 / total_calls as f64
        };
        error_rates.insert("overall".to_string(), overall_error_rate);

        AggregatedStats {
            total_calls,
            calls_by_method,
            response_times,
            error_rates,
        }
    }
}
