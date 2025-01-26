use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::config::{error::AppError, MangJooResult};

use super::ChatRoomId;

#[derive(Debug, Clone, Serialize)]
pub struct ChatRoom {
    id: i64,
    room_id: ChatRoomId,
    customer_id: i64,
    agent_id: Option<i64>,
    status: RoomStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ChatRoom {
    pub fn new(customer_id: i64) -> Self {
        Self {
            id: 0,
            room_id: ChatRoomId(Uuid::new_v4()),
            customer_id,
            agent_id: None,
            status: RoomStatus::Waiting,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn from(
        id: i64,
        room_id: Uuid,
        customer_id: i64,
        agent_id: Option<i64>,
        status: RoomStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            room_id: ChatRoomId(room_id),
            customer_id,
            agent_id,
            status,
            created_at,
            updated_at,
        }
    }

    pub fn enter_agent(&mut self, agent_id: i64) -> MangJooResult<()> {
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

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn room_id(&self) -> &Uuid {
        &self.room_id.0
    }

    pub fn customer_id(&self) -> i64 {
        self.customer_id
    }

    pub fn agent_id(&self) -> Option<i64> {
        self.agent_id
    }

    pub fn status(&self) -> &RoomStatus {
        &self.status
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum RoomStatus {
    Waiting,
    Connected,
    Ended,
}

impl ToString for RoomStatus {
    fn to_string(&self) -> String {
        match self {
            RoomStatus::Waiting => "Waiting".to_string(),
            RoomStatus::Connected => "Connected".to_string(),
            RoomStatus::Ended => "Ended".to_string(),
        }
    }
}

impl FromStr for RoomStatus {
    type Err = AppError;

    fn from_str(s: &str) -> MangJooResult<Self> {
        match s.to_uppercase().as_str() {
            "WAITING" => Ok(RoomStatus::Waiting),
            "CONNECTED" => Ok(RoomStatus::Connected),
            "ENDED" => Ok(RoomStatus::Ended),
            _ => Err(AppError::InternalError(format!(
                "Invalid RoomStatus: {}",
                s
            ))),
        }
    }
}
