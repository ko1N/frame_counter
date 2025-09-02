/*!
The frame_counter library provides a very simple to use framerate counter
based around the [rust time module](https://github.com/rust-lang/rust/blob/673d0db5e393e9c64897005b470bfeb6d5aec61b/library/std/src/time.rs#L29).

Additionally the `FrameCounter` can also be used to cap the framerate at a certain value either in a hot or cold loop.

# Examples:

Counting the framerate:
```no_run
use frame_counter::FrameCounter;

pub fn dummy_workload() {
    std::thread::sleep(std::time::Duration::from_millis(10));
}

pub fn main() {
    let mut frame_counter = FrameCounter::default();

    loop {
        frame_counter.tick();

        dummy_workload();

        println!("fps stats - {}", frame_counter);
    }
}
```
*/

pub const INITIAL_FRAMERATE: f64 = 60f64;

use std::fmt;
use std::time::{Duration, Instant};

pub struct FrameCounter {
    last_tick: Instant,
    frame_count: u64,
    last_frame_time: Duration,
    last_frame_rate: f64,
    avg_window_start: Instant,
    avg_frame_count_at_window_start: u64,
    avg_frame_time: Duration,
    avg_frame_rate: f64,
    // For more accurate FPS capping
    target_frame_start: Option<Instant>,
}

impl Default for FrameCounter {
    /// Creates a new FrameCounter with a starting framerate of 100.
    fn default() -> Self {
        Self::new(INITIAL_FRAMERATE)
    }
}

impl FrameCounter {
    /// Creates a new FrameCounter with the given starting framerate.
    ///
    /// # Arguments
    /// * `frame_rate` - initial frame rate guess.
    pub fn new(frame_rate: f64) -> Self {
        let now = Instant::now();
        Self {
            last_tick: now,
            frame_count: 0u64,
            last_frame_time: Duration::from_secs_f64(1.0 / frame_rate),
            last_frame_rate: frame_rate,
            avg_window_start: now,
            avg_frame_count_at_window_start: 0u64,
            avg_frame_time: Duration::from_secs_f64(1.0 / frame_rate),
            avg_frame_rate: frame_rate,
            target_frame_start: None,
        }
    }

    /// Updates the frame measurements
    pub fn tick(&mut self) {
        let now = Instant::now();

        // Calculate frame time since last tick
        self.last_frame_time = now.duration_since(self.last_tick);
        self.last_frame_rate = 1.0 / self.last_frame_time.as_secs_f64();
        self.frame_count += 1;

        // Update average each second (using a sliding window)
        let window_duration = now.duration_since(self.avg_window_start);
        if window_duration.as_secs_f64() >= 1.0 {
            let frames_in_window = self.frame_count - self.avg_frame_count_at_window_start;
            if frames_in_window > 0 {
                self.avg_frame_time = window_duration / frames_in_window as u32;
                self.avg_frame_rate = frames_in_window as f64 / window_duration.as_secs_f64();
            }
            self.avg_window_start = now;
            self.avg_frame_count_at_window_start = self.frame_count;
        }

        // Store for frame-rate capping
        self.target_frame_start = Some(now);

        // Update last tick time
        self.last_tick = now;
    }

    /// Waits in a hot-loop until the desired frame rate is achieved.
    /// This function will _not_ call `std::thread:sleep` to prevent
    /// the OS from scheduling this thread.
    ///
    /// This means the function will consume an entire core on the CPU.
    ///
    /// # Example:
    ///
    /// ```no_run
    /// use frame_counter::FrameCounter;
    ///
    /// pub fn dummy_workload() {
    ///     std::thread::sleep(std::time::Duration::from_millis(1));
    /// }
    ///
    /// pub fn main() {
    ///     let mut frame_counter = FrameCounter::default();
    ///
    ///     loop {
    ///         frame_counter.tick();
    ///
    ///         dummy_workload();
    ///
    ///         frame_counter.wait_until_framerate(60f64);
    ///
    ///         println!("fps stats - {}", frame_counter);
    ///     }
    /// }
    /// ```
    pub fn wait_until_framerate(&self, framerate: f64) {
        if let Some(frame_start) = self.target_frame_start {
            let target_frame_duration = Duration::from_secs_f64(1.0 / framerate);

            // Spin until we reach the target frame time
            loop {
                let elapsed = Instant::now().duration_since(frame_start);
                if elapsed >= target_frame_duration {
                    break;
                }

                // Yield to prevent excessive CPU cache thrashing
                // This is a hint to the CPU that we're in a spin loop
                std::hint::spin_loop();
            }
        }
    }

    /// Waits in a cold-loop until the desired frame rate is achieved.
    /// This function will call `std::thread:sleep` to prevent
    /// the process from consuming the entire CPU Core.
    ///
    /// # Example:
    ///
    /// ```no_run
    /// use frame_counter::FrameCounter;
    ///
    /// pub fn dummy_workload() {
    ///     std::thread::sleep(std::time::Duration::from_millis(1));
    /// }
    ///
    /// pub fn main() {
    ///     let mut frame_counter = FrameCounter::default();
    ///
    ///     loop {
    ///         frame_counter.tick();
    ///
    ///         dummy_workload();
    ///
    ///         frame_counter.sleep_until_framerate(60f64);
    ///
    ///         println!("fps stats - {}", frame_counter);
    ///     }
    /// }
    /// ```
    pub fn sleep_until_framerate(&self, framerate: f64) {
        if let Some(frame_start) = self.target_frame_start {
            let target_frame_duration = Duration::from_secs_f64(1.0 / framerate);

            loop {
                let elapsed = Instant::now().duration_since(frame_start);
                if elapsed >= target_frame_duration {
                    break;
                }

                let remaining = target_frame_duration - elapsed;

                // Sleep for most of the remaining time, but wake up a bit early
                // to account for sleep imprecision
                if remaining > Duration::from_millis(2) {
                    std::thread::sleep(Duration::from_millis(1));
                } else {
                    // For the last bit, use a spin loop for precision
                    std::hint::spin_loop();
                }
            }
        }
    }

    /// Returns the frame time of the last frame as a `Duration`.
    pub fn frame_time(&self) -> Duration {
        self.last_frame_time
    }

    /// Returns the average frame time over the last second as a `Duration`.
    pub fn avg_frame_time(&self) -> Duration {
        self.avg_frame_time
    }

    /// Returns the frame rate of the last frame.
    pub fn frame_rate(&self) -> f64 {
        self.last_frame_rate
    }

    /// Returns the average frame rate of the last second.
    pub fn avg_frame_rate(&self) -> f64 {
        self.avg_frame_rate
    }

    /// Returns the total number of frames counted since creation.
    pub fn total_frames(&self) -> u64 {
        self.frame_count
    }
}

impl fmt::Display for FrameCounter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "avg: {:.2} fps ({:.3}ms); current: {:.2} fps ({:.3}ms)",
            self.avg_frame_rate(),
            self.avg_frame_time().as_secs_f64() * 1000.0,
            self.frame_rate(),
            self.frame_time().as_secs_f64() * 1000.0
        )
    }
}
