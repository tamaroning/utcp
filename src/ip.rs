use crate::{
    error::UtcpResult,
    net::{self, NET_PROTOCOL_TYPE_IP, NetDeviceHandler, NetProtocol},
    utils,
};

/*

struct ip_hdr {
    uint8_t vhl;
    uint8_t tos;
    uint16_t total;
    uint16_t id;
    uint16_t offset;
    uint8_t ttl;
    uint8_t protocol;
    uint16_t sum;
    ip_addr_t src;
    ip_addr_t dst;
    uint8_t options[];
};
*/

#[repr(C)]
pub struct IpHeader {
    /// Version and header length
    vhl: u8,
    /// Type of service
    tos: u8,
    /// Total length (header + data)
    total: u16,
    /// Identification
    id: u16,
    /// Fragment offset (13 bits) + flags (3 bits)
    offset: u16,
    /// Time to live
    ttl: u8,
    /// Protocol
    protocol: u8,
    /// Header checksum
    sum: u16,
    /// Source address
    src: u32,
    /// Destination address
    dst: u32,
    /// Options
    options: [u8; 0],
}

// static assert IP header size
const _: [(); std::mem::size_of::<IpHeader>()] = [(); 20];

impl IpHeader {
    pub fn new(data: &[u8]) -> Option<&IpHeader> {
        if data.len() < std::mem::size_of::<IpHeader>() {
            return None;
        }
        let ip_hdr = unsafe { &*(data.as_ptr() as *const IpHeader) };
        Some(ip_hdr)
    }

    pub fn version(&self) -> u8 {
        self.vhl >> 4
    }

    pub fn header_len(&self) -> u8 {
        self.vhl & 0x0f
    }

    pub fn tos(&self) -> u8 {
        self.tos
    }

    pub fn total(&self) -> u16 {
        u16::from_be(self.total)
    }

    pub fn id(&self) -> u16 {
        u16::from_be(self.id)
    }

    pub fn offset(&self) -> u16 {
        u16::from_be(self.offset) & 0x1fff
    }

    pub fn flags(&self) -> u8 {
        ((u16::from_be(self.offset) & 0xe000) >> 13) as u8
    }

    pub fn dont_fragment(&self) -> bool {
        self.flags() & 0x02 != 0
    }

    pub fn more_fragments(&self) -> bool {
        self.flags() & 0x01 != 0
    }

    pub fn ttl(&self) -> u8 {
        self.ttl
    }

    pub fn protocol(&self) -> u8 {
        self.protocol
    }

    pub fn sum(&self) -> u16 {
        u16::from_be(self.sum)
    }

    pub fn src(&self) -> IpAddress {
        IpAddress(u32::from_be(self.src))
    }

    pub fn dst(&self) -> IpAddress {
        IpAddress(u32::from_be(self.dst))
    }
}

impl std::fmt::Debug for IpHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let version = self.version();
        let header_len = self.header_len();
        let tos = self.tos();
        let total = self.total();
        let id = self.id();
        let offset = self.offset();
        let df = self.dont_fragment();
        let mf = self.more_fragments();
        let ttl = self.ttl();
        let protocol = self.protocol();
        let sum = self.sum();
        let src = self.src();
        let dst = self.dst();
        write!(
            f,
            "version={}, header_len={}, tos={}, total={}, id={}, offset={} df={}, mf={}, ttl={}, protocol={}, sum={}, src={}, dst={}",
            version, header_len, tos, total, id, offset, df, mf, ttl, protocol, sum, src, dst
        )
    }
}

/// IPv4 address
pub struct IpAddress(u32);

impl std::fmt::Display for IpAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let octets = self.0.to_be_bytes();
        write!(f, "{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
    }
}

impl std::fmt::Debug for IpAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let octets = self.0.to_be_bytes();
        write!(f, "{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
    }
}

/// 0.0.0.0
const IP_ADDR_ANY: IpAddress = IpAddress(0);
/// 255.255.255.255
const IP_ADDR_BROADCAST: IpAddress = IpAddress(0xffffffff);

fn ip_input(data: &[u8], _: &NetDeviceHandler) {
    log::debug!("data={:?}", data);

    let Some(ip_hdr) = IpHeader::new(data) else {
        log::error!("IP header is too short");
        return;
    };
    if data.len() < ip_hdr.total() as usize {
        log::error!("IP datagram is too short");
        return;
    }
    if ip_hdr.version() != 4 {
        log::error!("IPv4 is only supported");
        return;
    }
    // Check checksum
    let actual = utils::checksum16(data, 0);
    if actual != 0 {
        log::error!("checksum mismatch: expected=0, actual=0x{:04x}", actual);
        return;
    }

    // do not support fragmented packets for now
    if ip_hdr.more_fragments() || ip_hdr.offset() != 0 {
        log::error!("fragmented packets are not supported");
        return;
    }

    dbg!(ip_hdr);
}

pub fn ip_init() -> UtcpResult<()> {
    net::net_protocol_register(NetProtocol::new(NET_PROTOCOL_TYPE_IP, ip_input));
    log::info!("initialized");
    Ok(())
}
