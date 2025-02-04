use std::sync::atomic::AtomicU32;

use bitflags::bitflags;

use crate::error::UtcpResult;

const IF_NAMESIZE: usize = 16;
const NET_DEVICE_ADDR_LEN: usize = 16;

const DUMMY_MTU: u16 = u16::MAX;

fn new_device_index() -> u32 {
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

pub struct NetDevice_ {
    name: String,
    ty: NetDeviceType,
    mtu: u16,
    flags: NetDeviceFlags,
    header_len: u16,
    addr_len: u16,
    addr: String,
}

pub trait NetDevice {
    const DEVICE_TYPE: NetDeviceType;
    const MTU: u16;
    const HEADER_LEN: u16;
    const ADDR_LEN: u16;

    fn open(&mut self) -> UtcpResult<()>;
    fn close(&mut self) -> UtcpResult<()>;
    fn transmit(&mut self, data: &[u8]) -> UtcpResult<()>;
}

pub struct DummyNetDevice {
    name: String,
}

impl DummyNetDevice {
    pub fn new() -> Self {
        let dev = Self {
            name: format!("dev{}", new_device_index()),
        };
        log::debug!("initialized dev={}", dev.name);
        dev
    }
}

impl NetDevice for DummyNetDevice {
    const DEVICE_TYPE: NetDeviceType = NetDeviceType::Dummy;
    const MTU: u16 = DUMMY_MTU;
    const HEADER_LEN: u16 = 0;
    const ADDR_LEN: u16 = 0;

    fn open(&mut self) -> UtcpResult<()> {
        log::debug!("opened dev={}", self.name);
        Ok(())
    }

    fn close(&mut self) -> UtcpResult<()> {
        log::debug!("closed dev={}", self.name);
        Ok(())
    }

    fn transmit(&mut self, data: &[u8]) -> UtcpResult<()> {
        log::debug!("dev={}, type={:?}", self.name, Self::DEVICE_TYPE);
        log::debug!("{:?}", data);
        Ok(())
    }
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
