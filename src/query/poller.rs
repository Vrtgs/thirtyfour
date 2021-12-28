use crate::support::sleep;
use crate::WebDriver;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Parameters used to determine the polling / timeout behaviour.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ElementPoller {
    /// No polling, single attempt.
    NoWait,
    /// Poll up to the specified timeout, with the specified interval being the
    /// minimum time elapsed between the start of each poll attempt.
    /// If the previous poll attempt took longer than the interval, the next will
    /// start immediately. Once the timeout is reached, a Timeout error will be
    /// returned regardless of the actual number of polling attempts completed.
    TimeoutWithInterval(Duration, Duration),
    /// Poll once every interval, up to the maximum number of polling attempts.
    /// If the previous poll attempt took longer than the interval, the next will
    /// start immediately. However, in the case that the desired element is not
    /// found, you will be guaranteed the specified number of polling attempts,
    /// regardless of how long it takes.
    NumTriesWithInterval(u32, Duration),
    /// Poll once every interval, up to the specified timeout, or the specified
    /// minimum number of polling attempts, whichever comes last.
    /// If the previous poll attempt took longer than the interval, the next will
    /// start immediately. If the timeout was reached before the minimum number
    /// of polling attempts has been executed, then the query will continue
    /// polling until the number of polling attempts equals the specified minimum.
    /// If the minimum number of polling attempts is reached prior to the
    /// specified timeout, then the polling attempts will continue until the
    /// timeout is reached instead.
    TimeoutWithIntervalAndMinTries(Duration, Duration, u32),
}

impl Default for ElementPoller {
    fn default() -> Self {
        ElementPoller::TimeoutWithInterval(Duration::from_secs(20), Duration::from_millis(500))
    }
}

pub struct ElementPollerTicker {
    timeout: Option<Duration>,
    interval: Option<Duration>,
    min_tries: u32,
    start: Instant,
    cur_tries: u32,
}

impl ElementPollerTicker {
    pub fn new(poller: ElementPoller) -> Self {
        let mut ticker = Self {
            timeout: None,
            interval: None,
            min_tries: 0,
            start: Instant::now(),
            cur_tries: 0,
        };

        match poller {
            ElementPoller::NoWait => {}
            ElementPoller::TimeoutWithInterval(timeout, interval) => {
                ticker.timeout = Some(timeout);
                ticker.interval = Some(interval);
            }
            ElementPoller::NumTriesWithInterval(num_tries, interval) => {
                ticker.interval = Some(interval);
                ticker.min_tries = num_tries;
            }
            ElementPoller::TimeoutWithIntervalAndMinTries(timeout, interval, num_tries) => {
                ticker.timeout = Some(timeout);
                ticker.interval = Some(interval);
                ticker.min_tries = num_tries
            }
        }

        ticker
    }

    pub async fn tick(&mut self) -> bool {
        self.cur_tries += 1;

        if self.timeout.filter(|t| &self.start.elapsed() < t).is_none()
            && self.cur_tries >= self.min_tries
        {
            return false;
        }

        if let Some(i) = self.interval {
            // Next poll is due no earlier than this long after the first poll started.
            let minimum_elapsed = i * self.cur_tries;

            // But this much time has elapsed since the first poll started.
            let actual_elapsed = self.start.elapsed();

            if actual_elapsed < minimum_elapsed {
                // So we need to wait this much longer.
                sleep(minimum_elapsed - actual_elapsed).await;
            }
        }

        true
    }
}

impl WebDriver {
    pub fn set_query_poller(&self, poller: ElementPoller) {
        self.handle.config.set_query_poller(poller);
    }
}
