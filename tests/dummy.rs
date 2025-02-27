use std::sync::{Arc, atomic::AtomicBool};

use driver::dummy::DummyNetDevice;
use utcp::{driver, net};

fn main() {
    utcp::log_init();

    let terminate = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&terminate)).unwrap();

    net::net_init().unwrap();
    let dev = DummyNetDevice::init().unwrap();

    net::net_run().unwrap();

    while !terminate.load(std::sync::atomic::Ordering::Relaxed) {
        net::net_device_output(&dev, 0, b"Hello, World", &mut []).unwrap();

        // sleep 1s
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    net::net_shutdown().unwrap();
}
