use crate::{
    error::{UtcpErr, UtcpResult},
    net::{self, NetDeviceFlags, NetDeviceOps},
};

#[derive(Debug)]
pub struct DummyNetDevice {
    name: String,
    flags: NetDeviceFlags,
}

impl DummyNetDevice {
    pub fn new() -> Self {
        let dev = Self {
            name: format!("dev{}", net::new_device_index()),
            flags: NetDeviceFlags::empty(),
        };
        log::debug!("initialized dev={}", dev.name);
        dev
    }
}

impl NetDeviceOps for DummyNetDevice {
    const MTU: u16 = u16::MAX;
    const HEADER_LEN: usize = 0;
    const ADDR_LEN: usize = 0;

    fn is_up(&self) -> bool {
        self.flags.contains(NetDeviceFlags::UP)
    }

    fn open(&mut self) -> UtcpResult<()> {
        log::debug!("opened dev={}", self.name);
        // TODO:
        self.flags.insert(NetDeviceFlags::UP);
        Ok(())
    }

    fn close(&mut self) -> UtcpResult<()> {
        log::debug!("closed dev={}", self.name);
        // TODO:
        Ok(())
    }

    fn transmit(&mut self, data: &[u8], _: &mut [u8]) -> UtcpResult<()> {
        log::debug!("dev={}, type=dummy", self.name);
        log::debug!("{:?}", data);
        Ok(())
    }
}
