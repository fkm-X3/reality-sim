use std::ops::Range;

/// Batch scheduler for parallel agent processing
pub struct BatchScheduler {
    /// Number of worker threads
    pub thread_count: usize,
    /// Batch size for agent processing
    pub batch_size: usize,
}

impl BatchScheduler {
    /// Create a new scheduler
    pub fn new(thread_count: usize) -> Self {
        Self {
            thread_count,
            batch_size: 1000,
        }
    }

    /// Calculate batch ranges for parallel processing
    pub fn calculate_batches(&self, total: usize) -> Vec<Range<usize>> {
        let batch_count = (total + self.batch_size - 1) / self.batch_size;
        let batches_per_thread = (batch_count + self.thread_count - 1) / self.thread_count;

        let mut ranges = Vec::with_capacity(self.thread_count);
        let mut start = 0;

        for _ in 0..self.thread_count {
            let end = (start + batches_per_thread * self.batch_size).min(total);
            if start < end {
                ranges.push(start..end);
            }
            start = end;
        }

        ranges
    }

    /// Calculate optimal batch size based on agent count
    pub fn optimal_batch_size(&self, agent_count: usize) -> usize {
        // Target: ~4 batches per thread for good load balancing
        let target_batches = self.thread_count * 4;
        let batch_size = (agent_count + target_batches - 1) / target_batches;
        batch_size.max(100).min(10000) // Clamp to reasonable range
    }

    /// Update batch size for optimal performance
    pub fn auto_tune(&mut self, agent_count: usize) {
        self.batch_size = self.optimal_batch_size(agent_count);
    }
}

impl Default for BatchScheduler {
    fn default() -> Self {
        Self::new(rayon::current_num_threads())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_calculation() {
        let scheduler = BatchScheduler {
            thread_count: 4,
            batch_size: 100,
        };

        let batches = scheduler.calculate_batches(350);
        
        // Should have up to 4 batches
        assert!(batches.len() <= 4);
        
        // Total range should cover all items
        let total: usize = batches.iter().map(|r| r.len()).sum();
        assert_eq!(total, 350);
    }

    #[test]
    fn test_optimal_batch_size() {
        let scheduler = BatchScheduler::new(4);
        
        let batch_size = scheduler.optimal_batch_size(10000);
        assert!(batch_size >= 100);
        assert!(batch_size <= 10000);
    }
}
