use crate::{
    packets::{
        handshake::HandshakePacket,
        parse_packet,
        status::{DescriptionData, PingRequest, PlayerData, PongResponse, StatusJson, VersionData},
        ReadPacket, WritePacket,
    },
    read::ReadExtension,
};
use alloc::string::ToString;
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
    //Timer::after_millis(100).await;

    let mut state = State::Handshake;
    let channel: Channel<ThreadModeRawMutex, PacketEvent, 4> = Channel::new();

    loop {
        info!("{}", socket.state());

        let (mut read, mut write) = socket.split();

        read_packets(&mut read, &channel, &state).await;

        loop {
            let msg = match channel.try_receive() {
                Ok(msg) => msg,
                Err(_) => break,
            };

            match msg {
                PacketEvent::ChangeState(new_state) => {
                    info!("Changing state to {:?}", new_state);
                    //Timer::after_millis(100).await;
                    state = new_state;
                }
                PacketEvent::StatusRequest => {
                    let status = StatusJson {
                        version: VersionData {
                            name: "1.20.1".to_string(),
                            protocol: 763,
                        },
                        players: Some(PlayerData {
                            max: 4,
                            online: 0,
                            sample: None,
                        }),
                        description: Some(DescriptionData {
                            text: "A PicoCraft server.".to_string(),
                        }),
                        favicon: None,
                        enforces_secure_chat: false,
                    };

                    status.write_packet(&mut write).await;
                }
                PacketEvent::PingRequest(payload) => {
                    info!("Sending pong with payload {}", payload);
                    //Timer::after_millis(100).await;
                    PongResponse { payload }.write_packet(&mut write).await;
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
    let mut packet = match parse_packet(socket).await {
        Ok(packet) => packet,
        Err(err) => {
            info!("Error parsing packet: {:?}", err);
            return;
        }
    };

    match state {
        State::Handshake => {
            info!("Received packet with id {}", packet.id);
            //Timer::after_millis(100).await;
            match packet.id {
                0x00 => {
                    info!("Packet data `{:?}`", packet.data);
                    //Timer::after_millis(100).await;
                    let packet = HandshakePacket::read_packet(&mut packet.data)
                        .await
                        .unwrap();

                    info!(
                        "Received handshake packet {} {} {} {:?}",
                        packet.protocol_version,
                        packet.server_address,
                        packet.server_port,
                        packet.next_state
                    );
                    //Timer::after_millis(100).await;

                    channel
                        .send(PacketEvent::ChangeState(packet.next_state))
                        .await;
                }
                _ => {}
            }
        }
        State::Status => {
            info!("parsed packet");

            match packet.id {
                0x00 => {
                    info!("Received status request");
                    //Timer::after_millis(100).await;
                    channel.send(PacketEvent::StatusRequest).await;
                }
                0x01 => {
                    info!("Received ping request");
                    //Timer::after_millis(100).await;
                    let ping = PingRequest::read_packet(&mut packet.data).await.unwrap();
                    channel.send(PacketEvent::PingRequest(ping.payload)).await;
                }
                _ => {
                    info!("Received unknown packet with id {}", packet.id);
                    //Timer::after_millis(100).await;
                }
            }
        }
        _ => {}
    }
}

pub enum PacketEvent {
    ChangeState(State),
    StatusRequest,
    PingRequest(i64),
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
