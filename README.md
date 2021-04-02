# frame_counter
[![Crates.io](https://img.shields.io/crates/v/frame_counter.svg)](https://crates.io/crates/frame_counter)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

The frame_counter library provides a very simple to use framerate counter
based around the [rust time module](https://github.com/rust-lang/rust/blob/673d0db5e393e9c64897005b470bfeb6d5aec61b/library/std/src/time.rs#L29).

Additionally the `FrameCounter` can also be used to cap the framerate at a certain value either in a hot or cold loop.

# Examples:

Counting the framerate:
```
use frame_counter::FrameCounter;

pub fn dummy_workload() {
    std::thread::sleep(std::time::Duration::from_millis(10));
}

pub fn main() {
    let mut frame_counter = FrameCounter::default();

    loop {
        {
            let frame = frame_counter.start_frame();

            dummy_workload();
        }

        println!("fps stats - {}", frame_counter);
    }
}
```
