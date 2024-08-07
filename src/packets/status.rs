use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    read::{ReadExtension, Slice},
    write::WriteExtension,
};
use embassy_net::tcp::TcpWriter;

use super::{ReadPacket, WritePacket};
use serde::Serialize;

// We don't have a StatusRequest packet because its empty and theres no point
// We also don't have a StatusResponse packet since its just a wrapper over StatusJson

const DATA: &'static str = r#"{"version":{"name":"1.20.1","protocol":763},"players":{"max":4,"online":0,},"description":{"text":"Hello, world!"},"enforcesSecureChat":false}"#;

impl WritePacket for StatusJson {
    async fn write_packet(&self, socket: &mut TcpWriter<'_>) {
        let data = DATA.as_bytes();
        //serde_json_core::ser::to_slice(self, &mut data).unwrap();
        let length = data.len() as i32;

        log::info!("len {}", length);

        socket.write_varint(145).await;
        socket.write_varint(0).await;
        socket.write(data).await.unwrap();

        log::info!("{:?}", data);

        socket.flush().await.unwrap();

        log::info!("DONE WRITING!");
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusJson {
    pub version: VersionData,
    pub players: Option<PlayerData>,
    // TODO: use text component
    pub description: Option<DescriptionData>,
    pub favicon: Option<String>,
    pub enforces_secure_chat: bool,
}

#[derive(Serialize)]
pub struct VersionData {
    pub name: String,
    pub protocol: i32,
}

#[derive(Serialize)]
pub struct PlayerData {
    pub max: u32,
    pub online: u32,
    pub sample: Option<Vec<SamplePlayer>>,
}

#[derive(Serialize)]
pub struct SamplePlayer {
    pub name: String,
    pub id: String,
}

#[derive(Serialize)]
pub struct DescriptionData {
    pub text: String,
}

pub struct PingRequest {
    pub payload: i64,
}

impl ReadPacket for PingRequest {
    async fn read_packet(socket: &mut Slice) -> Self {
        PingRequest {
            payload: socket.read_i64().await,
        }
    }
}

pub struct PongResponse {
    pub payload: i64,
}

impl WritePacket for PongResponse {
    async fn write_packet(&self, socket: &mut TcpWriter<'_>) {
        // TODO: better mechanism for lengths
        socket.write_varint(1 + 8).await;
        socket.write_varint(0x01).await;
        socket.write_i64(self.payload).await;
    }
}
