use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex, atomic::AtomicU32},
};

use bitflags::bitflags;

use crate::{driver::dummy::DummyNetDevice, error::UtcpResult, platform::linux::intr};

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

#[derive(Debug)]
pub enum NetDeviceType {
    Dummy,
    Loopback,
    Ethernet,
}

impl NetDevice {
    fn device_type(&self) -> NetDeviceType {
        match self {
            NetDevice::Dummy(_) => NetDeviceType::Dummy,
            NetDevice::Loopback => NetDeviceType::Loopback,
            NetDevice::Ethernet => NetDeviceType::Ethernet,
        }
    }

    fn mtu(&self) -> u16 {
        match self {
            NetDevice::Dummy(dev) => 0,
            NetDevice::Loopback => todo!(),
            NetDevice::Ethernet => todo!(),
        }
    }

    fn name(&self) -> &str {
        match self {
            NetDevice::Dummy(dev) => dev.name(),
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
    fn name(&self) -> &str;

    fn open(&mut self) -> UtcpResult<()>;
    fn close(&mut self) -> UtcpResult<()>;
    fn transmit(&mut self, data: &[u8], dst: &mut [u8]) -> UtcpResult<()>;
}

#[derive(Debug, Clone)]
pub struct NetDeviceHandler(String);

// TODO: use a better data structure, e.g. index
static DEVICES: LazyLock<Mutex<HashMap<String, NetDevice>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn net_init() -> UtcpResult<()> {
    intr::intr_init()?;
    log::info!("initialized");
    Ok(())
}

pub fn net_run() -> UtcpResult<()> {
    intr::intr_run()?;
    log::info!("opening all devices");
    let mut devices = DEVICES.lock().unwrap();
    for (_, dev) in devices.iter_mut() {
        net_device_open(dev)?;
    }
    Ok(())
}

pub fn net_shutdown() -> UtcpResult<()> {
    intr::intr_shutdown()?;
    let mut devices = DEVICES.lock().unwrap();
    for (_, dev) in devices.iter_mut() {
        net_device_close(dev)?;
    }
    log::info!("shutting down");
    Ok(())
}

pub fn net_device_register(dev: NetDevice) -> UtcpResult<NetDeviceHandler> {
    log::debug!("register dev={}, type={:?}", dev.name(), dev.device_type());
    let mut devices = DEVICES.lock().unwrap();
    let handler = NetDeviceHandler(dev.name().to_string());
    devices.insert(dev.name().to_string(), dev);
    Ok(handler)
}

pub fn net_device_output(dev: &NetDeviceHandler, data: &[u8], dst: &mut [u8]) -> UtcpResult<()> {
    let mut devices = DEVICES.lock().unwrap();
    let dev = devices.get_mut(&dev.0).unwrap();
    dev.transmit(data, dst)
}

fn net_device_open(dev: &mut NetDevice) -> UtcpResult<()> {
    dev.open()?;
    log::info!(
        "dev={}, state={}",
        dev.name(),
        if dev.is_up() { "up" } else { "down" }
    );
    Ok(())
}

fn net_device_close(dev: &mut NetDevice) -> UtcpResult<()> {
    dev.close()?;
    log::info!(
        "dev={}, state={}",
        dev.name(),
        if dev.is_up() { "up" } else { "down" }
    );
    Ok(())
}
