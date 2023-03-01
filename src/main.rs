use ps_affinity::set_process_affinity;
use std::{thread, time};

fn main() {
    // TODO: Config
    let interval = time::Duration::from_secs(10);
    loop {
        match set_process_affinity("audiodg.exe", 1) {
            Ok(details) => println!("{details}"),
            Err(error) => println!("{error}"),
        }

        thread::sleep(interval);
    }
}
