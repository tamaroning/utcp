use crate::{
    error::UtcpResult,
    net::{
        self, DEVICES, NetDevice, NetDeviceFlags, NetDeviceHandler, NetDeviceOps, NetDeviceType,
        net_device_register,
    },
    net_device_get,
    platform::{IRQFlags, linux::intr},
    utils::SmallQueue,
};

use super::INTR_IRQ_BASE;

const LOOPBACK_QUEUE_LIMIT: usize = 16;
const LOOPBACK_IRQ: i32 = INTR_IRQ_BASE + 1;

#[derive(Debug)]
pub struct LoopbackNetDevice {
    name: String,
    flags: NetDeviceFlags,
    pub(self) queue: SmallQueue<(u16, Vec<u8>), LOOPBACK_QUEUE_LIMIT>,
}

impl LoopbackNetDevice {
    pub fn init() -> UtcpResult<NetDeviceHandler> {
        let name = format!("dev{}", net::new_device_index());
        let dev = Self {
            name: name.clone(),
            flags: NetDeviceFlags::empty(),
            queue: SmallQueue::new(),
        };
        let handler = net_device_register(NetDevice::Loopback(dev))?;
        let flags = IRQFlags::from(IRQFlags::SHARED);
        intr::intr_request_irq(LOOPBACK_IRQ, loopback_isr, flags, name, handler.clone())?;
        Ok(handler)
    }
}

impl NetDeviceOps for LoopbackNetDevice {
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
        self.flags.insert(NetDeviceFlags::UP);
        Ok(())
    }

    fn close(&mut self) -> UtcpResult<()> {
        self.flags.remove(NetDeviceFlags::UP);
        Ok(())
    }

    fn transmit(&mut self, ty: u16, data: &[u8], _: &mut [u8]) -> UtcpResult<()> {
        let _ = self.queue.push((ty, data.to_vec()));
        log::debug!(
            "queue pushed (num:{}), dev={}, type={:?}, len={}",
            self.queue.len(),
            self.name,
            NetDeviceType::Loopback,
            data.len()
        );
        intr::intr_raise_irq(LOOPBACK_IRQ)?;
        Ok(())
    }
}

fn loopback_isr(_: i32, handler: NetDeviceHandler) {
    let ent = net_device_get!(&handler);
    let mut guard = ent.value().write().unwrap();
    let dev = &mut *guard;
    let dev = dev.try_into_loopback().unwrap();

    while let Some((ty, data)) = dev.queue.pop_front() {
        net::net_input_handler(&handler, ty, &data);
        log::debug!(
            "queue popped (num:{}), dev={}, type={:?}, len={}",
            dev.queue.len(),
            dev.name,
            NetDeviceType::Loopback,
            data.len()
        );
    }

    //log::debug!("irq={}, dev={:?}", irq, dev);
}
