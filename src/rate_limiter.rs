use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RateLimiter {
    max_executions_per_window: u64,
    executions_left_in_window: u64,
    window_start_time: Instant,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_executions_per_internal: u64, interval: Duration) -> Self {
        Self {
            max_executions_per_window: max_executions_per_internal,
            executions_left_in_window: max_executions_per_internal,
            window_start_time: Instant::now(),
            window_duration: interval,
        }
    }

    pub fn attempt(&mut self) -> bool {
        if self.window_start_time.elapsed() > self.window_duration {
            self.window_start_time = Instant::now();
            self.executions_left_in_window = self.max_executions_per_window;
        }

        if self.executions_left_in_window < 1 {
            return false;
        }
        self.executions_left_in_window -= 1;

        true
    }

    pub const fn remaining(&self) -> u64 {
        self.executions_left_in_window
    }
}
