mod driver;
mod platform;
mod net;
mod error;

use std::sync::{Arc, atomic::AtomicBool};

fn main() {
    env_logger::init();

    let terminate = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&terminate)).unwrap();

    net::net_init().unwrap();

    let dev = net::NetDevice::dummy();

    net::net_run().unwrap();

    while !terminate.load(std::sync::atomic::Ordering::Relaxed) {
        // Do some work

        // sleep 1s
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    net::net_shutdown().unwrap();
}
