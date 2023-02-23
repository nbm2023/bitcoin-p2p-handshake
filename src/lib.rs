use bitcoin::consensus::serialize;
use bitcoin_hashes::hex::ToHex;
// use bitcoin::network::message::{NetworkMessage, RawNetworkMessage};
// use bitcoin::network::message_network::VersionMessage;
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

const PROTOCOL_VERSION: i32 = 70015;
// 0xd9b4bef9 is the magic for "main" Bitcoin network
const BTC_MAIN_MAGIC: u32 = 3652501241;

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
    // 4	version	int32_t	Identifies protocol version being used by the node
    version: i32,
    // 8	services	uint64_t	bitfield of features to be enabled for this connection
    services: u64,
    // 8	timestamp	int64_t	standard UNIX timestamp in seconds
    timestamp: i64,
    // 26	addr_recv	net_addr	The network address of the node receiving this message
    addr_recv: NetworkAddress,

    // Fields below require version ≥ 106
    // 26	addr_from	net_addr	Field can be ignored. This used to be the network address of the node emitting this message, but most P2P implementations send 26 dummy bytes. The "services" field of the address would also be redundant with the second field of the version message.
    addr_from: NetworkAddress,
    // 8	nonce	uint64_t	Node random nonce, randomly generated every time a version packet is sent. This nonce is used to detect connections to self.
    nonce: u64,
    // ?	user_agent	var_str	User Agent (0x00 if string is 0 bytes long)
    user_agent: String,
    // 4	start_height	int32_t	The last block received by the emitting node
    start_height: i32,

    // Fields below require version ≥ 70001
    // 1	relay	bool	Whether the remote peer should announce relayed transactions or not, see BIP 0037
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
        version: PROTOCOL_VERSION,
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
    // 4	magic	uint32_t	Magic value indicating message origin network, and used to seek to next message when stream state is unknown
    buf.extend_from_slice(&BTC_MAIN_MAGIC.to_le_bytes());
    // 12	command	char[12]	ASCII string identifying the packet content, NULL padded (non-NULL padding results in packet rejected)
    buf.extend_from_slice(&"version".as_bytes());
    buf.resize(12, 0);
    // 4	length	uint32_t	Length of payload in number of bytes
    buf.extend_from_slice(&(version_message.to_bytes().len() as u32).to_le_bytes());
    // 4	checksum	uint32_t	First 4 bytes of sha256(sha256(payload))
    let version_message_bytes = version_message.to_bytes();
    let message_sha256 = sha256d::Hash::hash(&version_message_bytes);
    let first_four_bytes_of_message_sha256: [u8; 4] = [message_sha256[0], message_sha256[1], message_sha256[2], message_sha256[3]];
    buf.extend_from_slice(&first_four_bytes_of_message_sha256);
    //  ?	payload	uchar[]	The actual data
    buf.extend_from_slice(&version_message_bytes);

    let res = stream.write_all(&buf);
    println!("Write res: {:?}", res);

    loop {
        let mut reader = BufReader::new(&stream);
        let mut buf = [0u8; 1024];
        let res = reader.read(&mut buf).unwrap();
        // println!("Read res: {:?}", buf); // ---> Always empty
        if res != 0 {break;};
    }

}
