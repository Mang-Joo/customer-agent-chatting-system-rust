use chat_room::ChatRoom;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod chat_handler;
pub mod chat_room;
pub mod chat_service;
pub mod repository;

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct ChatRoomId(Uuid);

impl From<ChatRoom> for ChatRoomId {
    fn from(value: ChatRoom) -> Self {
        Self(value.room_id().to_owned())
    }
}
