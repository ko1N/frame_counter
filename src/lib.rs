/*!
The frame_counter library provides a very simple to use framerate counter
with high-precision timing options.

# Features

- `std_time` (default) - Uses std::time::Instant
- `quanta` - Uses quanta crate for TSC-based timing
- `minstant` - Uses minstant crate for TSC-based timing with fallback

Add to Cargo.toml:
```toml
[dependencies]
frame_counter = { version = "*", default-features = false, features = ["quanta"] }
# or
frame_counter = { version = "*", default-features = false, features = ["minstant"] }
```

# Examples:

Counting the framerate:
```no_run
use frame_counter::{FrameCounter, StdTimer};

pub fn dummy_workload() {
    std::thread::sleep(std::time::Duration::from_millis(10));
}

pub fn main() {
    let mut frame_counter = FrameCounter::<StdTimer>::default();

    loop {
        frame_counter.tick();

        dummy_workload();

        println!("fps stats - {}", frame_counter);
    }
}
```
*/

pub const INITIAL_FRAMERATE: f64 = 100f64;

use std::fmt;

pub mod timer;
pub use timer::*;

pub struct FrameCounter<T>
where
    T: Timer,
{
    last_tick: T,
    frame_count: u64,
    last_frame_time: std::time::Duration,
    last_frame_rate: f64,
    avg_window_start: T,
    avg_frame_time: std::time::Duration,
    avg_frame_rate: f64,
    // For more accurate FPS capping
    target_frame_start: Option<T>,
    // For even more accurate averaging
    frame_times_buffer: Vec<u64>, // Store last N frame times in nanoseconds
    buffer_index: usize,
}

impl<T: Timer> Default for FrameCounter<T> {
    /// Creates a new FrameCounter with a starting framerate of 100.
    fn default() -> Self {
        Self::new(INITIAL_FRAMERATE)
    }
}

impl<T: Timer> FrameCounter<T> {
    /// Creates a new FrameCounter with the given starting framerate.
    ///
    /// # Arguments
    /// * `frame_rate` - initial frame rate guess.
    pub fn new(frame_rate: f64) -> Self {
        let now = T::now();
        // Keep a buffer of frame times for rolling average (1 second at target fps)
        let buffer_size = frame_rate.max(30.0) as usize;

        Self {
            last_tick: now,
            frame_count: 0u64,
            last_frame_time: std::time::Duration::from_secs_f64(1.0 / frame_rate),
            last_frame_rate: frame_rate,
            avg_window_start: now,
            avg_frame_time: std::time::Duration::from_secs_f64(1.0 / frame_rate),
            avg_frame_rate: frame_rate,
            target_frame_start: None,
            frame_times_buffer: vec![0u64; buffer_size],
            buffer_index: 0,
        }
    }

    /// Updates the frame measurements
    pub fn tick(&mut self) {
        let now = T::now();

        // Calculate frame time since last tick with nanosecond precision
        self.last_frame_time = now.duration_since(&self.last_tick);
        let frame_nanos = self.last_frame_time.as_nanos() as u64;

        // Store in circular buffer for rolling average
        self.frame_times_buffer[self.buffer_index] = frame_nanos;
        self.buffer_index = (self.buffer_index + 1) % self.frame_times_buffer.len();

        // Calculate instant framerate with higher precision
        if frame_nanos > 0 {
            self.last_frame_rate = 1_000_000_000.0 / frame_nanos as f64;
        }

        self.frame_count += 1;

        // Calculate rolling average using the buffer
        if self.frame_count >= self.frame_times_buffer.len() as u64 {
            let avg_nanos: u64 =
                self.frame_times_buffer.iter().sum::<u64>() / self.frame_times_buffer.len() as u64;
            self.avg_frame_time = std::time::Duration::from_nanos(avg_nanos);
            self.avg_frame_rate = 1_000_000_000.0 / avg_nanos as f64;
        } else {
            // Still filling buffer, use simple average
            let window_duration = now.duration_since(&self.avg_window_start);
            if self.frame_count > 0 {
                self.avg_frame_time = window_duration / self.frame_count as u32;
                self.avg_frame_rate = self.frame_count as f64 / window_duration.as_secs_f64();
            }
        }

        // Store for frame-rate capping
        self.target_frame_start = Some(now);

        // Update last tick time
        self.last_tick = now;
    }

