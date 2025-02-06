use std::sync::{Mutex, atomic::AtomicU32};

use bitflags::bitflags;

use crate::{driver::dummy::DummyNetDevice, error::UtcpResult};

pub fn new_device_index() -> u32 {
    static IDX: AtomicU32 = AtomicU32::new(0);
    IDX.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

bitflags! {
    #[derive(Debug)]
    pub struct NetDeviceFlags: u16 {
        const UP = 0x1;
        const LOOPBACK = 0x10;
        const BROADCAST = 0x20;
        const P2P = 0x40;
        const NEED_ARP = 0x100;
    }
}

#[derive(Debug)]
pub enum NetDevice {
    Dummy(DummyNetDevice),
    Loopback,
    Ethernet,
}

impl NetDevice {
    fn mtu(&self) -> u16 {
        match self {
            NetDevice::Dummy(dev) => 0,
            NetDevice::Loopback => todo!(),
            NetDevice::Ethernet => todo!(),
        }
    }

    fn is_up(&self) -> bool {
        match self {
            NetDevice::Dummy(dev) => dev.is_up(),
            NetDevice::Loopback => false,
            NetDevice::Ethernet => false,
        }
    }

    fn open(&mut self) -> UtcpResult<()> {
        match self {
            NetDevice::Dummy(dev) => dev.open(),
            NetDevice::Loopback => todo!(),
            NetDevice::Ethernet => todo!(),
        }
    }

    fn close(&mut self) -> UtcpResult<()> {
        match self {
            NetDevice::Dummy(dev) => dev.close(),
            NetDevice::Loopback => todo!(),
            NetDevice::Ethernet => todo!(),
        }
    }

    fn transmit(&mut self, data: &[u8], dst: &mut [u8]) -> UtcpResult<()> {
        match self {
            NetDevice::Dummy(dev) => dev.transmit(data, dst),
            NetDevice::Loopback => todo!(),
            NetDevice::Ethernet => todo!(),
        }
    }
}

pub trait NetDeviceOps {
    const MTU: u16;
    const HEADER_LEN: usize;
    const ADDR_LEN: usize;

    fn is_up(&self) -> bool;
    fn open(&mut self) -> UtcpResult<()>;
    fn close(&mut self) -> UtcpResult<()>;
    fn transmit(&mut self, data: &[u8], dst: &mut [u8]) -> UtcpResult<()>;
}

pub struct NetContext {
    devices: Vec<Mutex<NetDevice>>,
}

impl NetContext {
    pub fn new() -> Self {
        let net = NetContext { devices: vec![] }; 
        net
    }

    fn init(&self) {}
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
