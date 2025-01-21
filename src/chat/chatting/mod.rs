use serde::{Deserialize, Serialize};

pub mod chat_handler;
pub mod chat_room;
pub mod chat_service;

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct ChatRoomId(String);
