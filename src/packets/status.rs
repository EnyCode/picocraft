use alloc::{string::String, vec::Vec};

use crate::read::ReadExtension;

use super::ReadPacket;
use serde::Serialize;

// We don't have a StatusRequest packet because its empty and theres no point

pub struct StatusResponse {
    pub data: String,
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
    pub text: StatusJson,
}
