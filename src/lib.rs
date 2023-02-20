use bitcoin::consensus::serialize;
use bitcoin::network::message::{NetworkMessage, RawNetworkMessage};
use bitcoin::network::message_network::VersionMessage;
use std::io::Read;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[test]
fn test_handshake() {
    handshake();
}

fn handshake() {
    // Connect to a Bitcoin node on the main network
    let stream = TcpStream::connect("seed.bitcoin.sipa.be:8333").unwrap();
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    // Send a version message to the node
    let version_message = VersionMessage {
        version: 70015,
        services: 1.into(),
        timestamp: chrono::Utc::now().timestamp() as i64,
        receiver: bitcoin::network::address::Address::new(
            &SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            0.into(),
        ),
        sender: bitcoin::network::address::Address::new(
            &SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            0.into(),
        ),
        nonce: 0,
        user_agent: "/rust-bitcoin:0.1.0/".to_string(),
        start_height: 0,
        relay: false,
    };

    let network_message = RawNetworkMessage {
        magic: 0,
        payload: NetworkMessage::Version(version_message),
    };

    writer
        .write_all(serialize(&network_message).as_slice())
        .unwrap();
    writer.flush().unwrap();

    // Wait for a version message response from the node
    loop {
        let mut buf = [0u8; 1024];
        let res = reader.read(&mut buf).unwrap();
        println!("Res: {:?}", res); // ---> Always 0
    }

    println!("Handshake completed successfully");
}

fn from_reader<R: std::io::Read>(reader: &mut R) -> Result<NetworkMessage, std::io::Error> {
    // Read the first 4 bytes to get the message magic number.
    let mut magic_bytes = [0u8; 4];
    reader.read_exact(&mut magic_bytes)?;

    // Read the next 12 bytes to get the message command.
    let mut command_bytes = [0u8; 12];
    reader.read_exact(&mut command_bytes)?;

    // Parse the command bytes into a string and trim the null characters.
    let command = std::str::from_utf8(&command_bytes)
        .unwrap()
        .trim_end_matches('\0')
        .to_owned();

    // Read the next 4 bytes to get the payload length.
    let mut payload_length_bytes = [0u8; 4];
    reader.read_exact(&mut payload_length_bytes)?;

    // Parse the payload length bytes into a u32 integer.
    let payload_length = u32::from_le_bytes(payload_length_bytes);

    // Read the payload bytes.
    let mut payload_bytes = vec![0u8; payload_length as usize];
    reader.read_exact(&mut payload_bytes)?;

    // Construct the network message.
    let message = match command.as_str() {
        "version" => NetworkMessage::Verack, //NetworkMessage::Version(VersionMessage::from_payload(&payload_bytes)?),
        "verack" => NetworkMessage::Verack,
        // Handle other message types here.
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid command",
            ))
        }
    };

    Ok(message)
}
