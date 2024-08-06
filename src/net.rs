use crate::{
    packets::{handshake::HandshakePacket, parse_packet, ReadPacket},
    read::ReadExtension,
};
use embassy_net::tcp::{TcpReader, TcpSocket};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::Timer;
use heapless::Vec;
use log::info;

#[embassy_executor::task]
pub async fn handle_conn(
    mut socket: TcpSocket<'static>,
    //rx_buf: [u8; 8192],
    //tx_buf: [u8; 8192],
) -> ! {
    info!("Handling connection");
    Timer::after_millis(100).await;

    let (mut read, mut write) = socket.split();

    let mut state = State::Handshake;
    let channel: Channel<ThreadModeRawMutex, PacketEvent, 4> = Channel::new();

    loop {
        read_packets(&mut read, &channel, &state).await;

        loop {
            let msg = match channel.try_receive() {
                Ok(msg) => msg,
                Err(_) => break,
            };

            match msg {
                PacketEvent::ChangeState(new_state) => {
                    info!("Changing state to {:?}", new_state);
                    Timer::after_millis(100).await;
                    state = new_state;
                }
            }
        }
    }
}

async fn read_packets(
    socket: &mut TcpReader<'_>,
    channel: &Channel<ThreadModeRawMutex, PacketEvent, 4>,
    state: &State,
) {
    match state {
        State::Handshake => {
            let mut packet = parse_packet(socket).await;
            info!("Received packet with id {}", packet.id);
            Timer::after_millis(100).await;
            match packet.id {
                0x00 => {
                    let packet = HandshakePacket::read_packet(&mut packet.data).await;

                    info!(
                        "Received handshake packet {} {:?}",
                        packet.protocol_version, packet.next_state
                    );
                    Timer::after_millis(100).await;

                    channel
                        .send(PacketEvent::ChangeState(packet.next_state))
                        .await;
                }
                _ => {}
            }
        }
        _ => {}
    }
}

pub enum PacketEvent {
    ChangeState(State),
}

#[derive(Debug)]
#[repr(i32)]
pub enum State {
    Handshake = 0,
    Status = 1,
    Login = 2,
    Transfer = 3,
    Custom(i32) = 4,
}