    /// Waits in a hot-loop until the desired frame rate is achieved.
    /// Uses high-precision timing for accurate frame limiting.
    pub fn wait_until_framerate(&self, framerate: f64) {
        if let Some(frame_start) = self.target_frame_start {
            let target_nanos = (1_000_000_000.0 / framerate) as u128;

            // Use direct nanosecond comparison for highest precision
            let start_nanos = frame_start.as_nanos();

            loop {
                let current_nanos = T::now().as_nanos();
                if current_nanos.saturating_sub(start_nanos) >= target_nanos {
                    break;
                }

                // Yield to prevent excessive CPU cache thrashing
                std::hint::spin_loop();
            }
        }
    }

    /// Waits in a cold-loop until the desired frame rate is achieved.
    /// Combines sleep with high-precision spin-wait for accuracy.
    pub fn sleep_until_framerate(&self, framerate: f64) {
        if let Some(frame_start) = self.target_frame_start {
            let target_nanos = (1_000_000_000.0 / framerate) as u128;
            let start_nanos = frame_start.as_nanos();

            loop {
                let current_nanos = T::now().as_nanos();
                let elapsed_nanos = current_nanos.saturating_sub(start_nanos);

                if elapsed_nanos >= target_nanos {
                    break;
                }

                let remaining_nanos = target_nanos - elapsed_nanos;

                // Sleep for most of the remaining time, but wake up early
                // to account for sleep imprecision (typically ~1ms on most OSes)
                if remaining_nanos > 2_000_000 {
                    // More than 2ms remaining
                    std::thread::sleep(std::time::Duration::from_micros(500));
                } else if remaining_nanos > 100_000 {
                    // 100us to 2ms
                    std::thread::yield_now(); // Yield to scheduler
                } else {
                    // Final precision with spin loop
                    std::hint::spin_loop();
                }
            }
        }
    }

    /// Returns the frame time of the last frame as a `Duration`.
    pub fn frame_time(&self) -> std::time::Duration {
        self.last_frame_time
    }

    /// Returns the average frame time over the rolling window as a `Duration`.
    pub fn avg_frame_time(&self) -> std::time::Duration {
        self.avg_frame_time
    }

    /// Returns the frame rate of the last frame.
    pub fn frame_rate(&self) -> f64 {
        self.last_frame_rate
    }

    /// Returns the average frame rate over the rolling window.
    pub fn avg_frame_rate(&self) -> f64 {
        self.avg_frame_rate
    }

    /// Returns the total number of frames counted since creation.
    pub fn total_frames(&self) -> u64 {
        self.frame_count
    }
}

impl<T: Timer> fmt::Display for FrameCounter<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "avg: {:.2} fps ({:.3}ms); current: {:.2} fps ({:.3}ms)",
            self.avg_frame_rate(),
            self.avg_frame_time().as_secs_f64() * 1000.0,
            self.frame_rate(),
            self.frame_time().as_secs_f64() * 1000.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_counter_accuracy() {
        let mut fc = FrameCounter::<StdTimer>::new(60.0);

        // Simulate 60 fps workload
        for _ in 0..120 {
            fc.tick();
            fc.wait_until_framerate(60.0);
        }

        // Should be close to 60 fps
        let avg_fps = fc.avg_frame_rate();
        assert!(
            (avg_fps - 60.0).abs() < 2.0,
            "Average FPS {} not close to 60",
            avg_fps
        );
    }
}
