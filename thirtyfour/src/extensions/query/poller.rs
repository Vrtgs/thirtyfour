use crate::support::sleep;
use std::fmt::Debug;
use std::time::{Duration, Instant};

/// Trait for implementing the element polling strategy.
///
/// Each time the element condition is not met, the `tick()` method will be
/// called. Upon returning `false`, the polling loop will terminate.
#[async_trait::async_trait]
pub trait ElementPoller: Debug {
    /// Process the poller forward by one tick.
    async fn tick(&mut self) -> bool;
}

/// Trait for returning a struct that implements ElementPoller.
///
/// The start() method will be called at the beginning of the polling loop.
pub trait IntoElementPoller: Debug {
    /// Start a new poller.
    fn start(&self) -> Box<dyn ElementPoller + Send + Sync>;
}

/// Poll up to the specified timeout, with the specified interval being the
/// minimum time elapsed between the start of each poll attempt.
/// If the previous poll attempt took longer than the interval, the next will
/// start immediately. Once the timeout is reached, a Timeout error will be
/// returned regardless of the actual number of polling attempts completed.
#[derive(Debug)]
pub struct ElementPollerWithTimeout {
    timeout: Duration,
    interval: Duration,
    start: Instant,
    cur_tries: u32,
}

impl ElementPollerWithTimeout {
    /// Create a new `ElementPollerWithTimeout`.
    pub fn new(timeout: Duration, interval: Duration) -> Self {
        Self {
            timeout,
            interval,
            start: Instant::now(),
            cur_tries: 0,
        }
    }
}

impl Default for ElementPollerWithTimeout {
    fn default() -> Self {
        Self::new(Duration::from_secs(20), Duration::from_millis(500))
    }
}

#[async_trait::async_trait]
impl ElementPoller for ElementPollerWithTimeout {
    async fn tick(&mut self) -> bool {
        self.cur_tries += 1;

        if self.start.elapsed() >= self.timeout {
            return false;
        }

        // The Next poll is due no earlier than this long after the first poll started.
        let minimum_elapsed = self.interval.saturating_mul(self.cur_tries);

        // But this much time has elapsed since the first poll started.
        let actual_elapsed = self.start.elapsed();

        if actual_elapsed < minimum_elapsed {
            // So we need to wait this much longer.
            sleep(minimum_elapsed - actual_elapsed).await;
        }

        true
    }
}

impl IntoElementPoller for ElementPollerWithTimeout {
    fn start(&self) -> Box<dyn ElementPoller + Send + Sync> {
        Box::new(Self::new(self.timeout, self.interval))
    }
}

/// No polling, single attempt.
#[derive(Debug)]
pub struct ElementPollerNoWait;

#[async_trait::async_trait]
impl ElementPoller for ElementPollerNoWait {
    async fn tick(&mut self) -> bool {
        false
    }
}

impl IntoElementPoller for ElementPollerNoWait {
    fn start(&self) -> Box<dyn ElementPoller + Send + Sync> {
        Box::new(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_poller_with_timeout() {
        let mut poller =
            ElementPollerWithTimeout::new(Duration::from_secs(1), Duration::from_millis(500));
        assert!(poller.tick().await);
        // This should have waited 500ms already.
        // Waiting an additional 500ms should exceed the timeout.
        sleep(Duration::from_millis(500)).await;
        assert!(!poller.tick().await);
    }

    #[tokio::test]
    async fn test_poller_nowait() {
        let mut poller = ElementPollerNoWait;
        assert!(!poller.tick().await); // Should instantly return false.
    }
}
