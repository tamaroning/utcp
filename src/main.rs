mod driver;
mod platform;
mod net;
mod error;

use std::sync::{Arc, atomic::AtomicBool};

use driver::dummy::DummyNetDevice;
use net::NetDevice;

fn main() {
    utcp::log_init();

    let terminate = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&terminate)).unwrap();

    net::net_init().unwrap();

    let mut dev = DummyNetDevice::new();

    net::net_run().unwrap();

    while !terminate.load(std::sync::atomic::Ordering::Relaxed) {
        dev.transmit(b"Hello, world!", &mut []).unwrap();

        // sleep 1s
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    net::net_shutdown().unwrap();
}
