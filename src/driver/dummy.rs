use crate::{error::UtcpResult, net};

pub struct DummyNetDevice {
    name: String,
}

impl DummyNetDevice {
    pub fn new() -> Self {
        let dev = Self {
            name: format!("dev{}", net::new_device_index()),
        };
        log::debug!("initialized dev={}", dev.name);
        dev
    }
}

impl net::NetDevice for DummyNetDevice {
    const DEVICE_TYPE: net::NetDeviceType = net::NetDeviceType::Dummy;
    const MTU: u16 = net::DUMMY_MTU;
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

    fn transmit(&mut self, data: &[u8], _: &mut [u8]) -> UtcpResult<()> {
        log::debug!("dev={}, type={:?}", self.name, Self::DEVICE_TYPE);
        log::debug!("{:?}", data);
        Ok(())
    }
}
