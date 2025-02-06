use std::sync::{LazyLock, LockResult, RwLock, RwLockWriteGuard, atomic::AtomicU32};

use bitflags::bitflags;
use crossbeam_skiplist::SkipMap;

use crate::{
    driver::{dummy::DummyNetDevice, loopback::LoopbackNetDevice},
    error::{UtcpErr, UtcpResult},
    platform::linux::intr,
};

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
    Loopback(LoopbackNetDevice),
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
            NetDevice::Loopback(_) => NetDeviceType::Loopback,
            NetDevice::Ethernet => NetDeviceType::Ethernet,
        }
    }

    fn mtu(&self) -> u16 {
        match self {
            NetDevice::Dummy(_) => DummyNetDevice::MTU,
            NetDevice::Loopback(_) => LoopbackNetDevice::MTU,
            NetDevice::Ethernet => todo!(),
        }
    }

    fn name(&self) -> &str {
        match self {
            NetDevice::Dummy(dev) => dev.name(),
            NetDevice::Loopback(dev) => dev.name(),
            NetDevice::Ethernet => todo!(),
        }
    }

    fn is_up(&self) -> bool {
        match self {
            NetDevice::Dummy(dev) => dev.is_up(),
            NetDevice::Loopback(dev) => dev.is_up(),
            NetDevice::Ethernet => false,
        }
    }

    fn open(&mut self) -> UtcpResult<()> {
        match self {
            NetDevice::Dummy(dev) => dev.open(),
            NetDevice::Loopback(dev) => dev.open(),
            NetDevice::Ethernet => todo!(),
        }
    }

    fn close(&mut self) -> UtcpResult<()> {
        match self {
            NetDevice::Dummy(dev) => dev.close(),
            NetDevice::Loopback(dev) => dev.close(),
            NetDevice::Ethernet => todo!(),
        }
    }

    fn transmit(&mut self, ty: u16, data: &[u8], dst: &mut [u8]) -> UtcpResult<()> {
        match self {
            NetDevice::Dummy(dev) => dev.transmit(ty, data, dst),
            NetDevice::Loopback(dev) => dev.transmit(ty, data, dst),
            NetDevice::Ethernet => todo!(),
        }
    }

    pub fn try_into_loopback(&mut self) -> UtcpResult<&mut LoopbackNetDevice> {
        match self {
            NetDevice::Loopback(dev) => Ok(dev),
            _ => Err(UtcpErr::Net("not a loopback device".into())),
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
    fn transmit(&mut self, ty: u16, data: &[u8], dst: &mut [u8]) -> UtcpResult<()>;
}

// TODO: use a better data structure, e.g. index
#[derive(Debug, Clone)]
pub struct NetDeviceHandler {
    /// Note: Do not use this directly, use `net_device_get` instead.
    /// FIXME: make this more user-friendly
    pub(crate) private: String,
}

impl NetDeviceHandler {
    fn new(private: String) -> Self {
        Self { private }
    }
}

/// Note: Do not use this directly, use `net_device_get` instead.
pub static DEVICES: LazyLock<SkipMap<String, RwLock<NetDevice>>> = LazyLock::new(|| SkipMap::new());

pub fn net_init() -> UtcpResult<()> {
    intr::intr_init()?;
    log::info!("initialized");
    Ok(())
}

pub fn net_run() -> UtcpResult<()> {
    intr::intr_run()?;
    log::info!("opening all devices");

    for ent in DEVICES.iter() {
        let mut dev = ent.value().write().unwrap();
        net_device_open(&mut *dev)?;
    }
    Ok(())
}

pub fn net_shutdown() -> UtcpResult<()> {
    intr::intr_shutdown()?;
    for ent in DEVICES.iter() {
        let mut dev = ent.value().write().unwrap();
        net_device_close(&mut *dev)?;
    }
    log::info!("shutting down");
    Ok(())
}

pub fn net_device_register(dev: NetDevice) -> UtcpResult<NetDeviceHandler> {
    log::debug!("register dev={}, type={:?}", dev.name(), dev.device_type());
    let handler = NetDeviceHandler::new(dev.name().to_string());
    DEVICES.insert(dev.name().to_string(), RwLock::new(dev));
    Ok(handler)
}

pub fn net_device_output(
    dev: &NetDeviceHandler,
    r#type: u16,
    data: &[u8],
    dst: &mut [u8],
) -> UtcpResult<()> {
    let dev = DEVICES.get(&dev.private).unwrap();
    let mut dev = dev.value().write().unwrap();
    if !dev.is_up() {
        return Err(UtcpErr::Net("device not opened".into()));
    }
    if data.len() > dev.mtu() as usize {
        return Err(UtcpErr::Net("data too large".into()));
    }
    dev.transmit(r#type, data, dst)
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

pub fn net_input_handler(dev: &NetDeviceHandler, r#type: u16, data: &[u8]) {
    log::debug!("dev={}, type={}, len={}", dev.private, r#type, data.len());
    log::debug!("data={:?}", data);
}

#[macro_export]
macro_rules! net_device_get {
    ($handler:expr) => {
        DEVICES.get(&$handler.private).unwrap()
    };
}
