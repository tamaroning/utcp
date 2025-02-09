use std::{collections::VecDeque, sync::atomic::AtomicU32};

use bitflags::bitflags;

use crate::{
    driver::{INTR_IRQ_SOFTIRQ, dummy::DummyNetDevice, loopback::LoopbackNetDevice},
    error::{UtcpErr, UtcpResult},
    ip::{self, IpInterface},
    platform::linux::intr,
};

pub const NET_PROTOCOL_TYPE_IP: u16 = 0x0800;
pub const NET_PROTOCOL_TYPE_ARP: u16 = 0x0806;
pub const NET_PROTOCOL_TYPE_IPV6: u16 = 0x86dd;

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

    pub fn name(&self) -> &str {
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

    pub(crate) fn get_interfaces(&self) -> &[NetInterface] {
        match self {
            NetDevice::Dummy(_) => todo!(),
            NetDevice::Loopback(dev) => dev.get_interfaces(),
            NetDevice::Ethernet => todo!(),
        }
    }

    // TODO: remove handler argument. It can be calculated from self.
    pub(crate) fn add_interface(
        &mut self,
        handler: NetDeviceHandler,
        iface: NetInterface,
    ) -> NetInterfaceHandler {
        match self {
            NetDevice::Dummy(_) => todo!(),
            NetDevice::Loopback(dev) => dev.add_interface(handler, iface),
            NetDevice::Ethernet => todo!(),
        }
    }
}

