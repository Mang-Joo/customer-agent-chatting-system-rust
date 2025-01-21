use std::{collections::HashMap, sync::Arc};

use anyhow::{Ok, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::ChatRoomId;

// 채팅방 정보
#[derive(Debug, Clone, Serialize)]
struct ChatRoom {
    room_id: ChatRoomId,
    customer_id: String,
    agent_id: Option<String>,
    status: RoomStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum RoomStatus {
    Waiting,   // 상담원 배정 대기
    Connected, // 상담 진행 중
    Ended,     // 종료됨
}

#[derive(Debug, Clone)]
pub struct ChatRooms {
    rooms: Arc<RwLock<HashMap<ChatRoomId, ChatRoom>>>,
}

impl ChatRooms {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_room(&self, customer_id: i64) -> Result<String> {
        let room_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let chat_room = ChatRoom {
            room_id: ChatRoomId(room_id.to_string()),
            customer_id: customer_id.to_string(),
            agent_id: None,
            status: RoomStatus::Waiting,
            created_at: now,
            updated_at: now,
        };

        {
            let mut rooms = self.rooms.write().await;
            let chat_room_id = ChatRoomId(room_id.to_string());
            rooms.insert(chat_room_id, chat_room);
            println!("Rooms are {:?}", rooms);
        }

        Ok(room_id.to_string())
    }

    pub async fn is_available_room(&self, chat_room_id: ChatRoomId) -> bool {
        let rooms = self.rooms.read().await;
        rooms.contains_key(&chat_room_id)
    }
}
