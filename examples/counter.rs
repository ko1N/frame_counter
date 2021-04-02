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
