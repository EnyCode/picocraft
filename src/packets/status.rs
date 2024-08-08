use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use embassy_time::Timer;

use crate::{
    read::{ReadExtension, Slice},
    write::WriteExtension,
};
use embassy_net::tcp::{Error, TcpWriter};

use super::{ReadPacket, WritePacket};
use serde::Serialize;
use serde_json_core::ser;

// We don't have a StatusRequest packet because its empty and theres no point
// We also don't have a StatusResponse packet since its just a wrapper over StatusJson

impl WritePacket for StatusJson {
    async fn write_packet(&self, socket: &mut TcpWriter<'_>) {
        let mut string = [0; 256];
        let written = ser::to_slice(self, &mut string).unwrap();

        let mut data = Slice::new(Vec::new().into_boxed_slice());

        data.write_varint(0).await;
        data.write_varint(written as i32).await;
        data.write(&string[..written]).await.unwrap();

        socket.write_varint(data.buf.len() as i32).await;
        socket.write(&data.buf).await.unwrap();

        socket.flush().await.unwrap();

        log::info!("DONE WRITING!");
        //Timer::after_millis(100).await;
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
    async fn read_packet(socket: &mut Slice) -> Result<Self, Error> {
        Ok(PingRequest {
            payload: socket.read_i64().await?,
        })
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
