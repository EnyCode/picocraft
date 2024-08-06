use embassy_net::tcp::TcpReader;
use embedded_io_async::Read;
use handshake::HandshakePacket;

use crate::read::{ReadExtension, Slice};

pub trait ReadPacket {
    async fn read_packet(socket: &mut Slice) -> Self;
}

pub struct Packet<'a> {
    pub id: i32,
    pub data: Slice<'a>,
}

pub async fn parse_packet<'a>(socket: &mut TcpReader<'_>) -> Packet<'a> {
    let length = socket.read_varint().await;

    let mut buffer = [0; 1024];
    socket.read_exact(&mut buffer).await.unwrap();
    let mut data = Slice::new(&buffer[..length as usize]);
    let id = data.read_varint().await;

    Packet { id, data }
}

pub mod handshake {
    use embassy_net::tcp::TcpReader;
    use heapless::String;

    use crate::{
        net::State,
        read::{ReadExtension, Slice},
    };

    use super::ReadPacket;

    pub struct HandshakePacket {
        pub protocol_version: i32,
        pub server_address: String<15>,
        pub server_port: u16,
        pub next_state: State,
    }

    impl ReadPacket for HandshakePacket {
        async fn read_packet(socket: &mut Slice<'_>) -> Self {
            HandshakePacket {
                protocol_version: socket.read_varint().await,
                server_address: socket.read_string().await,
                server_port: socket.read_u16().await,
                next_state: match socket.read_varint().await {
                    1 => State::Status,
                    2 => State::Login,
                    3 => State::Transfer,
                    any => State::Custom(any),
                },
            }
        }
    }
}
