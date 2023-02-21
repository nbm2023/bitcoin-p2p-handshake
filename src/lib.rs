use bitcoin::consensus::serialize;
// use bitcoin::network::message::{NetworkMessage, RawNetworkMessage};
// use bitcoin::network::message_network::VersionMessage;
use std::io::Read;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

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
        bytes.extend_from_slice(&[0; 12]);
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
    // // Connect to a Bitcoin node on the main network
    let mut stream = TcpStream::connect("seed.bitcoin.sipa.be:8333").unwrap();

    // Send a version message to the node
    let version_message = VersionMessage {
        version: 80015,
        services: 0,
        timestamp: chrono::Utc::now().timestamp() as i64,
        addr_recv: NetworkAddress {
            services: 0,
            ip: [0; 16],
            port: 0,
        },
        addr_from: NetworkAddress {
            services: 0,
            ip: [0; 16],
            port: 0,
        },
        nonce: 0,
        user_agent: "/rust-bitcoin:0.1.0/".to_string(),
        start_height: 0,
        relay: false,
    };

    let mut buf = Vec::new();
    buf.extend_from_slice(b"0xD9B4BEF9");
    buf.extend_from_slice(&"version".as_bytes());
    buf.resize(12, 0);
    buf.extend_from_slice(&(version_message.to_bytes().len() as u32).to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&version_message.to_bytes());
    let res = stream.write_all(&buf);
    println!("Res: {:?}", res);

    let mut reader = BufReader::new(&stream);

    let mut buf = [0u8; 1024];
    let _ = reader.read(&mut buf).unwrap();
    println!("Read res: {:?}", buf); // ---> Always empty

    println!("Why is the buf always empty :/ ");
}
