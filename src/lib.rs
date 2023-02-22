use bitcoin::consensus::serialize;
use bitcoin_hashes::hex::ToHex;
use std::io::Read;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use bitcoin_hashes::{sha256d, Hash};


#[test]
fn test_handshake() {
    handshake();
}

const PROTOCOL_VERSION: u32 = 70015;
#[derive(Debug)]
struct NetworkAddress {
    services: u64,
    ip: [u8; 16],
    port: u16,
}
impl NetworkAddress {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(26);
        bytes.extend_from_slice(&self.services.to_le_bytes());
        // IP and Port need to be Big Endian. The IP is already passed as big endian.
        bytes.extend_from_slice(&self.ip);
        bytes.extend_from_slice(&self.port.to_be_bytes());
        bytes
    }
}
#[derive(Debug)]
struct VersionMessage {
    version: u32,
    services: u64,
    timestamp: i64,
    addr_recv: NetworkAddress,
    addr_from: NetworkAddress,
    nonce: u64,
    user_agent: String,
    start_height: i32,
    relay: bool,
}
impl VersionMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(85 + self.user_agent.len());
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.services.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.addr_recv.to_bytes());
        bytes.extend_from_slice(&self.addr_from.to_bytes());
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        bytes.extend_from_slice(&(self.user_agent.len() as u8).to_le_bytes());
        bytes.extend_from_slice(self.user_agent.as_bytes());
        bytes.extend_from_slice(&self.start_height.to_le_bytes());
        bytes.extend_from_slice(&(self.relay as u8).to_le_bytes());
        bytes
    }
}

fn handshake() {
    // Connect to a Bitcoin node on the main network
    let mut stream = TcpStream::connect("seed.bitcoin.sipa.be:8333").unwrap();

    // Big endian represenation of the "seed.bitcoin.sipa.be" address
    let ipv6_address: std::net::Ipv6Addr = "::ffff:185.49.141.1".parse().unwrap();
    let ipv4_mapped_ipv6 = ipv6_address.octets();
    println!("ipv4_mapped_ipv6: {:?}", ipv4_mapped_ipv6);

    // Send a version message to the node
    let version_message = VersionMessage {
        version: 70015,
        services: 1,
        timestamp: chrono::Utc::now().timestamp() as i64,
        addr_recv: NetworkAddress {
            services: 1,
            ip: ipv4_mapped_ipv6,
            port: 8333,
        },
        addr_from: NetworkAddress {
            services: 0,
            ip: [0; 16],
            port: 0,
        },
        nonce: 0,
        user_agent: "/bitcoin-rs:0.1.0/".to_string(),
        start_height: 0,
        relay: false,
    };

    let mut buf = Vec::new();
    // 0xd9b4bef9 magic for "main" network
    buf.extend_from_slice(&(3652501241u32).to_le_bytes());
    buf.extend_from_slice(&"version".as_bytes());
    buf.resize(12, 0);
    buf.extend_from_slice(&(version_message.to_bytes().len() as u32).to_le_bytes());
    let version_message_bytes = version_message.to_bytes();
    let message_sha256 = sha256d::Hash::hash(&version_message_bytes);
    let first_four_bytes_of_message_sha256: [u8; 4] = [message_sha256[0], message_sha256[1], message_sha256[2], message_sha256[3]];
    buf.extend_from_slice(&first_four_bytes_of_message_sha256);
    buf.extend_from_slice(&version_message_bytes);

    let res = stream.write_all(&buf);
    println!("Write res: {:?}", res);

    let mut reader = BufReader::new(&stream);
    let mut buf = [0u8; 1024];
    let _ = reader.read(&mut buf).unwrap();
    println!("Read res: {:?}", buf); // ---> Always empty
}
