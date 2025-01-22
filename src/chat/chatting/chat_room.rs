use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    config::{error::AppError, MangJooResult},
    user::user::UserRole,
};

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

impl ChatRoom {
    pub fn enter_agent(&mut self, agent_id: String) -> MangJooResult<()> {
        if self.agent_id.is_some() {
            return Err(AppError::InvalidRequest(
                "This chat room is full".to_string(),
            ));
        }
        self.agent_id = Some(agent_id);
        self.status = RoomStatus::Connected;
        self.updated_at = Utc::now();

        Ok(())
    }

    pub fn end_chat(&mut self) {
        self.status = RoomStatus::Ended;
        self.updated_at = Utc::now();
    }
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

    pub async fn create_room(&self, customer_id: i64) -> MangJooResult<String> {
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

    pub async fn is_available_room(&self, chat_room_id: &ChatRoomId, user_id: i64) -> bool {
        let rooms = self.rooms.read().await;
        let room = rooms.get(chat_room_id);
        if let Some(room) = room {
            room.customer_id == user_id.to_string()
        } else {
            false
        }
    }

    pub async fn enter_room(
        &self,
        role: UserRole,
        chat_room_id: ChatRoomId,
        user_id: i64,
    ) -> MangJooResult<()> {
        if role.is_user() {
            return Err(AppError::InvalidRequest("Can't enter on user".to_string()));
        }

        let mut rooms = self.rooms.write().await;
        let room = rooms
            .get_mut(&chat_room_id)
            .ok_or_else(|| AppError::InvalidRequest("Not found chat room".to_string()))?;

        let _ = room.enter_agent(user_id.to_string())?;

        Ok(())
    }

    pub async fn remove_room(&self, chat_room_id: &ChatRoomId) -> MangJooResult<()> {
        let mut rooms = self.rooms.write().await;
        let _ = rooms.remove(chat_room_id);

        Ok(())
    }
}
