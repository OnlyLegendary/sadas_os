#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::vec::Vec;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkDeviceBackend {
    VirtioNet,
    E1000e,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MacAddress(pub [u8; 6]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ipv4Address(pub [u8; 4]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeviceInfo {
    pub backend: NetworkDeviceBackend,
    pub mac: MacAddress,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetError {
    BufferTooSmall,
    InvalidPacket,
    Unsupported,
    Device,
}

pub trait NetworkDevice {
    fn info(&self) -> DeviceInfo;
    fn send_frame(&mut self, frame: &[u8]) -> Result<(), NetError>;
    fn recv_frame(&mut self, out: &mut [u8]) -> Result<usize, NetError>;
}

pub const ETHERTYPE_ARP: u16 = 0x0806;
pub const ETHERTYPE_IPV4: u16 = 0x0800;
pub const IPPROTO_ICMP: u8 = 1;
pub const IPPROTO_UDP: u8 = 17;
pub const IPPROTO_TCP: u8 = 6;

pub struct EthernetFrame<'a> {
    pub dst: MacAddress,
    pub src: MacAddress,
    pub ethertype: u16,
    pub payload: &'a [u8],
}

impl<'a> EthernetFrame<'a> {
    pub fn encode(&self, out: &mut [u8]) -> Result<usize, NetError> {
        let total = 14 + self.payload.len();
        if out.len() < total {
            return Err(NetError::BufferTooSmall);
        }
        out[..6].copy_from_slice(&self.dst.0);
        out[6..12].copy_from_slice(&self.src.0);
        out[12..14].copy_from_slice(&self.ethertype.to_be_bytes());
        out[14..total].copy_from_slice(self.payload);
        Ok(total)
    }

    pub fn parse(buf: &'a [u8]) -> Result<Self, NetError> {
        if buf.len() < 14 {
            return Err(NetError::InvalidPacket);
        }
        let mut dst = [0u8; 6];
        let mut src = [0u8; 6];
        dst.copy_from_slice(&buf[..6]);
        src.copy_from_slice(&buf[6..12]);
        let ethertype = u16::from_be_bytes([buf[12], buf[13]]);
        Ok(Self {
            dst: MacAddress(dst),
            src: MacAddress(src),
            ethertype,
            payload: &buf[14..],
        })
    }
}

pub struct ArpPacket {
    pub operation: u16,
    pub sender_mac: MacAddress,
    pub sender_ip: Ipv4Address,
    pub target_mac: MacAddress,
    pub target_ip: Ipv4Address,
}

impl ArpPacket {
    pub fn encode(&self, out: &mut [u8]) -> Result<usize, NetError> {
        if out.len() < 28 {
            return Err(NetError::BufferTooSmall);
        }
        out[0..2].copy_from_slice(&1u16.to_be_bytes());
        out[2..4].copy_from_slice(&ETHERTYPE_IPV4.to_be_bytes());
        out[4] = 6;
        out[5] = 4;
        out[6..8].copy_from_slice(&self.operation.to_be_bytes());
        out[8..14].copy_from_slice(&self.sender_mac.0);
        out[14..18].copy_from_slice(&self.sender_ip.0);
        out[18..24].copy_from_slice(&self.target_mac.0);
        out[24..28].copy_from_slice(&self.target_ip.0);
        Ok(28)
    }
}

pub struct Ipv4Packet<'a> {
    pub src: Ipv4Address,
    pub dst: Ipv4Address,
    pub protocol: u8,
    pub payload: &'a [u8],
}

impl<'a> Ipv4Packet<'a> {
    pub fn encode(&self, out: &mut [u8]) -> Result<usize, NetError> {
        let total = 20 + self.payload.len();
        if out.len() < total {
            return Err(NetError::BufferTooSmall);
        }
        out[0] = 0x45;
        out[1] = 0;
        out[2..4].copy_from_slice(&(total as u16).to_be_bytes());
        out[4..6].copy_from_slice(&0u16.to_be_bytes());
        out[6..8].copy_from_slice(&0u16.to_be_bytes());
        out[8] = 64;
        out[9] = self.protocol;
        out[10..12].copy_from_slice(&0u16.to_be_bytes());
        out[12..16].copy_from_slice(&self.src.0);
        out[16..20].copy_from_slice(&self.dst.0);
        let csum = checksum16(&out[..20]);
        out[10..12].copy_from_slice(&csum.to_be_bytes());
        out[20..total].copy_from_slice(self.payload);
        Ok(total)
    }
}

pub struct IcmpEcho {
    pub id: u16,
    pub seq: u16,
}

impl IcmpEcho {
    pub fn encode_request(&self, payload: &[u8], out: &mut [u8]) -> Result<usize, NetError> {
        let total = 8 + payload.len();
        if out.len() < total {
            return Err(NetError::BufferTooSmall);
        }
        out[0] = 8;
        out[1] = 0;
        out[2..4].copy_from_slice(&0u16.to_be_bytes());
        out[4..6].copy_from_slice(&self.id.to_be_bytes());
        out[6..8].copy_from_slice(&self.seq.to_be_bytes());
        out[8..total].copy_from_slice(payload);
        let csum = checksum16(&out[..total]);
        out[2..4].copy_from_slice(&csum.to_be_bytes());
        Ok(total)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DhcpLease {
    pub address: Ipv4Address,
    pub router: Ipv4Address,
    pub dns: Ipv4Address,
}

pub fn checksum16(bytes: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut i = 0;
    while i + 1 < bytes.len() {
        sum += u16::from_be_bytes([bytes[i], bytes[i + 1]]) as u32;
        i += 2;
    }
    if i < bytes.len() {
        sum += (bytes[i] as u32) << 8;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

#[cfg(feature = "std")]
pub fn dhcp_discover_stub<D: NetworkDevice>(device: &D) -> DhcpLease {
    let _ = device.info();
    DhcpLease {
        address: Ipv4Address([10, 0, 2, 15]),
        router: Ipv4Address([10, 0, 2, 2]),
        dns: Ipv4Address([1, 1, 1, 1]),
    }
}

#[cfg(feature = "std")]
pub fn http_get(host: &str, path: &str) -> Result<String, NetError> {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let mut stream = TcpStream::connect((host, 80)).map_err(|_| NetError::Device)?;
    let req = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: sadas-httpget/0.1\r\n\r\n",
        path, host
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|_| NetError::Device)?;

    let mut out = String::new();
    stream
        .read_to_string(&mut out)
        .map_err(|_| NetError::Device)?;
    Ok(out)
}

#[cfg(feature = "std")]
pub struct LoopbackNic {
    pub backend: NetworkDeviceBackend,
    pub mac: MacAddress,
    rx: Vec<u8>,
}

#[cfg(feature = "std")]
impl LoopbackNic {
    pub fn virtio_default() -> Self {
        Self {
            backend: NetworkDeviceBackend::VirtioNet,
            mac: MacAddress([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]),
            rx: Vec::new(),
        }
    }

    pub fn e1000e_default() -> Self {
        Self {
            backend: NetworkDeviceBackend::E1000e,
            mac: MacAddress([0x00, 0x1b, 0x21, 0xaa, 0xbb, 0xcc]),
            rx: Vec::new(),
        }
    }
}

#[cfg(feature = "std")]
impl NetworkDevice for LoopbackNic {
    fn info(&self) -> DeviceInfo {
        DeviceInfo {
            backend: self.backend,
            mac: self.mac,
        }
    }

    fn send_frame(&mut self, frame: &[u8]) -> Result<(), NetError> {
        self.rx.clear();
        self.rx.extend_from_slice(frame);
        Ok(())
    }

    fn recv_frame(&mut self, out: &mut [u8]) -> Result<usize, NetError> {
        if self.rx.is_empty() {
            return Err(NetError::Unsupported);
        }
        if out.len() < self.rx.len() {
            return Err(NetError::BufferTooSmall);
        }
        out[..self.rx.len()].copy_from_slice(&self.rx);
        let len = self.rx.len();
        self.rx.clear();
        Ok(len)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn encodes_icmp_checksum() {
        let echo = IcmpEcho { id: 7, seq: 1 };
        let mut out = [0u8; 64];
        let n = echo.encode_request(b"ping", &mut out).expect("encode");
        assert_eq!(n, 12);
        assert_ne!(u16::from_be_bytes([out[2], out[3]]), 0);
    }

    #[test]
    fn ethernet_roundtrip() {
        let payload = b"arp";
        let frame = EthernetFrame {
            dst: MacAddress([0xff; 6]),
            src: MacAddress([1, 2, 3, 4, 5, 6]),
            ethertype: ETHERTYPE_ARP,
            payload,
        };
        let mut out = [0u8; 32];
        let n = frame.encode(&mut out).expect("encode");
        let parsed = EthernetFrame::parse(&out[..n]).expect("parse");
        assert_eq!(parsed.ethertype, ETHERTYPE_ARP);
        assert_eq!(parsed.payload, payload);
    }

    #[test]
    fn loopback_send_recv() {
        let mut nic = LoopbackNic::virtio_default();
        nic.send_frame(b"frame").expect("send");
        let mut out = [0u8; 16];
        let n = nic.recv_frame(&mut out).expect("recv");
        assert_eq!(&out[..n], b"frame");
    }
}
