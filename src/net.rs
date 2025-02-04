use std::sync::atomic::AtomicU32;

use bitflags::bitflags;

use crate::error::UtcpResult;

const IF_NAMESIZE: usize = 16;
const NET_DEVICE_ADDR_LEN: usize = 16;

const DUMMY_MTU: u16 = u16::MAX;

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

pub struct NetDevice {
    name: String,
    ty: NetDeviceType,
    mtu: u16,
    flags: NetDeviceFlags,
    header_len: u16,
    addr_len: u16,
    addr: String,
}

//enum NetDevice

impl NetDevice {
    fn new() -> Self {
        static IDX: AtomicU32 = AtomicU32::new(0);
        let idx = IDX.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Self {
            name: format!("net{}", idx),
            ty: NetDeviceType::Dummy,
            mtu: 0,
            flags: NetDeviceFlags::empty(),
            header_len: 0,
            addr_len: 0,
            addr: String::new(),
        }
    }

    pub fn dummy() -> Self {
        let mut dev = Self::new();

        dev.ty = NetDeviceType::Dummy;
        dev.mtu = DUMMY_MTU;
        dev.header_len = 0;
        dev.addr_len = 0;

        log::debug!("initialized dev={}", dev.name);
        dev
    }
}

pub trait NetDeviceOps {
    fn open(&mut self);
    fn close(&mut self);
    fn transmit(&mut self);
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
