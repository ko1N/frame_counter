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

pub const INITIAL_FRAMERATE: f64 = 100f64;

use std::fmt;
use std::time::{Duration, Instant};

pub struct FrameCounter {
    now: Instant,
    frame_count: u64,
    last_frame_time: Duration,
    last_frame_rate: f64,
    avg_now: Instant,
    avg_frame_count: u64,
    avg_frame_time: Duration,
    avg_frame_rate: f64,
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
        Self {
            now: Instant::now(),
            frame_count: 0u64,
            last_frame_time: Duration::from_millis((1000f64 / frame_rate) as u64),
            last_frame_rate: frame_rate,
            avg_now: Instant::now(),
            avg_frame_count: 0u64,
            avg_frame_time: Duration::from_millis((1000f64 / frame_rate) as u64),
            avg_frame_rate: frame_rate,
        }
    }

    /// Updates the frame measurements
    pub fn tick(&mut self) {
        // update last tick
        self.last_frame_time = self.now.elapsed();
        self.last_frame_rate = 1e+9f64 / (self.last_frame_time.as_nanos() as f64);
        self.frame_count += 1;

        // update average each second
        if self.avg_now.elapsed().as_millis() > 1000 {
            self.avg_frame_time =
                self.avg_now.elapsed() / (self.frame_count - self.avg_frame_count) as u32;
            self.avg_frame_rate = 1e+9f64 / (self.avg_frame_time.as_nanos() as f64);
            self.avg_frame_count = self.frame_count;
            self.avg_now = self.now;
        }

        // start new tick
        self.now = Instant::now();
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
        let target_frame_time_nanos = 1e+9f64 / framerate;
        while target_frame_time_nanos > (self.now.elapsed().as_nanos() as f64) {}
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
        let target_frame_time_nanos = 1e+9f64 / framerate;
        while target_frame_time_nanos > (self.now.elapsed().as_nanos() as f64) {
            std::thread::sleep(std::time::Duration::from_millis(1));
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
}

impl fmt::Display for FrameCounter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "avg: {} {:?}; current: {} {:?}",
            self.avg_frame_rate(),
            self.avg_frame_time(),
            self.frame_rate(),
            self.frame_time()
        )
    }
}
