use frame_counter::FrameCounter;

pub fn dummy_workload() {
    std::thread::sleep(std::time::Duration::from_millis(1));
}

pub fn main() {
    let mut frame_counter = FrameCounter::default();

    loop {
        {
            let frame = frame_counter.start_frame();

            dummy_workload();

            // hot loop, do not trigger scheduler
            frame.sleep_until_framerate(60f64);
        }

        println!("fps stats - {}", frame_counter);
    }
}
