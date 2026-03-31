use std::time::{Duration, Instant};

/// Time management for simulation
pub struct TimeManager {
    /// Target ticks per second
    pub target_tps: u32,
    /// Time per tick
    pub tick_duration: Duration,
    /// Last tick timestamp
    pub last_tick: Instant,
    /// Accumulated time for fixed timestep
    pub accumulator: Duration,
    /// Actual TPS (measured)
    pub actual_tps: f32,
    /// Total elapsed time
    pub total_time: Duration,
    /// Tick counter for TPS calculation
    tick_counter: u32,
    /// Last TPS measurement time
    last_tps_time: Instant,
}

impl TimeManager {
    /// Create a new time manager
    pub fn new(target_tps: u32) -> Self {
        Self {
            target_tps,
            tick_duration: Duration::from_secs_f64(1.0 / target_tps as f64),
            last_tick: Instant::now(),
            accumulator: Duration::ZERO,
            actual_tps: 0.0,
            total_time: Duration::ZERO,
            tick_counter: 0,
            last_tps_time: Instant::now(),
        }
    }

    /// Check if a tick should occur, accumulating time
    pub fn should_tick(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now - self.last_tick;
        self.last_tick = now;

        self.accumulator += elapsed;

        if self.accumulator >= self.tick_duration {
            self.accumulator -= self.tick_duration;
            self.total_time += self.tick_duration;
            self.tick_counter += 1;

            // Update TPS measurement every second
            let tps_elapsed = now - self.last_tps_time;
            if tps_elapsed >= Duration::from_secs(1) {
                self.actual_tps = self.tick_counter as f32 / tps_elapsed.as_secs_f32();
                self.tick_counter = 0;
                self.last_tps_time = now;
            }

            true
        } else {
            false
        }
    }

    /// Get delta time in seconds
    pub fn delta_time(&self) -> f32 {
        self.tick_duration.as_secs_f32()
    }

    /// Set target TPS
    pub fn set_target_tps(&mut self, tps: u32) {
        self.target_tps = tps;
        self.tick_duration = Duration::from_secs_f64(1.0 / tps as f64);
    }

    /// Get time until next tick (for sleeping)
    pub fn time_until_next_tick(&self) -> Duration {
        if self.accumulator >= self.tick_duration {
            Duration::ZERO
        } else {
            self.tick_duration - self.accumulator
        }
    }

    /// Reset the time manager
    pub fn reset(&mut self) {
        self.last_tick = Instant::now();
        self.accumulator = Duration::ZERO;
        self.total_time = Duration::ZERO;
        self.tick_counter = 0;
        self.last_tps_time = Instant::now();
    }
}

/// Convert simulation ticks to in-world time
pub struct SimulationTime {
    /// Ticks per in-game day
    pub ticks_per_day: u64,
    /// Starting year
    pub starting_year: i32,
}

impl SimulationTime {
    pub fn new(starting_year: i32, ticks_per_second: u32) -> Self {
        Self {
            // One in-game day = 10 real seconds at normal speed
            ticks_per_day: ticks_per_second as u64 * 10,
            starting_year,
        }
    }

    /// Convert ticks to (year, day)
    pub fn ticks_to_date(&self, ticks: u64) -> (i32, u32) {
        let total_days = ticks / self.ticks_per_day;
        let years_passed = (total_days / 365) as i32;
        let day_of_year = (total_days % 365) as u32;
        (self.starting_year + years_passed, day_of_year)
    }

    /// Get formatted date string
    pub fn format_date(&self, ticks: u64) -> String {
        let (year, day) = self.ticks_to_date(ticks);
        if year < 0 {
            format!("{} BCE, Day {}", -year, day + 1)
        } else {
            format!("{} CE, Day {}", year, day + 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_time_manager_creation() {
        let tm = TimeManager::new(60);
        assert_eq!(tm.target_tps, 60);
        assert!(tm.tick_duration.as_micros() > 0);
    }

    #[test]
    fn test_simulation_time() {
        let st = SimulationTime::new(-10000, 60);
        
        let (year, day) = st.ticks_to_date(0);
        assert_eq!(year, -10000);
        assert_eq!(day, 0);
        
        // After one year of ticks
        let ticks_per_year = st.ticks_per_day * 365;
        let (year, _) = st.ticks_to_date(ticks_per_year);
        assert_eq!(year, -9999);
    }

    #[test]
    fn test_date_formatting() {
        let st = SimulationTime::new(-5000, 60);
        let formatted = st.format_date(0);
        assert!(formatted.contains("5000 BCE"));
    }
}
