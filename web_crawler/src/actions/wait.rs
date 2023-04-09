use random_number::random;
use std::thread;
use std::time::Duration;

pub fn wait(start: u64, end: u64) {
    let delay: u64 = random!(start, end); // random delay between 3 and 15 seconds
    thread::sleep(Duration::from_secs(delay as u64));
}
