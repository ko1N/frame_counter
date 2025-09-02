use std::time::{Duration, Instant};

pub trait Timer
where
    Self: Sized + Copy + Clone,
{
    fn now() -> Self;
    fn duration_since(&self, earlier: &Self) -> Duration;
    fn as_nanos(&self) -> u128;
}

#[derive(Clone, Copy)]
pub struct StdTimer {
    instant: Instant,
}

impl Timer for StdTimer {
    fn now() -> Self {
        Self {
            instant: Instant::now(),
        }
    }

    fn duration_since(&self, earlier: &Self) -> Duration {
        self.instant.duration_since(earlier.instant)
    }

    fn as_nanos(&self) -> u128 {
        // For std::time, we can't get absolute nanos, so we use a static reference point
        static INIT: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
        let start = INIT.get_or_init(|| Instant::now());
        self.instant.duration_since(*start).as_nanos()
    }
}

#[cfg(feature = "quanta")]
#[derive(Clone, Copy)]
pub struct QuantaTimer {
    ticks: u64,
}

#[cfg(feature = "quanta")]
impl Timer for QuantaTimer {
    fn now() -> Self {
        // quanta::Clock uses TSC (Time Stamp Counter) on x86/x86_64
        // which provides nanosecond-level precision
        static CLOCK: std::sync::OnceLock<quanta::Clock> = std::sync::OnceLock::new();
        let clock = CLOCK.get_or_init(|| quanta::Clock::new());
        Self { ticks: clock.raw() }
    }

    fn duration_since(&self, earlier: &Self) -> Duration {
        static CLOCK: std::sync::OnceLock<quanta::Clock> = std::sync::OnceLock::new();
        let clock = CLOCK.get_or_init(|| quanta::Clock::new());

        let nanos = clock.delta(earlier.ticks, self.ticks).as_nanos();
        Duration::from_nanos(nanos as u64)
    }

    fn as_nanos(&self) -> u128 {
        static CLOCK: std::sync::OnceLock<quanta::Clock> = std::sync::OnceLock::new();
        let clock = CLOCK.get_or_init(|| quanta::Clock::new());
        clock.delta(0, self.ticks).as_nanos()
    }
}

#[cfg(feature = "minstant")]
#[derive(Clone, Copy)]
pub struct MInstantTimer {
    instant: minstant::Instant,
    anchor: minstant::Anchor,
}

#[cfg(feature = "minstant")]
impl Timer for MInstantTimer {
    fn now() -> Self {
        // minstant uses TSC on x86/x86_64 with automatic calibration
        // Falls back to std::time on other platforms
        Self {
            instant: minstant::Instant::now(),
            anchor: minstant::Anchor::new(),
        }
    }

    fn duration_since(&self, earlier: &Self) -> Duration {
        self.instant.duration_since(earlier.instant)
    }

    fn as_nanos(&self) -> u128 {
        self.instant.as_unix_nanos(&self.anchor) as u128
    }
}
