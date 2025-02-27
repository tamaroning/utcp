use crate::{
    error::UtcpResult,
    net::{self, NetDevice, NetDeviceFlags, NetDeviceHandler, NetDeviceOps, net_device_register},
    platform::{IRQFlags, linux::intr},
};

use super::INTR_IRQ_BASE;

const DUMMY_IRQ: i32 = INTR_IRQ_BASE;

#[derive(Debug)]
pub struct DummyNetDevice {
    name: String,
    flags: NetDeviceFlags,
}

impl DummyNetDevice {
    pub fn init() -> UtcpResult<NetDeviceHandler> {
        let name = format!("dev{}", net::new_device_index());
        let dev = Self {
            name: name.clone(),
            flags: NetDeviceFlags::empty(),
        };
        let handler = net_device_register(NetDevice::Dummy(dev))?;
        let flags = IRQFlags::SHARED;
        intr::intr_request_irq(DUMMY_IRQ, dummy_isr, flags, name, handler.clone())?;
        Ok(handler)
    }
}

impl NetDeviceOps for DummyNetDevice {
    const MTU: u16 = u16::MAX;
    const HEADER_LEN: usize = 0;
    const ADDR_LEN: usize = 0;

    fn name(&self) -> &str {
        &self.name
    }

    fn is_up(&self) -> bool {
        self.flags.contains(NetDeviceFlags::UP)
    }

    fn open(&mut self) -> UtcpResult<()> {
        // TODO:
        self.flags.insert(NetDeviceFlags::UP);
        Ok(())
    }

    fn close(&mut self) -> UtcpResult<()> {
        // TODO:
        self.flags.remove(NetDeviceFlags::UP);
        Ok(())
    }

    fn transmit(&mut self, ty: u16, data: &[u8], _: &mut [u8]) -> UtcpResult<()> {
        log::debug!("dev={}, type=dummy", self.name);
        log::debug!("data_type={}, data={:?}", ty, data);
        intr::intr_raise_irq(DUMMY_IRQ)?;
        Ok(())
    }
}

fn dummy_isr(irq: i32, dev: NetDeviceHandler) {
    log::debug!("irq={}, dev={:?}", irq, dev);
}
