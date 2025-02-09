use std::sync::{Arc, atomic::AtomicBool};

use error::UtcpResult;
use utcp::{driver::loopback::LoopbackNetDevice, *};

use net::NET_PROTOCOL_TYPE_IP;

const TEST_DATA: [u8; 48] = [
    0x45, 0x00, 0x00, 0x30, 0x00, 0x80, 0x00, 0x00, 0xff, 0x01, 0xbd, 0x4a, 0x7f, 0x00, 0x00, 0x01,
    0x7f, 0x00, 0x00, 0x01, 0x08, 0x00, 0x35, 0x64, 0x00, 0x80, 0x00, 0x01, 0x31, 0x32, 0x33, 0x34,
    0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x21, 0x40, 0x23, 0x24, 0x25, 0x5e, 0x26, 0x2a, 0x28, 0x29,
];

fn utcp_main() -> UtcpResult<()> {
    utcp::log_init();

    let terminate = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&terminate)).unwrap();

    net::net_init()?;
    let dev = LoopbackNetDevice::init()?;

    net::net_run()?;

    while !terminate.load(std::sync::atomic::Ordering::Relaxed) {
        net::net_device_output(&dev, NET_PROTOCOL_TYPE_IP, &TEST_DATA, &mut []).unwrap();

        // sleep 1s
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    net::net_shutdown()?;

    Ok(())
}

fn main() {
    if let Err(e) = utcp_main() {
        log::error!("{}", e);
    }
}
