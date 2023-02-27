use bitcoin::{
    consensus::{deserialize, encode::serialize},
    network::{
        constants::ServiceFlags,
        message::{NetworkMessage, RawNetworkMessage},
        message_network::VersionMessage,
    },
};
use chrono::Utc;
use std::{
    io::{Error, ErrorKind},
    net::{IpAddr, Ipv4Addr, SocketAddr},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use serde::Deserialize;

const BTC_MAIN_MAGIC: [u8; 4] = [0xf9, 0xbe, 0xb4, 0xd9];
const TO_ADDR: &str = "seed.bitcoin.sipa.be";
const PORT: u16 = 8333;
const PROTOCOL_VERSION: u32 = 70015;

struct BitcoinPeer {
    socket_addr: SocketAddr,
    stream: TcpStream,
}

impl BitcoinPeer {
    async fn connect(socket_addr: SocketAddr) -> Result<Self, Error> {
        let stream = TcpStream::connect(socket_addr).await?;
        Ok(Self {
            socket_addr,
            stream,
        })
    }

    async fn send_message(&mut self, message: NetworkMessage) -> Result<(), Error> {
        let raw_message = RawNetworkMessage {
            magic: u32::from_le_bytes(BTC_MAIN_MAGIC),
            payload: message,
        };
        let bytes = serialize(&raw_message);
        self.stream.write_all(&bytes).await?;
        Ok(())
    }

    async fn read_message(&mut self) -> Result<NetworkMessage, Error> {
        let mut buffer = [0u8; 1024];
        self.stream.read(&mut buffer).await?;
        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&buffer[16..20]);
        let length = u32::from_le_bytes(length_bytes);
        let raw_message =
        match deserialize::<RawNetworkMessage>(&buffer[..24 + length as usize]) {
            Ok(msg) => msg,
            Err(_) => return Err(Error::new(
                // One reason could be an invalid checksum validation performed in the deserialization
                ErrorKind::InvalidData,
                "Failed deserializing message",
            )),
        };
        Ok(raw_message.payload)
    }

    async fn send_version(&mut self) -> Result<(), Error> {
        let version_message = VersionMessage {
            version: PROTOCOL_VERSION,
            services: ServiceFlags::NONE,
            timestamp: Utc::now().timestamp() as i64,
            receiver: bitcoin::network::Address::new(
                &SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
                0.into(),
            ),
            sender: bitcoin::network::Address::new(
                &SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
                0.into(),
            ),
            nonce: 0,
            user_agent: "my-node".to_owned(),
            start_height: 0,
            relay: false,
        };
        self.send_message(NetworkMessage::Version(version_message))
            .await?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::net::ToSocketAddrs;

    #[tokio::test]
    async fn test_handshake() -> Result<(), Error> {
        let socket_addr = format!("{}:{}", TO_ADDR, PORT)
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();

        let mut peer = BitcoinPeer::connect(socket_addr).await?;
        peer.send_version().await?;
        let mut message = peer.read_message().await?;
        match message {
            NetworkMessage::Version(version_message) => {
                println!("Version Message Returned: {:?}", version_message);

                let verack_message = NetworkMessage::Verack;
                peer.send_message(verack_message).await?;

                message = peer.read_message().await?;
                assert_eq!(message, NetworkMessage::Verack);

                Ok(())
            }
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "Expected Version message",
            )),
        }
    }
}
