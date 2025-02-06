use crate::{
    error::UtcpResult,
    net::{self, NET_PROTOCOL_TYPE_IP, NetDeviceHandler, NetProtocol},
};

fn ip_input(data: &[u8], dev: &NetDeviceHandler) {
    log::debug!("data={:?}", data);
}

pub fn ip_init() -> UtcpResult<()> {
    net::net_protocol_register(NetProtocol::new(NET_PROTOCOL_TYPE_IP, ip_input));
    log::info!("initialized");
    Ok(())
}
