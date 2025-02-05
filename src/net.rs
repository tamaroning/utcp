use std::sync::atomic::AtomicU32;

use bitflags::bitflags;

use crate::error::UtcpResult;

pub const DUMMY_MTU: u16 = u16::MAX;

pub fn new_device_index() -> u32 {
    static IDX: AtomicU32 = AtomicU32::new(0);
    IDX.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[derive(Debug, Copy, Clone)]
pub enum NetDeviceType {
    Dummy,
    Loopback,
    Ethernet,
}

bitflags! {
    pub struct NetDeviceFlags: u16 {
        const UP = 0x1;
        const LOOPBACK = 0x10;
        const BROADCAST = 0x20;
        const P2P = 0x40;
        const NEED_ARP = 0x100;
    }
}

pub trait NetDevice {
    const DEVICE_TYPE: NetDeviceType;
    const MTU: u16;
    const HEADER_LEN: u16;
    const ADDR_LEN: u16;

    fn open(&mut self) -> UtcpResult<()>;
    fn close(&mut self) -> UtcpResult<()>;
    fn transmit(&mut self, data: &[u8], dst: &mut [u8]) -> UtcpResult<()>;
}

pub fn net_run() -> UtcpResult<()> {
    Ok(())
}

pub fn net_shutdown() -> UtcpResult<()> {
    log::info!("closing all devices");
    Ok(())
}

pub fn net_init() -> UtcpResult<()> {
    log::info!("initialized");
    Ok(())
}
