mod driver;
mod error;
mod ip;
mod net;
mod platform;
pub mod utils;

use std::sync::{Arc, atomic::AtomicBool};

use driver::loopback::LoopbackNetDevice;
use net::NET_PROTOCOL_TYPE_IP;

fn main() {
    utcp::log_init();

    let terminate = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&terminate)).unwrap();

    net::net_init().unwrap();
    let dev = LoopbackNetDevice::init().unwrap();

    net::net_run().unwrap();

    while !terminate.load(std::sync::atomic::Ordering::Relaxed) {
        net::net_device_output(&dev, NET_PROTOCOL_TYPE_IP, b"Hello, World", &mut []).unwrap();

        // sleep 1s
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    net::net_shutdown().unwrap();
}
