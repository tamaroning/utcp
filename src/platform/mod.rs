use crate::net::NetDeviceHandler;

pub mod linux;

use bitflags::bitflags;

#[derive(Debug)]
struct IRQEntry {
    irq: i32,
    flags: IRQFlags,
    debug_name: String,
    dev: NetDeviceHandler,
    handler: fn(irq: i32, dev: NetDeviceHandler),
}

bitflags! {
    #[derive(Debug)]
    pub  struct IRQFlags: u32 {
        const SHARED = 0x01;
    }
}
