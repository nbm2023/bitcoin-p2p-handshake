#![allow(dead_code)]

use bitcoin_hashes::{sha256d, Hash};
use tokio::io::AsyncWriteExt;
use tokio::{
    io::{split, AsyncReadExt, ReadHalf},
    net::TcpStream,
};

#[derive(Debug)]
struct NetworkAddress {
    ip: [u8; 16],
    services: u64,
    port: u16,
}

impl NetworkAddress {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(26);
        bytes.extend_from_slice(&self.ip);
        bytes.extend_from_slice(&self.services.to_le_bytes());
        bytes.extend_from_slice(&self.port.to_be_bytes());
        bytes
    }
}

#[derive(Debug)]
struct VersionMessage {
    version: i32,
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

const PROTOCOL_VERSION: i32 = 70002;
const BTC_MAIN_MAGIC: [u8; 4] = [0xf9, 0xbe, 0xb4, 0xd9];
const TO_ADDR: &str = "seed.bitcoin.sipa.be";
const PORT: u16 = 8333;

async fn handshake() -> Result<(), Box<dyn std::error::Error>> {
    let to_addr = format!("{}:{}", TO_ADDR, PORT);
    let stream = TcpStream::connect(to_addr).await?;
    let (mut reader, mut writer): (ReadHalf<_>, _) = split(stream);

    let version_message = VersionMessage {
        version: PROTOCOL_VERSION,
        services: 1,
        timestamp: chrono::Utc::now().timestamp(),
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
        user_agent: "".to_string(),
        start_height: 0,
        relay: false,
    };

    let version_message_bytes = version_message.to_bytes();
    let payload_size = version_message_bytes.len() as u32;
    let mut payload = Vec::with_capacity(24 + payload_size as usize);
    payload.extend_from_slice(&BTC_MAIN_MAGIC);
    payload.extend_from_slice("version\0\0\0\0\0".as_bytes());
    payload.extend_from_slice(&payload_size.to_le_bytes());
    payload.extend_from_slice(&sha256d::Hash::hash(&version_message_bytes)[..4]);
    payload.extend_from_slice(&version_message_bytes);
    println!("version message payload for sending: {:?}", payload);

    writer.write_all(&payload).await?;

    let mut responses = [0; 2084];
    let mut bytes_read = 0;

    loop {
        // Read the remote peer's version message and reply with our own
        let current_response = &mut responses[bytes_read..];
        match reader.read(current_response).await {
            Ok(0) => {
                return Err("Remote peer disconnected without sending version message".into());
            }
            Ok(n) => {
                bytes_read += n;
                println!("current_response bytes: {:?}", &current_response[..n]);
                let command_bytes = &current_response[4..16];
                let command = std::str::from_utf8(command_bytes)
                    .unwrap()
                    .trim_end_matches('\0')
                    .to_owned();

                println!("The returned command is: {:?}", command);
                if command == "version" {
                    assert!(validate_message(&current_response[..n]));

                    let verack_message_bytes = Vec::new();
                    let payload_size = 0u32;
                    let mut payload = Vec::with_capacity(24 + payload_size as usize);
                    payload.extend_from_slice(&BTC_MAIN_MAGIC);
                    payload.extend_from_slice("verack\0\0\0\0\0\0".as_bytes());
                    payload.extend_from_slice(&payload_size.to_le_bytes());
                    payload.extend_from_slice(&sha256d::Hash::hash(&verack_message_bytes)[..4]);
                    println!("verack response payload: {:?}", payload);

                    writer.write_all(&payload).await?;

                    return Ok(());
                } else if command == "verack" {
                    println!("Never received this message for some reason :/");
                } else {
                    return Err("Remote peer disconnected without sending version message".into());
                }
            }
            Err(_) => {
                return Err("Unexpected error connectecting peer".into());
            }
        }
    }
}

fn validate_message(message: &[u8]) -> bool {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&message[16..20]);
    let payload_length = u32::from_le_bytes(bytes);
    let payload_end = 24 + payload_length;
    let payload = &message[24..payload_end as usize];
    let checksum = &message[20..24];

    let hash = &sha256d::Hash::hash(payload);
    println!("message: {:?}", &message);
    println!("hash: {:?}", &hash[..4]);
    println!("checksum: {:?}", &checksum);

    &hash[..4] == checksum
}

#[tokio::test]
async fn test_handshake() {
    assert!(handshake().await.is_ok());
}
