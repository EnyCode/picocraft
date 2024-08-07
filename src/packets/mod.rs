use alloc::{boxed::Box, vec::Vec};
use embassy_net::tcp::TcpReader;
use embassy_time::Timer;
use embedded_io_async::Read;
use handshake::HandshakePacket;

use crate::read::{ReadExtension, Slice};

pub mod status;

pub trait ReadPacket {
    async fn read_packet(socket: &mut Slice) -> Self;
}

pub struct Packet {
    pub id: i32,
    pub data: Slice,
}

pub async fn parse_packet(socket: &mut TcpReader<'_>) -> Packet {
    let length = socket.read_varint().await as usize;
    let mut data = Vec::with_capacity(length);
    unsafe { data.set_len(length) };
    socket.read(&mut data).await.unwrap();

    let mut slice = Slice::new(data.into_boxed_slice());
    let id = slice.read_varint().await;

    log::info!("DONE PARSING!");
    Timer::after_millis(100).await;

    Packet { id, data: slice }
}

pub mod handshake {
    use alloc::string::String;
    use embassy_net::tcp::TcpReader;
    use embassy_time::Timer;
    use log::info;

    use crate::{
        net::State,
        read::{ReadExtension, Slice},
    };

    use super::ReadPacket;

    pub struct HandshakePacket {
        pub protocol_version: i32,
        pub server_address: String,
        pub server_port: u16,
        pub next_state: State,
    }

    impl ReadPacket for HandshakePacket {
        async fn read_packet(socket: &mut Slice) -> Self {
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