impl<'a> TryFrom<&'a mut NetDevice> for &'a mut LoopbackNetDevice {
    type Error = UtcpErr;

    fn try_from(value: &'a mut NetDevice) -> Result<Self, Self::Error> {
        match value {
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

#[derive(Debug, Clone, Copy)]
pub struct NetDeviceHandler {
    pub(crate) private: usize,
}

impl NetDeviceHandler {
    #[allow(static_mut_refs)]
    fn new(dev: NetDevice) -> Self {
        let index = unsafe {
            let index = DEVICES.len();
            DEVICES.push(dev);
            index
        };
        Self { private: index }
    }
}

/// Note: Do not use this directly, use `net_device_get` instead.
pub static mut DEVICES: Vec<NetDevice> = Vec::new();

pub fn net_init() -> UtcpResult<()> {
    intr::intr_init()?;
    ip::ip_init()?;
    log::info!("initialized");
    Ok(())
}

#[allow(static_mut_refs)]
pub fn net_run() -> UtcpResult<()> {
    intr::intr_run()?;
    log::info!("opening all devices");

    for dev in unsafe { DEVICES.iter_mut() } {
        net_device_open(dev)?;
    }
    Ok(())
}

#[allow(static_mut_refs)]
pub fn net_shutdown() -> UtcpResult<()> {
    intr::intr_shutdown()?;
    for dev in unsafe { DEVICES.iter_mut() } {
        net_device_close(dev)?;
    }
    log::info!("shutting down");
    Ok(())
}

pub fn net_device_register(dev: NetDevice) -> UtcpResult<NetDeviceHandler> {
    log::debug!("register dev={}, type={:?}", dev.name(), dev.device_type());
    let handler = NetDeviceHandler::new(dev);
    Ok(handler)
}

#[allow(static_mut_refs)]
pub fn net_device_output(
    dev: &NetDeviceHandler,
    r#type: u16,
    data: &[u8],
    dst: &mut [u8],
) -> UtcpResult<()> {
    let dev = unsafe { &mut DEVICES[dev.private] };
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

#[allow(static_mut_refs)]
pub fn net_input_handler(dev: &NetDeviceHandler, r#type: u16, data: &[u8]) -> UtcpResult<()> {
    log::debug!("dev={}, type={}, len={}", dev.private, r#type, data.len());
    log::debug!("data={:?}", data);

    // Safety: We know that the current thread is the only one accessing the static variable
    for proto in unsafe { NET_PROTOCOLS.iter_mut() } {
        if proto.ty == r#type {
            // enqueue the packet to the protocol queue
            proto.queue.push_back(NetProtocolQueueEntry {
                dev: dev.clone(),
                data: data.to_vec(),
            });
            intr::intr_raise_irq(INTR_IRQ_SOFTIRQ)?;
            return Ok(());
        }
    }
    // unsupported protocol. drop the packet
    Ok(())
}

#[macro_export]
macro_rules! net_device_get {
    ($handler:expr) => {
        unsafe {
            use $crate::net::DEVICES;
            &DEVICES[$handler.private]
        }
    };
}

#[macro_export]
macro_rules! net_device_get_mut {
    ($handler:expr) => {
        unsafe {
            use $crate::net::DEVICES;
            &mut DEVICES[$handler.private]
        }
    };
}

static mut NET_PROTOCOLS: Vec<NetProtocol> = Vec::new();

pub struct NetProtocol {
    pub ty: u16,
    pub handler: fn(data: &[u8], dev: &NetDeviceHandler),
    queue: VecDeque<NetProtocolQueueEntry>,
}

impl NetProtocol {
    pub fn new(ty: u16, handler: fn(data: &[u8], dev: &NetDeviceHandler)) -> Self {
        Self {
            ty,
            handler,
            queue: VecDeque::new(),
        }
    }
}

pub struct NetProtocolQueueEntry {
    dev: NetDeviceHandler,
    data: Vec<u8>,
}

#[allow(static_mut_refs)]
pub fn net_protocol_register(proto: NetProtocol) {
    log::info!("registered protocol={:?}", proto.ty);
    // Safety: We know that the current thread is the only one accessing the static variable
    unsafe {
        NET_PROTOCOLS.push(proto);
    }
}

#[allow(static_mut_refs)]
pub fn net_softirq_handler() -> UtcpResult<()> {
    for proto in unsafe { NET_PROTOCOLS.iter_mut() } {
        while let Some(entry) = proto.queue.pop_front() {
            (proto.handler)(&entry.data, &entry.dev);
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum NetInterface {
    Ip(IpInterface),
}

impl<'a> TryFrom<&'a NetInterface> for &'a IpInterface {
    type Error = UtcpErr;

    fn try_from(value: &'a NetInterface) -> Result<Self, Self::Error> {
        match value {
            NetInterface::Ip(iface) => Ok(iface),
        }
    }
}

/// Do not use this directly, use `net_device_get_iface` instead.
#[derive(Copy, Clone)]
pub struct NetInterfaceHandler {
    pub(crate) dev: NetDeviceHandler,
    pub(crate) iface_index: usize,
    pub(crate) family: NetInterfaceFamily,
}

#[macro_export]
macro_rules! net_iface_get {
    ($handler:expr) => {
        unsafe {
            use $crate::net::DEVICES;
            &DEVICES[$handler.dev.private].get_interfaces()[$handler.iface_index]
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NetInterfaceFamily {
    Ip,
}

impl NetInterface {
    pub fn family(&self) -> NetInterfaceFamily {
        match self {
            NetInterface::Ip(_) => NetInterfaceFamily::Ip,
        }
    }
}

pub fn net_device_add_iface(handler: NetDeviceHandler, iface: NetInterface) -> UtcpResult<()> {
    let dev = unsafe { &mut DEVICES[handler.private] };

    for iface in dev.get_interfaces() {
        if std::mem::discriminant(iface) == std::mem::discriminant(iface) {
            return Err(UtcpErr::Net(format!(
                "interface already exists, dev={}, family={:?}",
                dev.name(),
                iface.family()
            )));
        }
    }

    dev.add_interface(handler, iface);

    Ok(())
}

pub fn net_device_get_iface(
    dev: &NetDeviceHandler,
    family: NetInterfaceFamily,
) -> Option<&NetInterface> {
    let dev = unsafe { &DEVICES[dev.private] };

    for iface in dev.get_interfaces() {
        if iface.family() == family {
            return Some(iface);
        }
    }
    None
}
