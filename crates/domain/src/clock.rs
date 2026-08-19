//! Clock abstraction for deterministic testing.

use time::OffsetDateTime;

/// Trait for obtaining the current time. Allows deterministic tests via `TestClock`.
pub trait Clock: Send + Sync {
    fn now(&self) -> OffsetDateTime;
}

/// Production clock using the system time.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

/// Test clock with a fixed or manually advanced time.
#[derive(Debug, Clone)]
pub struct TestClock {
    now: OffsetDateTime,
}

impl TestClock {
    pub fn new(now: OffsetDateTime) -> Self {
        Self { now }
    }

    pub fn advance(&mut self, duration: time::Duration) {
        self.now += duration;
    }

    pub fn set(&mut self, now: OffsetDateTime) {
        self.now = now;
    }
}

impl Clock for TestClock {
    fn now(&self) -> OffsetDateTime {
        self.now
    }
}
