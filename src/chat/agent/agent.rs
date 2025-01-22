use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::RwLock;

use crate::{
    chat::chatting::ChatRoomId,
    config::{error::AppError, MangJooResult},
};

// 상담원 정보
#[derive(Debug, Clone, Serialize)]
pub struct Agent {
    pub agent_id: String,
    pub name: String,
    pub status: AgentStatus,
    pub active_room_id: Option<ChatRoomId>, // 현재 상담 중인 방
    pub last_active: DateTime<Utc>,
}

impl Agent {
    pub fn new(
        agent_id: String,
        name: String,
        status: AgentStatus,
        active_room_id: Option<ChatRoomId>,
        last_active: DateTime<Utc>,
    ) -> Self {
        Self {
            agent_id,
            name,
            status,
            active_room_id,
            last_active,
        }
    }

    pub fn is_available(&self) -> bool {
        self.status == AgentStatus::Available
    }

    pub fn update_agent_status(&mut self, agent_status: AgentStatus) {
        self.status = agent_status;
        self.last_active = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum AgentStatus {
    Available,
    Busy,
    Away,
}

#[derive(Debug, Clone)]
pub struct Agents {
    agents: Arc<RwLock<HashMap<i64, Agent>>>,
}

impl Agents {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn find_available_agent(&self) -> Option<i64> {
        let agents = self.agents.read().await;
        agents
            .iter()
            .find(|(_, agent)| agent.status == AgentStatus::Available)
            .map(|(agent_id, _)| agent_id.clone())
    }

    pub async fn update_agent_status(
        &self,
        agent_id: i64,
        status: AgentStatus,
    ) -> MangJooResult<i64> {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(&agent_id) {
            agent.update_agent_status(status);
            Ok(agent_id)
        } else {
            Err(AppError::InvalidRequest("agent update Failed".to_string()))
        }
    }
}
