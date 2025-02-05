mod driver;
mod platform;
mod net;
mod error;

use std::sync::{Arc, atomic::AtomicBool};

use net::NetDevice;

fn main() {
    utcp::log_init();
}
