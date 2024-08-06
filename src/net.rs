use crate::read::ReadExtension;
use embassy_net::tcp::{TcpReader, TcpSocket};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::Timer;
use heapless::Vec;
use log::info;

#[embassy_executor::task]
pub async fn handle_conn(mut socket: TcpSocket<'static>) -> ! {
    info!("Handling connection");
    Timer::after_millis(100).await;

    //let (mut read, mut write) = socket.split();

    let state = State::Handshake;
    let channel: Channel<ThreadModeRawMutex, PacketEvent, 4> = Channel::new();

    loop {
        read_packets(&mut socket, &channel, &state).await;
    }
}

async fn read_packets(
    socket: &mut TcpSocket<'_>,
    channel: &Channel<ThreadModeRawMutex, PacketEvent, 4>,
    state: &State,
) {
    let mut buf = [0u8; 1024];
    let no = socket.read(&mut buf).await.unwrap();
    info!("Read {} packet: {:?}", no, buf);
    /*match state {
        State::Handshake => {
            let id = socket.read_varint().await;
            match id {
                0x00 => {
                    let protocol_version = socket.read_varint().await;
                    let server_address = socket.read_string::<255>().await;
                    let server_port = socket.read_u16().await;
                    let next_state = socket.read_varint().await;
                    log::info!(
                        "Handshake packet: protocol_version: {}, server_address: {}, server_port: {}, next_state: {}",
                        protocol_version, server_address, server_port, next_state
                    );
                }
                _ => {}
            }
        }
        _ => {}
    }*/
}

pub enum PacketEvent {}

pub enum State {
    Handshake = 0,
    Status = 1,
    Login = 2,
    Transfer = 3,
}
